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

        // Embed only after the page row commits so foreign keys hold.
        self.embed_page(&data.rel_path, &data.body)?;
        Ok(())
    }

    pub fn delete_page(&mut self, rel_path: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM pages WHERE rel_path = ?1", params![rel_path])?;
        Ok(())
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

            for chunk in &chunks {
                let v = self.embedder.embed(&chunk.text);
                let mut hasher = Sha256::new();
                hasher.update(chunk.text.as_bytes());
                let hash: Vec<u8> = hasher.finalize().to_vec();
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
            // empty bodies, the embedder's zero-output.
            let page_vec = if chunk_vectors.is_empty() {
                self.embedder.embed(body)
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

            let mut hasher = Sha256::new();
            hasher.update(body.as_bytes());
            let hash: Vec<u8> = hasher.finalize().to_vec();
            stmt.execute(params![
                rel_path,
                "page",
                -1_i64,
                "",
                body,
                hash,
                model_name,
                vec_to_bytes(&page_vec),
            ])?;
        }
        tx.commit()?;
        Ok(())
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
