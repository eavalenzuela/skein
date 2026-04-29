use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::chunker::{chunk_markdown, Chunk};
use crate::embedder::{bytes_to_vec, cosine, normalize, vec_to_bytes, SharedEmbedder};
use crate::vault::PageData;

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS pages (
  rowid       INTEGER PRIMARY KEY,
  rel_path    TEXT NOT NULL UNIQUE,
  title       TEXT NOT NULL,
  book        TEXT,
  body        TEXT NOT NULL,
  hash        BLOB NOT NULL,
  modified    INTEGER NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
  rel_path UNINDEXED,
  title,
  body,
  content='pages',
  content_rowid='rowid',
  tokenize='porter unicode61'
);

CREATE TRIGGER IF NOT EXISTS pages_ai AFTER INSERT ON pages BEGIN
  INSERT INTO pages_fts(rowid, rel_path, title, body)
  VALUES (new.rowid, new.rel_path, new.title, new.body);
END;

CREATE TRIGGER IF NOT EXISTS pages_ad AFTER DELETE ON pages BEGIN
  INSERT INTO pages_fts(pages_fts, rowid, rel_path, title, body)
  VALUES ('delete', old.rowid, old.rel_path, old.title, old.body);
END;

CREATE TRIGGER IF NOT EXISTS pages_au AFTER UPDATE ON pages BEGIN
  INSERT INTO pages_fts(pages_fts, rowid, rel_path, title, body)
  VALUES ('delete', old.rowid, old.rel_path, old.title, old.body);
  INSERT INTO pages_fts(rowid, rel_path, title, body)
  VALUES (new.rowid, new.rel_path, new.title, new.body);
END;

