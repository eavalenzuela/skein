use std::sync::Arc;

use parking_lot::Mutex;

use crate::embedder::{HashBagEmbedder, SharedEmbedder};
use crate::index::Index;
use crate::vault::Vault;
use crate::watcher::DebouncerHandle;

pub struct AppState {
    inner: Arc<Mutex<Inner>>,
    pub index: Arc<Mutex<Option<Index>>>,
    pub embedder: Arc<Mutex<SharedEmbedder>>,
}

#[derive(Default)]
struct Inner {
    vault: Option<Vault>,
    watcher: Option<DebouncerHandle>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            index: Arc::new(Mutex::new(None)),
            embedder: Arc::new(Mutex::new(Arc::new(HashBagEmbedder::new()))),
        }
    }
}

impl AppState {
    pub fn vault(&self) -> Option<Vault> {
        self.inner.lock().vault.clone()
    }

    pub fn install(&self, vault: Vault, watcher: DebouncerHandle) {
        let mut inner = self.inner.lock();
        inner.watcher = None;
        inner.vault = Some(vault);
        inner.watcher = Some(watcher);
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.watcher = None;
        inner.vault = None;
    }

    pub fn current_embedder(&self) -> SharedEmbedder {
        self.embedder.lock().clone()
    }

    pub fn set_embedder(&self, embedder: SharedEmbedder) {
        *self.embedder.lock() = embedder;
    }
}
