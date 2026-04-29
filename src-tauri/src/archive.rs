// Phase 12 — vault export.
//
// Produces a zip with the vault contents at the root and a content-addressed
// embeddings sidecar at `.skein/vectors.db`. The sidecar holds vectors keyed
// by chunk-text SHA-256 + model name so that a re-index after restore can
// reuse them instead of re-running the embedder.
//
// `.skein/` at the vault root is reserved and skipped during export so that
// stale sidecars don't accumulate inside the source vault.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const SIDECAR_RELATIVE_PATH: &str = ".skein/vectors.db";

const SIDECAR_SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

CREATE TABLE IF NOT EXISTS vectors (
  hash    BLOB NOT NULL,
  level   TEXT NOT NULL,
  heading TEXT NOT NULL DEFAULT '',
  text    TEXT NOT NULL,
  model   TEXT NOT NULL,
  vector  BLOB NOT NULL,
  PRIMARY KEY (hash, model)
);
"#;

pub fn export_vault(vault_root: &Path, index_db: &Path, dest_zip: &Path) -> Result<()> {
    let file = File::create(dest_zip)
        .with_context(|| format!("creating archive {}", dest_zip.display()))?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let canonical_root = vault_root.canonicalize().with_context(|| {
        format!("canonicalizing vault root {}", vault_root.display())
    })?;

    let mut buf = Vec::with_capacity(64 * 1024);
    for entry in WalkDir::new(&canonical_root).into_iter().filter_entry(|e| {
        // Skip the reserved `.skein/` directory at the vault root.
        if e.depth() == 1 && e.file_name() == ".skein" {
            return false;
        }
        true
    }) {
        let entry = entry.with_context(|| "walking vault")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(&canonical_root)
            .with_context(|| format!("stripping prefix from {}", path.display()))?;
        let name = path_to_zip_name(rel);
        zip.start_file(&name, opts)?;
        let mut f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
        buf.clear();
        f.read_to_end(&mut buf)
            .with_context(|| format!("reading {}", path.display()))?;
        zip.write_all(&buf)?;
    }

    let sidecar_tmp = build_sidecar(index_db, dest_zip)?;
    {
        zip.start_file(SIDECAR_RELATIVE_PATH, opts)?;
        let mut f = File::open(&sidecar_tmp)?;
        buf.clear();
        f.read_to_end(&mut buf)?;
        zip.write_all(&buf)?;
    }
    let _ = std::fs::remove_file(&sidecar_tmp);

    zip.finish()?;
    Ok(())
}

fn path_to_zip_name(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Extract a zip archive into `dest_dir`. The dest must either not exist or
/// be empty; we don't merge into populated directories here to avoid
/// silently overwriting user files.
pub fn extract_archive(archive: &Path, dest_dir: &Path) -> Result<()> {
    if dest_dir.exists() {
        let mut iter = std::fs::read_dir(dest_dir)
            .with_context(|| format!("reading {}", dest_dir.display()))?;
        if iter.next().is_some() {
            return Err(anyhow!(
                "destination is not empty: {}",
                dest_dir.display()
            ));
        }
    } else {
        std::fs::create_dir_all(dest_dir)?;
    }
    let canonical_dest = dest_dir
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", dest_dir.display()))?;

    let file = File::open(archive)
        .with_context(|| format!("opening archive {}", archive.display()))?;
    let mut zip = ZipArchive::new(file)?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(name) = entry.enclosed_name() else {
            continue; // skip entries with traversal-y names
        };
        let target = canonical_dest.join(&name);
        // Defense in depth — enclosed_name should already prevent escape, but
        // re-verify after join so a symlinked dest can't be exploited.
        if !target.starts_with(&canonical_dest) {
            return Err(anyhow!("zip entry escapes destination: {}", name.display()));
        }
        if entry.is_dir() {
            std::fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&target)
            .with_context(|| format!("creating {}", target.display()))?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(())
}

fn build_sidecar(index_db: &Path, near: &Path) -> Result<PathBuf> {
    let parent = near.parent().unwrap_or(Path::new("."));
    let stamp = uuid::Uuid::new_v4().simple().to_string();
    let tmp = parent.join(format!(".skein-export-{}.db", stamp));
    let conn = Connection::open(&tmp)?;
    conn.execute_batch(SIDECAR_SCHEMA)?;
    if index_db.exists() {
        let attach_path = index_db
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-utf8 index path"))?
            .replace('\'', "''");
        conn.execute_batch(&format!(
            "ATTACH DATABASE '{attach_path}' AS src;
             INSERT OR IGNORE INTO vectors (hash, level, heading, text, model, vector)
             SELECT hash, level, heading, text, model, vector FROM src.embeddings;
             DETACH DATABASE src;"
        ))?;
    }
    Ok(tmp)
}
