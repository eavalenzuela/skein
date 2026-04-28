use std::sync::Arc;

use parking_lot::Mutex;

use crate::index::Index;
use crate::vault::Vault;
use crate::watcher::DebouncerHandle;

#[derive(Default)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
    pub index: Arc<Mutex<Option<Index>>>,
}

#[derive(Default)]
struct Inner {
    vault: Option<Vault>,
    watcher: Option<DebouncerHandle>,
}

impl AppState {
    pub fn vault(&self) -> Option<Vault> {
        self.inner.lock().vault.clone()
    }

    pub fn install(&self, vault: Vault, watcher: DebouncerHandle) {
        let mut inner = self.inner.lock();
        // Drop the previous watcher (and its thread) before installing the new one.
        inner.watcher = None;
        inner.vault = Some(vault);
        inner.watcher = Some(watcher);
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.watcher = None;
        inner.vault = None;
    }
}