CREATE TABLE IF NOT EXISTS tags (
  rel_path TEXT NOT NULL,
  tag      TEXT NOT NULL,
  PRIMARY KEY (rel_path, tag),
  FOREIGN KEY (rel_path) REFERENCES pages(rel_path) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

CREATE TABLE IF NOT EXISTS links (
  from_rel_path TEXT NOT NULL,
  target        TEXT NOT NULL,
  target_lc     TEXT NOT NULL,
  alias         TEXT,
  PRIMARY KEY (from_rel_path, target),
  FOREIGN KEY (from_rel_path) REFERENCES pages(rel_path) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_links_target_lc ON links(target_lc);

CREATE TABLE IF NOT EXISTS embeddings (
  rel_path   TEXT NOT NULL,
  level      TEXT NOT NULL,    -- 'page' or 'chunk'
  chunk_idx  INTEGER NOT NULL, -- -1 for page-level
  heading    TEXT NOT NULL DEFAULT '',
  text       TEXT NOT NULL,
  hash       BLOB NOT NULL,
  model      TEXT NOT NULL,
  vector     BLOB NOT NULL,
  PRIMARY KEY (rel_path, level, chunk_idx, model),
  FOREIGN KEY (rel_path) REFERENCES pages(rel_path) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_embeddings_level ON embeddings(level, model);

CREATE TABLE IF NOT EXISTS vector_cache (
  hash   BLOB NOT NULL,
  model  TEXT NOT NULL,
  vector BLOB NOT NULL,
  PRIMARY KEY (hash, model)
);
"#;

pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("index.db")
}

pub struct Index {
    conn: Connection,
    embedder: SharedEmbedder,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub rel_path: String,
    pub title: String,
    pub book: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelatedHit {
    pub rel_path: String,
    pub title: String,
    pub book: Option<String>,
    pub similarity: f32,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageTitle {
    pub rel_path: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacklinkHit {
    pub from_rel_path: String,
    pub from_title: String,
    pub from_book: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkHit {
    pub rel_path: String,
    pub title: String,
    pub heading: String,
    pub text: String,
    pub similarity: f32,
}

impl Index {
    pub fn open(path: &Path, embedder: SharedEmbedder) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| format!("opening index at {}", path.display()))?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn, embedder })
    }

    pub fn upsert_page(&mut self, data: &PageData) -> Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(data.body.as_bytes());
        let hash: Vec<u8> = hasher.finalize().to_vec();

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO pages (rel_path, title, book, body, hash, modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(rel_path) DO UPDATE SET
               title = excluded.title,
               book = excluded.book,
               body = excluded.body,
               hash = excluded.hash,
               modified = excluded.modified",
            params![
                data.rel_path,
                data.title,
                data.book,
                data.body,
                hash,
                data.modified
            ],
        )?;
        tx.execute(
            "DELETE FROM tags WHERE rel_path = ?1",
            params![data.rel_path],
        )?;
        {
            let mut stmt =
                tx.prepare("INSERT OR IGNORE INTO tags (rel_path, tag) VALUES (?1, ?2)")?;
            for tag in &data.tags {
                stmt.execute(params![data.rel_path, tag])?;
            }
        }
        tx.commit()?;

        // Replace links extracted from the body.
        self.replace_links(&data.rel_path, &data.body)?;

        // Embed only after the page row commits so foreign keys hold.
        self.embed_page(&data.rel_path, &data.body)?;
        Ok(())
    }

    fn replace_links(&mut self, rel_path: &str, body: &str) -> Result<()> {
        let parsed = parse_wikilinks(body);
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM links WHERE from_rel_path = ?1",
            params![rel_path],
        )?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO links (from_rel_path, target, target_lc, alias)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;
            for link in &parsed {
                stmt.execute(params![
                    rel_path,
                    link.target,
                    link.target.to_lowercase(),
                    link.alias.as_deref(),
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_page_titles(&self) -> Result<Vec<PageTitle>> {
        let mut stmt = self
            .conn
            .prepare("SELECT rel_path, title FROM pages ORDER BY title COLLATE NOCASE")?;
        let rows = stmt.query_map([], |row| {
            Ok(PageTitle {
                rel_path: row.get(0)?,
                title: row.get(1)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn find_backlinks(&self, rel_path: &str) -> Result<Vec<BacklinkHit>> {
        // Build the set of identifiers other notes might use to link here:
        // the rel_path, the file stem, and the page's frontmatter title.
        let row: rusqlite::Result<(String, Option<String>)> = self.conn.query_row(
            "SELECT title, book FROM pages WHERE rel_path = ?1",
            params![rel_path],
            |r| Ok((r.get(0)?, r.get(1)?)),
        );
        let (title, _book) = match row {
            Ok(t) => t,
            Err(_) => return Ok(vec![]),
        };

        let mut targets: Vec<String> = vec![rel_path.to_string()];
        let stem = std::path::Path::new(rel_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        if let Some(s) = stem {
            targets.push(s);
        }
        targets.push(title.clone());
        targets.dedup();
        let lower_targets: Vec<String> = targets.iter().map(|t| t.to_lowercase()).collect();

        // Build a SQL IN(?,?,?) with a variable number of params.
        let placeholders = lower_targets
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(",");
        let q = format!(
            "SELECT DISTINCT l.from_rel_path, p.title, p.book, l.alias
             FROM links l
             JOIN pages p ON p.rel_path = l.from_rel_path
             WHERE l.target_lc IN ({placeholders})
             AND l.from_rel_path != ?{}",
            lower_targets.len() + 1
        );
        let mut stmt = self.conn.prepare(&q)?;
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = lower_targets
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();
        params_vec.push(&rel_path);
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(BacklinkHit {
                from_rel_path: row.get(0)?,
                from_title: row.get(1)?,
                from_book: row.get(2)?,
                alias: row.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        out.sort_by(|a, b| {
            a.from_title
                .to_lowercase()
                .cmp(&b.from_title.to_lowercase())
        });
        Ok(out)
    }

    pub fn delete_page(&mut self, rel_path: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM pages WHERE rel_path = ?1", params![rel_path])?;
        Ok(())
    }

    pub fn all_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT tag FROM tags ORDER BY tag COLLATE NOCASE")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete_all(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM pages; DELETE FROM tags; DELETE FROM embeddings; \
             DELETE FROM pages_fts; \
             INSERT INTO pages_fts(pages_fts) VALUES('rebuild');",
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let fts_query = sanitize_fts_query(query);
        let mut stmt = self.conn.prepare(
            "SELECT pages.rel_path, pages.title, pages.book,
                    snippet(pages_fts, 2, '<mark>', '</mark>', '…', 12) AS snip
             FROM pages_fts
             JOIN pages ON pages.rowid = pages_fts.rowid
             WHERE pages_fts MATCH ?1
             ORDER BY bm25(pages_fts) ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![fts_query, limit as i64], |row| {
            Ok(SearchHit {
                rel_path: row.get(0)?,
                title: row.get(1)?,
                book: row.get(2)?,
                snippet: row.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn embed_page(&mut self, rel_path: &str, body: &str) -> Result<()> {
        let model_name = self.embedder.name().to_string();
        let chunks: Vec<Chunk> = chunk_markdown(body);
        let mut chunk_vectors: Vec<Vec<f32>> = Vec::with_capacity(chunks.len());

        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM embeddings WHERE rel_path = ?1 AND model = ?2",
            params![rel_path, model_name],
        )?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO embeddings (rel_path, level, chunk_idx, heading, text, hash, model, vector)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            let mut cache_get = tx.prepare(
                "SELECT vector FROM vector_cache WHERE hash = ?1 AND model = ?2",
            )?;
            let mut cache_put = tx.prepare(
                "INSERT OR REPLACE INTO vector_cache (hash, model, vector) VALUES (?1, ?2, ?3)",
            )?;

            for chunk in &chunks {
                let mut hasher = Sha256::new();
                hasher.update(chunk.text.as_bytes());
                let hash: Vec<u8> = hasher.finalize().to_vec();

                // Skip the embedder if a vector for this exact chunk text +
                // model is already cached (round-trip restore, or another
                // page sharing identical content).
                let cached: Option<Vec<u8>> = cache_get
                    .query_row(params![hash, model_name], |r| r.get::<_, Vec<u8>>(0))
                    .ok();
                let v = match cached {
                    Some(bytes) => bytes_to_vec(&bytes),
                    None => {
                        let computed = self.embedder.embed(&chunk.text);
                        cache_put.execute(params![hash, model_name, vec_to_bytes(&computed)])?;
                        computed
                    }
                };

                stmt.execute(params![
                    rel_path,
                    "chunk",
                    chunk.idx as i64,
                    chunk.heading,
                    chunk.text,
                    hash,
                    model_name,
                    vec_to_bytes(&v),
                ])?;
                chunk_vectors.push(v);
            }

            // Page-level vector: mean of chunk vectors (renormalized) or, for
            // empty bodies, the embedder's zero-output. Page-level vectors
            // are also content-hashed for cache reuse.
            let mut page_hasher = Sha256::new();
            page_hasher.update(body.as_bytes());
            let page_hash: Vec<u8> = page_hasher.finalize().to_vec();

            let page_vec = if chunk_vectors.is_empty() {
                let cached: Option<Vec<u8>> = cache_get
                    .query_row(params![page_hash, model_name], |r| r.get::<_, Vec<u8>>(0))
                    .ok();
                match cached {
                    Some(bytes) => bytes_to_vec(&bytes),
                    None => {
                        let computed = self.embedder.embed(body);
                        cache_put.execute(params![
                            page_hash,
                            model_name,
                            vec_to_bytes(&computed)
                        ])?;
                        computed
                    }
                }
            } else {
                let dim = chunk_vectors[0].len();
                let mut acc = vec![0f32; dim];
                for v in &chunk_vectors {
                    for i in 0..dim {
                        acc[i] += v[i];
                    }
                }
                let n = chunk_vectors.len() as f32;
                for x in acc.iter_mut() {
                    *x /= n;
                }
                normalize(&mut acc);
                acc
            };

            stmt.execute(params![
                rel_path,
                "page",
                -1_i64,
                "",
                body,
                page_hash,
                model_name,
                vec_to_bytes(&page_vec),
            ])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Pre-populate the vector cache from a sidecar SQLite produced by
    /// `archive::export_vault`. Returns the number of rows imported.
    pub fn import_vector_cache(&mut self, sidecar_db: &std::path::Path) -> Result<usize> {
        if !sidecar_db.exists() {
            return Ok(0);
        }
        let attach = sidecar_db
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-utf8 sidecar path"))?
            .replace('\'', "''");
        self.conn.execute_batch(&format!(
            "ATTACH DATABASE '{attach}' AS sc;
             INSERT OR REPLACE INTO vector_cache (hash, model, vector)
             SELECT hash, model, vector FROM sc.vectors;
             DETACH DATABASE sc;"
        ))?;
        let n: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM vector_cache", [], |r| r.get(0))?;
        Ok(n)
    }

    /// Cosine over the chunk-level vectors, optionally excluding chunks
    /// from a given page (used to keep RAG from echoing the active page back
    /// at the model). Used by Phase 7 chat for "current + related" context.
    pub fn retrieve_chunks(
        &self,
        query: &str,
        limit: usize,
        exclude_rel_path: Option<&str>,
    ) -> Result<Vec<ChunkHit>> {
        let model_name = self.embedder.name().to_string();
        let q = self.embedder.embed(query);

        let mut stmt = self.conn.prepare(
            "SELECT e.rel_path, p.title, e.heading, e.text, e.vector
             FROM embeddings e
             JOIN pages p ON p.rel_path = e.rel_path
             WHERE e.level = 'chunk' AND e.model = ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![model_name], |row| {
            let rel_path: String = row.get(0)?;
            let title: String = row.get(1)?;
            let heading: String = row.get(2)?;
            let text: String = row.get(3)?;
            let bytes: Vec<u8> = row.get(4)?;
            Ok((rel_path, title, heading, text, bytes_to_vec(&bytes)))
        })?;

        let mut hits: Vec<(f32, ChunkHit)> = Vec::new();
        for r in rows {
            let (rp, title, heading, text, v) = r?;
            if let Some(ex) = exclude_rel_path {
                if rp == ex {
                    continue;
                }
            }
            let sim = cosine(&q, &v);
            if sim <= 0.0 {
                continue;
            }
            hits.push((
                sim,
                ChunkHit {
                    rel_path: rp,
                    title,
                    heading,
                    text,
                    similarity: sim,
                },
            ));
        }
        hits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(hits.into_iter().take(limit).map(|(_, h)| h).collect())
    }

    pub fn find_related(&self, rel_path: &str, limit: usize) -> Result<Vec<RelatedHit>> {
        let model_name = self.embedder.name().to_string();
        let target: Vec<f32> = match self
            .conn
            .query_row(
                "SELECT vector FROM embeddings
                 WHERE rel_path = ?1 AND level = 'page' AND model = ?2",
                params![rel_path, model_name],
                |row| {
                    let bytes: Vec<u8> = row.get(0)?;
                    Ok(bytes_to_vec(&bytes))
                },
            )
            .ok()
        {
            Some(v) => v,
            None => return Ok(vec![]),
        };

        let mut stmt = self.conn.prepare(
            "SELECT e.rel_path, p.title, p.book, e.vector,
                    (SELECT text FROM embeddings c
                     WHERE c.rel_path = e.rel_path AND c.level = 'chunk' AND c.model = e.model
                     ORDER BY c.chunk_idx ASC LIMIT 1) AS first_chunk
             FROM embeddings e
             JOIN pages p ON p.rel_path = e.rel_path
             WHERE e.level = 'page' AND e.model = ?1 AND e.rel_path != ?2",
        )?;
        let rows = stmt.query_map(params![model_name, rel_path], |row| {
            let rp: String = row.get(0)?;
            let title: String = row.get(1)?;
            let book: Option<String> = row.get(2)?;
            let bytes: Vec<u8> = row.get(3)?;
            let snippet: Option<String> = row.get(4).ok();
            Ok((rp, title, book, bytes_to_vec(&bytes), snippet))
        })?;

        let mut hits: Vec<(f32, RelatedHit)> = Vec::new();
        for row in rows {
            let (rp, title, book, v, snip) = row?;
            let sim = cosine(&target, &v);
            if sim <= 0.0 {
                continue;
            }
            let snippet = snip
                .map(|s| {
                    let mut t = s.replace('\n', " ").trim().to_string();
                    if t.chars().count() > 140 {
                        t = t.chars().take(140).collect::<String>();
                        t.push('…');
                    }
                    t
                })
                .unwrap_or_default();
            hits.push((
                sim,
                RelatedHit {
                    rel_path: rp,
                    title,
                    book,
                    similarity: sim,
                    snippet,
                },
            ));
        }
        hits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(hits.into_iter().take(limit).map(|(_, h)| h).collect())
    }
}

#[derive(Debug, Clone)]
struct ParsedWikilink {
    target: String,
    alias: Option<String>,
}

fn parse_wikilinks(body: &str) -> Vec<ParsedWikilink> {
    let mut out = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            // Find matching ]]
            let start = i + 2;
            let mut j = start;
            let mut found_end = None;
            while j + 1 < bytes.len() {
                if bytes[j] == b']' && bytes[j + 1] == b']' {
                    found_end = Some(j);
                    break;
                }
                if bytes[j] == b'\n' {
                    break;
                }
                j += 1;
            }
            if let Some(end) = found_end {
                if let Ok(inner) = std::str::from_utf8(&bytes[start..end]) {
                    let inner = inner.trim();
                    if !inner.is_empty() {
                        let (target, alias) = match inner.split_once('|') {
                            Some((t, a)) => (t.trim().to_string(), Some(a.trim().to_string())),
                            None => (inner.to_string(), None),
                        };
                        out.push(ParsedWikilink { target, alias });
                    }
                }
                i = end + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn sanitize_fts_query(input: &str) -> String {
    let mut tokens = Vec::new();
    for raw in input.split_whitespace() {
        let cleaned: String = raw
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if cleaned.is_empty() {
            continue;
        }
        tokens.push(format!("\"{}\"*", cleaned));
    }
    tokens.join(" AND ")
}
