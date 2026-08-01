use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Runtime};

use crate::index::Index;
use crate::vault::{self, Vault};

pub type DebouncerHandle = Debouncer<RecommendedWatcher, RecommendedCache>;

fn rel_path_of(vault: &Vault, abs: &Path) -> Option<String> {
    abs.strip_prefix(&vault.root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('\\', "/"))
}

fn handle_paths(vault: &Vault, index: &Mutex<Option<Index>>, paths: &[PathBuf]) {
    let mut guard = index.lock();
    let Some(idx) = guard.as_mut() else { return };
    for path in paths {
        if path.is_file() {
            if let Some(data) = vault::read_page_data(vault, path) {
                let _ = idx.upsert_page(&data);
            }
            continue;
        }
        // Only a path that is *gone* means a deletion. A directory is also
        // "not a file", and writing any page inside a book touches its
        // folder — so testing `!is_file()` swept the whole book out of the
        // index on every edit, leaving search silently missing those pages
        // until a manual rebuild.
        if path.exists() {
            continue;
        }
        let Some(rel) = rel_path_of(vault, path) else {
            continue;
        };
        let _ = idx.delete_page(&rel);
        // A vanished directory (book folder deleted or renamed outside the
        // app) only reports the folder path, but takes every page beneath
        // it. Anything without a markdown extension might have been a
        // folder, so sweep the prefix too — for a plain file the pattern
        // matches nothing and the sweep is a no-op.
        let lower = rel.to_lowercase();
        if !lower.ends_with(".md") && !lower.ends_with(".markdown") {
            let _ = idx.delete_pages_with_prefix(&format!("{}/", rel));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Index;
    use crate::vault::PageData;

    fn page(rel: &str, book: Option<&str>) -> PageData {
        PageData {
            rel_path: rel.to_string(),
            title: rel.to_string(),
            book: book.map(str::to_string),
            body: "body text".to_string(),
            tags: vec![],
            modified: 0,
        }
    }

    fn setup() -> (tempfile::TempDir, Vault, Mutex<Option<Index>>) {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("vault");
        std::fs::create_dir_all(root.join("Research")).unwrap();
        std::fs::write(root.join("Research/a.md"), "---\ntitle: A\n---\n\nbody\n").unwrap();
        let vault = Vault::from_path(root).unwrap();
        let embedder: crate::embedder::SharedEmbedder =
            Arc::new(crate::embedder::HashBagEmbedder::new());
        let mut idx = Index::open(&tmp.path().join("i.db"), embedder).unwrap();
        idx.upsert_page(&page("Research/a.md", Some("Research"))).unwrap();
        (tmp, vault, Mutex::new(Some(idx)))
    }

    fn indexed(index: &Mutex<Option<Index>>) -> Vec<String> {
        let guard = index.lock();
        let idx = guard.as_ref().unwrap();
        idx.list_page_titles()
            .unwrap()
            .into_iter()
            .map(|t| t.rel_path)
            .collect()
    }

    #[test]
    fn an_event_on_a_live_book_folder_keeps_its_pages() {
        // Writing a page touches its parent directory, so the watcher sees
        // the folder path. That must not be read as "the book was deleted".
        let (_tmp, vault, index) = setup();
        handle_paths(&vault, &index, &[vault.root.join("Research")]);
        assert_eq!(indexed(&index), vec!["Research/a.md".to_string()]);
    }

    #[test]
    fn a_vanished_book_folder_still_drops_its_pages() {
        let (_tmp, vault, index) = setup();
        std::fs::remove_dir_all(vault.root.join("Research")).unwrap();
        handle_paths(&vault, &index, &[vault.root.join("Research")]);
        assert!(indexed(&index).is_empty());
    }
}

pub fn spawn<R: Runtime>(
    app: AppHandle<R>,
    vault: Vault,
    index: Arc<Mutex<Option<Index>>>,
) -> Result<DebouncerHandle> {
    let app_for_callback = app.clone();
    let vault_for_callback = vault.clone();
    let index_for_callback = index.clone();
    let mut debouncer: DebouncerHandle = new_debouncer(
        Duration::from_millis(500),
        None,
        move |res: DebounceEventResult| {
            let Ok(events) = res else { return };
            if events.is_empty() {
                return;
            }
            for ev in &events {
                handle_paths(&vault_for_callback, &index_for_callback, &ev.paths);
            }
            let _ = app_for_callback.emit("vault-changed", ());
        },
    )?;
    debouncer.watch(&vault.root, RecursiveMode::Recursive)?;
    Ok(debouncer)
}
