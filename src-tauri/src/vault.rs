use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Vault {
    pub root: PathBuf,
    pub name: String,
}

impl Vault {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let path = path
            .canonicalize()
            .with_context(|| format!("vault path does not exist: {}", path.display()))?;
        if !path.is_dir() {
            anyhow::bail!("vault path is not a directory: {}", path.display());
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Vault")
            .to_string();
        Ok(Self { root: path, name })
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Book {
    pub name: String,
    pub rel_path: String,
    pub page_count: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct Page {
    /// Relative path from the vault root, using forward slashes.
    pub rel_path: String,
    pub title: String,
    /// `None` for loose top-level pages; `Some(book_name)` for pages in a book.
    pub book: Option<String>,
    pub tags: Vec<String>,
    pub modified: i64,
}

fn rel_string(root: &Path, full: &Path) -> Option<String> {
    full.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('\\', "/"))
}

fn read_modified(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn extract_frontmatter(body: &str) -> (Option<String>, Vec<String>) {
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(body);
    let Some(data) = parsed.data else {
        return (None, vec![]);
    };
    let Ok(map) = data.as_hashmap() else {
        return (None, vec![]);
    };
    let title = map.get("title").and_then(|v| v.as_string().ok());
    let tags = map
        .get("tags")
        .and_then(|v| v.as_vec().ok())
        .map(|vs| {
            vs.into_iter()
                .filter_map(|v| v.as_string().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    (title, tags)
}

fn page_from_path(root: &Path, full: &Path, book: Option<&str>) -> Option<Page> {
    let rel = rel_string(root, full)?;
    let body = fs::read_to_string(full).ok().unwrap_or_default();
    let (fm_title, tags) = extract_frontmatter(&body);
    let stem = full
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    let title = fm_title.unwrap_or(stem);
    Some(Page {
        rel_path: rel,
        title,
        book: book.map(|s| s.to_string()),
        tags,
        modified: read_modified(full),
    })
}

/// Full data the indexer needs for one page.
#[derive(Debug, Clone)]
pub struct PageData {
    pub rel_path: String,
    pub title: String,
    pub book: Option<String>,
    pub body: String,
    pub tags: Vec<String>,
    pub modified: i64,
}

/// Read everything we need to index for the file at `abs`. Returns None for
/// non-markdown / non-vault paths. Used by the watcher.
pub fn read_page_data(vault: &Vault, abs: &Path) -> Option<PageData> {
    if !abs.is_file() || !is_md(abs) || is_hidden(abs) {
        return None;
    }
    let canonical = abs.canonicalize().ok()?;
    if !canonical.starts_with(&vault.root) {
        return None;
    }
    let rel_path = rel_string(&vault.root, &canonical)?;
    let body = fs::read_to_string(&canonical).ok()?;
    let (fm_title, tags) = extract_frontmatter(&body);
    let stem = canonical
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    let title = fm_title.unwrap_or(stem);
    let book = canonical
        .strip_prefix(&vault.root)
        .ok()
        .and_then(|p| p.parent())
        .and_then(|p| {
            let s = p.to_string_lossy().replace('\\', "/");
            if s.is_empty() {
                None
            } else {
                Some(s.split('/').next().unwrap_or("").to_string())
            }
        })
        .filter(|s| !s.is_empty());
    Some(PageData {
        rel_path,
        title,
        book,
        body,
        tags,
        modified: read_modified(&canonical),
    })
}

/// Walk the vault and yield PageData for every markdown file.
pub fn walk_pages(vault: &Vault) -> Vec<PageData> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(&vault.root)
        .max_depth(2)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.path()))
        .flatten()
    {
        let path = entry.path();
        if let Some(data) = read_page_data(vault, path) {
            out.push(data);
        }
    }
    out
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(true)
}

fn is_md(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("md") || s.eq_ignore_ascii_case("markdown"))
}

pub fn list_books(vault: &Vault) -> Result<Vec<Book>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(&vault.root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() || is_hidden(&path) {
            continue;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let rel_path = name.clone();
        let page_count = fs::read_dir(&path)
            .map(|rd| {
                rd.flatten()
                    .filter(|e| {
                        let p = e.path();
                        p.is_file() && !is_hidden(&p) && is_md(&p)
                    })
                    .count()
            })
            .unwrap_or(0);
        out.push(Book {
            name,
            rel_path,
            page_count,
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn list_loose_pages(vault: &Vault) -> Result<Vec<Page>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(&vault.root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || is_hidden(&path) || !is_md(&path) {
            continue;
        }
        if let Some(p) = page_from_path(&vault.root, &path, None) {
            out.push(p);
        }
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

pub fn list_pages_in_book(vault: &Vault, book: &str) -> Result<Vec<Page>> {
    let book_dir = vault.root.join(book);
    if !book_dir.is_dir() {
        anyhow::bail!("book not found: {}", book);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&book_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || is_hidden(&path) || !is_md(&path) {
            continue;
        }
        if let Some(p) = page_from_path(&vault.root, &path, Some(book)) {
            out.push(p);
        }
    }
    out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(out)
}

pub fn read_page_body(vault: &Vault, rel_path: &str) -> Result<String> {
    let full = vault.root.join(rel_path);
    let canonical = full
        .canonicalize()
        .with_context(|| format!("page not found: {}", rel_path))?;
    if !canonical.starts_with(&vault.root) {
        anyhow::bail!("path escapes vault root: {}", rel_path);
    }
    Ok(fs::read_to_string(canonical)?)
}

pub fn write_page_body(vault: &Vault, rel_path: &str, body: &str) -> Result<()> {
    let full = vault.root.join(rel_path);
    let parent = full
        .parent()
        .with_context(|| format!("page path has no parent: {}", rel_path))?;
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }
    let canonical_parent = parent.canonicalize()?;
    if !canonical_parent.starts_with(&vault.root) {
        anyhow::bail!("path escapes vault root: {}", rel_path);
    }
    fs::write(&full, body)?;
    Ok(())
}
