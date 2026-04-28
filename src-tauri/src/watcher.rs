use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use tauri::{AppHandle, Emitter, Runtime};

pub type DebouncerHandle = Debouncer<RecommendedWatcher, RecommendedCache>;

pub fn spawn<R: Runtime>(app: AppHandle<R>, root: &Path) -> Result<DebouncerHandle> {
    let app_for_callback = app.clone();
    let mut debouncer: DebouncerHandle = new_debouncer(
        Duration::from_millis(500),
        None,
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                if !events.is_empty() {
                    let _ = app_for_callback.emit("vault-changed", ());
                }
            }
        },
    )?;
    debouncer.watch(root, RecursiveMode::Recursive)?;
    Ok(debouncer)
}
