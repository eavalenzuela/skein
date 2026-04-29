// Phase A — bookshelf backend.
//
// Books are first-class folders at the vault root. This module owns the
// CRUD verbs (create / rename / delete / move-to-folio / reorder) plus a
// per-vault `.skein/books.json` metadata file that records explicit order.
//
// Names known by the order file are listed first in the recorded order;
// names absent from it fall through alphabetical at the end. That way a
// vault edited by hand (someone makes a new folder outside the app) still
// produces a sane list, and the order file never has to be perfectly in
// sync with the filesystem.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::vault::{Book, Vault};

const META_DIR: &str = ".skein";
const META_FILE: &str = "books.json";
// Names we never let users use for a book — they collide with reserved
// app paths or behave badly on disk.
const FORBIDDEN_NAMES: &[&str] = &[".skein", ".git", "..", "."];

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct BooksMeta {
    #[serde(default)]
    pub order: Vec<String>,
}

fn meta_path(vault: &Vault) -> PathBuf {
    vault.root.join(META_DIR).join(META_FILE)
}

pub fn read_meta(vault: &Vault) -> BooksMeta {
    let path = meta_path(vault);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn write_meta(vault: &Vault, meta: &BooksMeta) -> Result<()> {
    let dir = vault.root.join(META_DIR);
    fs::create_dir_all(&dir)?;
    let path = dir.join(META_FILE);
    fs::write(path, serde_json::to_string_pretty(meta)?)?;
    Ok(())
}

/// Reorder a book list against a meta record. Names listed in `meta.order`
/// keep that order; names absent from it land at the end in case-insensitive
/// alphabetical order.
pub fn apply_order(mut books: Vec<Book>, meta: &BooksMeta) -> Vec<Book> {
    let priority: std::collections::HashMap<&str, usize> = meta
        .order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();
    books.sort_by(|a, b| {
        let pa = priority.get(a.name.as_str()).copied();
        let pb = priority.get(b.name.as_str()).copied();
        match (pa, pb) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    books
}

fn validate_name(name: &str) -> Result<()> {
    let n = name.trim();
    if n.is_empty() {
        return Err(anyhow!("book name is empty"));
    }
    if n.len() > 100 {
        return Err(anyhow!("book name is too long"));
    }
    if FORBIDDEN_NAMES.contains(&n) {
        return Err(anyhow!("book name `{n}` is reserved"));
    }
    if n.starts_with('.') {
        return Err(anyhow!("book name cannot start with `.`"));
    }
    if n.contains('/') || n.contains('\\') {
        return Err(anyhow!("book name cannot contain path separators"));
    }
    if n.chars().any(|c| matches!(c, '\0' | '\n' | '\r' | '\t')) {
        return Err(anyhow!("book name has invalid whitespace"));
    }
    Ok(())
}

fn book_dir(vault: &Vault, name: &str) -> PathBuf {
    vault.root.join(name)
}

pub fn create_book(vault: &Vault, name: &str) -> Result<()> {
    validate_name(name)?;
    let dir = book_dir(vault, name);
    if dir.exists() {
        return Err(anyhow!("a folder named `{name}` already exists"));
    }
    fs::create_dir(&dir).with_context(|| format!("creating {}", dir.display()))?;
    Ok(())
}

pub fn rename_book(vault: &Vault, old: &str, new: &str) -> Result<()> {
    if old == new {
        return Ok(());
    }
    validate_name(new)?;
    let from = book_dir(vault, old);
    let to = book_dir(vault, new);
    if !from.is_dir() {
        return Err(anyhow!("book `{old}` not found"));
    }
    if to.exists() {
        return Err(anyhow!("a folder named `{new}` already exists"));
    }
    fs::rename(&from, &to)
        .with_context(|| format!("renaming {} → {}", from.display(), to.display()))?;
    // Patch the order list so the rename survives the next list_books call.
    let mut meta = read_meta(vault);
    let mut changed = false;
    for entry in meta.order.iter_mut() {
        if entry == old {
            *entry = new.to_string();
            changed = true;
        }
    }
    if changed {
        write_meta(vault, &meta)?;
    }
    Ok(())
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct DeleteResult {
    /// Pages that were deleted from disk (their old rel_paths). Empty when
    /// pages were moved to Folio rather than deleted.
    pub deleted_rel_paths: Vec<String>,
    /// Pairs of (old_rel_path, new_rel_path) for pages moved to the vault
    /// root. Empty when pages were deleted.
    pub moved: Vec<(String, String)>,
}

pub fn delete_book(
    vault: &Vault,
    name: &str,
    also_delete_pages: bool,
) -> Result<DeleteResult> {
    let dir = book_dir(vault, name);
    if !dir.is_dir() {
        return Err(anyhow!("book `{name}` not found"));
    }

    let mut result = DeleteResult::default();

    if also_delete_pages {
        // Capture rel_paths of pages in the book before nuking the folder so
        // the index can drop their rows.
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_file() && is_md(&p) {
                if let Some(rel) = rel_string(&vault.root, &p) {
                    result.deleted_rel_paths.push(rel);
                }
            }
        }
        fs::remove_dir_all(&dir)
            .with_context(|| format!("removing {}", dir.display()))?;
    } else {
        // Move every file out to the vault root. Walk in two passes so we
        // can handle collisions deterministically. We never recurse — book
        // folders are flat per spec.
        let mut entries: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_file() {
                entries.push(p);
            }
        }
        let mut taken: HashSet<String> = fs::read_dir(&vault.root)?
            .flatten()
            .filter_map(|e| {
                e.path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect();
        for src in &entries {
            let Some(file_name) = src.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            let target_name = unique_in(&taken, file_name);
            taken.insert(target_name.clone());
            let dest = vault.root.join(&target_name);
            fs::rename(src, &dest)
                .with_context(|| format!("moving {} → {}", src.display(), dest.display()))?;
            if is_md(&dest) {
                if let (Some(old_rel), Some(new_rel)) = (
                    rel_string(&vault.root, src),
                    rel_string(&vault.root, &dest),
                ) {
                    result.moved.push((old_rel, new_rel));
                }
            }
        }
        // Remove the (now-empty) book folder.
        fs::remove_dir(&dir)
            .with_context(|| format!("removing empty {}", dir.display()))?;
    }

    // Drop the name from the order list.
    let mut meta = read_meta(vault);
    let len_before = meta.order.len();
    meta.order.retain(|n| n != name);
    if meta.order.len() != len_before {
        write_meta(vault, &meta)?;
    }
    Ok(result)
}

pub fn set_order(vault: &Vault, names: Vec<String>) -> Result<()> {
    write_meta(vault, &BooksMeta { order: names })
}

fn rel_string(root: &Path, full: &Path) -> Option<String> {
    full.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('\\', "/"))
}

fn is_md(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("md") || s.eq_ignore_ascii_case("markdown"))
}

fn unique_in(taken: &HashSet<String>, name: &str) -> String {
    if !taken.contains(name) {
        return name.to_string();
    }
    // Insert "(N)" before the extension. Markdown collisions hit this path
    // most; attachments hit it occasionally.
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], Some(&name[i..])),
        _ => (name, None),
    };
    for n in 1.. {
        let candidate = match ext {
            Some(e) => format!("{stem} ({n}){e}"),
            None => format!("{stem} ({n})"),
        };
        if !taken.contains(&candidate) {
            return candidate;
        }
    }
    unreachable!()
}
