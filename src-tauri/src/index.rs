use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde::Serialize;
use sha2::{Digest, Sha256};

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
"#;

pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("index.db")
}

pub struct Index {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub rel_path: String,
    pub title: String,
    pub book: Option<String>,
    pub snippet: String,
}

impl Index {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| format!("opening index at {}", path.display()))?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
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
        Ok(())
    }

    pub fn delete_page(&mut self, rel_path: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM pages WHERE rel_path = ?1", params![rel_path])?;
        Ok(())
    }

    pub fn delete_all(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM pages; DELETE FROM tags; DELETE FROM pages_fts; \
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
}

/// Coerce arbitrary user input into a safe FTS5 MATCH expression. We treat
/// each whitespace-separated token as a prefix term, which covers the
/// "typing as you go" command-palette use case.
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
