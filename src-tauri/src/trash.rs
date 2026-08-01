// Vault-local trash.
//
// Deleting a page used to be `fs::remove_file` — permanent, with no undo
// anywhere in the app. Deleted pages now move to `.skein/trash/`, which is
// already excluded from indexing, export and git, so a mistaken delete is
// recoverable and nothing about the vault's on-disk shape changes.
//
// Each entry is stored under a unique id together with a small JSON record
// of where it came from, so restore can put it back exactly. Entries older
// than the retention window are swept on the next delete.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::vault::{resolve_in_vault, Vault};

const TRASH_DIR: &str = ".skein/trash";
/// Entries older than this are swept. Long enough to notice a mistake,
/// short enough that the vault doesn't quietly grow forever.
const RETENTION_SECS: u64 = 30 * 24 * 60 * 60;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrashEntry {
    /// Opaque handle the frontend passes back to `restore`.
    pub id: String,
    /// Where the page lived, vault-relative.
    pub rel_path: String,
    pub title: String,
    pub deleted_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn trash_dir(vault: &Vault) -> PathBuf {
    vault.root.join(TRASH_DIR)
}

fn meta_path(dir: &std::path::Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.json"))
}

fn blob_path(dir: &std::path::Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.md"))
}

/// Move a page into the trash. Returns the entry so the caller can offer
/// an undo.
pub fn trash_page(vault: &Vault, rel_path: &str) -> Result<TrashEntry> {
    let full = resolve_in_vault(vault, rel_path, true)?;
    if !full.is_file() {
        return Err(anyhow!("page not found: {rel_path}"));
    }
    let dir = trash_dir(vault);
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    let deleted_at = now_secs();
    let id = format!("{deleted_at}-{}", uuid::Uuid::new_v4().simple());
    let title = full
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();

    // Rename when we can (cheap, atomic); fall back to copy+remove when the
    // vault spans filesystems.
    let dest = blob_path(&dir, &id);
    if fs::rename(&full, &dest).is_err() {
        fs::copy(&full, &dest).with_context(|| format!("copying {}", full.display()))?;
        fs::remove_file(&full).with_context(|| format!("removing {}", full.display()))?;
    }

    let entry = TrashEntry {
        id: id.clone(),
        rel_path: rel_path.to_string(),
        title,
        deleted_at,
    };
    fs::write(meta_path(&dir, &id), serde_json::to_string(&entry)?)?;
    sweep(vault);
    Ok(entry)
}

/// Put a trashed page back where it came from. If something now occupies
/// that path, restore beside it rather than overwriting.
pub fn restore(vault: &Vault, id: &str) -> Result<String> {
    // The id goes into a filename, so it must not contain path syntax.
    if id.is_empty() || id.contains(['/', '\\', '.', '\0']) {
        return Err(anyhow!("invalid trash id"));
    }
    let dir = trash_dir(vault);
    let meta = meta_path(&dir, id);
    let entry: TrashEntry = serde_json::from_str(
        &fs::read_to_string(&meta).with_context(|| "this item is no longer in the trash")?,
    )?;

    let mut target_rel = entry.rel_path.clone();
    let mut target = resolve_in_vault(vault, &target_rel, false)?;
    if target.exists() {
        let stem = target
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("restored")
            .to_string();
        let parent_rel = match entry.rel_path.rfind('/') {
            Some(i) => &entry.rel_path[..i + 1],
            None => "",
        };
        for n in 1.. {
            let candidate = format!("{parent_rel}{stem} (restored {n}).md");
            let path = resolve_in_vault(vault, &candidate, false)?;
            if !path.exists() {
                target_rel = candidate;
                target = path;
                break;
            }
        }
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    let blob = blob_path(&dir, id);
    if fs::rename(&blob, &target).is_err() {
        fs::copy(&blob, &target).with_context(|| "restoring from trash")?;
        fs::remove_file(&blob).ok();
    }
    fs::remove_file(&meta).ok();
    Ok(target_rel)
}

/// Everything currently recoverable, newest first.
pub fn list(vault: &Vault) -> Vec<TrashEntry> {
    let dir = trash_dir(vault);
    let Ok(rd) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<TrashEntry> = rd
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| fs::read_to_string(e.path()).ok())
        .filter_map(|s| serde_json::from_str::<TrashEntry>(&s).ok())
        .filter(|e| blob_path(&dir, &e.id).exists())
        .collect();
    out.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
    out
}

/// Delete everything in the trash for good.
pub fn empty(vault: &Vault) -> Result<usize> {
    let dir = trash_dir(vault);
    let entries = list(vault);
    for e in &entries {
        fs::remove_file(blob_path(&dir, &e.id)).ok();
        fs::remove_file(meta_path(&dir, &e.id)).ok();
    }
    Ok(entries.len())
}

/// Drop entries past the retention window. Best-effort: a failure here
/// must never block the delete that triggered it.
fn sweep(vault: &Vault) {
    let dir = trash_dir(vault);
    let cutoff = now_secs().saturating_sub(RETENTION_SECS);
    for e in list(vault) {
        if e.deleted_at < cutoff {
            fs::remove_file(blob_path(&dir, &e.id)).ok();
            fs::remove_file(meta_path(&dir, &e.id)).ok();
        }
    }
}
