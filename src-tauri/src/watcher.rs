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
        } else if let Some(rel) = rel_path_of(vault, path) {
            // File no longer exists at this path — remove from the index.
            let _ = idx.delete_page(&rel);
        }
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
