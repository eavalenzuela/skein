use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::autotag;
use crate::chat::{self, ChatMessageIn};
use crate::embedder::{self, OnnxBgeEmbedder, SharedEmbedder};
use crate::index::{self, BacklinkHit, Index, PageTitle, RelatedHit, SearchHit};
use crate::secrets;
use crate::settings::{self, Settings, SettingsPatch};
use crate::state::AppState;
use crate::vault::{self, Book, Page, Vault};
use crate::watcher;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn open_or_init_index<R: Runtime>(
    app: &AppHandle<R>,
    state: &State<'_, AppState>,
) -> Result<Index, String> {
    let dir = app.path().app_data_dir().map_err(err)?;
    let path = index::db_path(&dir);
    Index::open(&path, state.current_embedder()).map_err(err)
}

fn rebuild_index_for(idx: &mut Index, vault: &Vault) -> Result<(), String> {
    idx.delete_all().map_err(err)?;
    for page in vault::walk_pages(vault) {
        idx.upsert_page(&page).map_err(err)?;
    }
    Ok(())
}

fn install_vault<R: Runtime>(
    app: &AppHandle<R>,
    state: &State<'_, AppState>,
    vault: Vault,
) -> Result<Vault, String> {
    // Make sure the index DB is open.
    {
        let mut idx_slot = state.index.lock();
        if idx_slot.is_none() {
            *idx_slot = Some(open_or_init_index(app, state)?);
        }
        let idx = idx_slot
            .as_mut()
            .ok_or_else(|| "index not initialized".to_string())?;
        rebuild_index_for(idx, &vault)?;
    }

    let watcher = watcher::spawn(app.clone(), vault.clone(), state.index.clone()).map_err(err)?;

    let mut settings = settings::load(app);
    settings.vault_path = Some(vault.root.clone());
    settings::save(app, &settings).map_err(err)?;

    state.install(vault.clone(), watcher);
    Ok(vault)
}

#[tauri::command]
pub fn current_vault(state: State<'_, AppState>) -> Option<Vault> {
    state.vault()
}

#[tauri::command]
pub fn open_vault<R: Runtime>(
    path: String,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<Vault, String> {
    let vault = Vault::from_path(PathBuf::from(path)).map_err(err)?;
    install_vault(&app, &state, vault)
}

#[tauri::command]
pub fn close_vault<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.clear();
    {
        let mut idx_slot = state.index.lock();
        if let Some(idx) = idx_slot.as_mut() {
            let _ = idx.delete_all();
        }
    }
    let mut settings = settings::load(&app);
    settings.vault_path = None;
    settings::save(&app, &settings).map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn list_books(state: State<'_, AppState>) -> Result<Vec<Book>, String> {
    let vault = state.vault().ok_or("no vault open")?;
    vault::list_books(&vault).map_err(err)
}

#[tauri::command]
pub fn list_loose_pages(state: State<'_, AppState>) -> Result<Vec<Page>, String> {
    let vault = state.vault().ok_or("no vault open")?;
    vault::list_loose_pages(&vault).map_err(err)
}

#[tauri::command]
pub fn list_pages_in_book(book: String, state: State<'_, AppState>) -> Result<Vec<Page>, String> {
    let vault = state.vault().ok_or("no vault open")?;
    vault::list_pages_in_book(&vault, &book).map_err(err)
}

#[tauri::command]
pub fn read_page(rel_path: String, state: State<'_, AppState>) -> Result<String, String> {
    let vault = state.vault().ok_or("no vault open")?;
    vault::read_page_body(&vault, &rel_path).map_err(err)
}

#[tauri::command]
pub fn write_page(
    rel_path: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let vault = state.vault().ok_or("no vault open")?;
    vault::write_page_body(&vault, &rel_path, &body).map_err(err)?;
    // Update the index immediately rather than waiting for the watcher
    // to fire — keeps search and "related notes" tight after a save.
    if let Some(data) = vault::read_page_data(&vault, &vault.root.join(&rel_path)) {
        if let Some(idx) = state.index.lock().as_mut() {
            let _ = idx.upsert_page(&data);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn search_pages(
    query: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchHit>, String> {
    let mut idx_slot = state.index.lock();
    let idx = idx_slot
        .as_mut()
        .ok_or_else(|| "index not initialized".to_string())?;
    let limit = limit.unwrap_or(20).min(200) as usize;
    idx.search(&query, limit).map_err(err)
}

#[tauri::command]
pub fn find_related(
    rel_path: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<RelatedHit>, String> {
    let mut idx_slot = state.index.lock();
    let idx = idx_slot
        .as_mut()
        .ok_or_else(|| "index not initialized".to_string())?;
    let limit = limit.unwrap_or(8).min(50) as usize;
    idx.find_related(&rel_path, limit).map_err(err)
}

#[tauri::command]
pub fn list_page_titles(state: State<'_, AppState>) -> Result<Vec<PageTitle>, String> {
    let mut idx_slot = state.index.lock();
    let idx = idx_slot
        .as_mut()
        .ok_or_else(|| "index not initialized".to_string())?;
    idx.list_page_titles().map_err(err)
}

#[tauri::command]
pub fn find_backlinks(
    rel_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<BacklinkHit>, String> {
    let mut idx_slot = state.index.lock();
    let idx = idx_slot
        .as_mut()
        .ok_or_else(|| "index not initialized".to_string())?;
    idx.find_backlinks(&rel_path).map_err(err)
}

#[tauri::command]
pub fn rebuild_index(state: State<'_, AppState>) -> Result<u32, String> {
    let vault = state.vault().ok_or("no vault open")?;
    let mut idx_slot = state.index.lock();
    let idx = idx_slot
        .as_mut()
        .ok_or_else(|| "index not initialized".to_string())?;
    rebuild_index_for(idx, &vault)?;
    Ok(vault::walk_pages(&vault).len() as u32)
}

/// Re-open the vault recorded in settings, if any.
pub fn restore_last_vault<R: Runtime>(app: &AppHandle<R>) {
    let settings: Settings = settings::load(app);
    let Some(path) = settings.vault_path else {
        return;
    };
    let Ok(vault) = Vault::from_path(path) else {
        return;
    };
    let state = app.state::<AppState>();
    let _ = install_vault(app, &state, vault);
}

/// At startup, if the BGE marker exists in the cache dir, try to load the
/// model and install it as the active embedder before any indexing happens.
/// On failure we silently fall back to HashBagEmbedder.
pub fn try_load_local_embedding_model<R: Runtime>(app: &AppHandle<R>) {
    let Ok(dir) = app.path().app_data_dir() else {
        return;
    };
    let cache = embedder::bge_cache_dir(&dir);
    if !cache.join(embedder::BGE_READY_MARKER).exists() {
        return;
    }
    if let Ok(model) = OnnxBgeEmbedder::load_or_download(cache) {
        let new_emb: SharedEmbedder = Arc::new(model);
        let state = app.state::<AppState>();
        state.set_embedder(new_emb);
    }
}

#[derive(Serialize)]
pub struct EmbeddingModelStatus {
    pub name: String,
    pub local: bool, // true = real on-device model (e.g. BGE), false = fallback hash-bag
}

#[tauri::command]
pub fn embedding_model_status(state: State<'_, AppState>) -> EmbeddingModelStatus {
    let emb = state.current_embedder();
    let name = emb.name().to_string();
    EmbeddingModelStatus {
        local: name == "bge-small-en-v1.5",
        name,
    }
}

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> Settings {
    settings::load(&app)
}

#[tauri::command]
pub fn set_settings<R: Runtime>(
    patch: SettingsPatch,
    app: AppHandle<R>,
) -> Result<Settings, String> {
    let mut s = settings::load(&app);
    s.apply(patch);
    settings::save(&app, &s).map_err(err)?;
    Ok(s)
}

#[tauri::command]
pub fn has_secret(name: String) -> bool {
    secrets::has(&name)
}

#[tauri::command]
pub fn set_secret(name: String, value: String) -> Result<(), String> {
    secrets::set(&name, &value).map_err(err)
}

#[tauri::command]
pub fn clear_secret(name: String) -> Result<(), String> {
    secrets::clear(&name).map_err(err)
}

#[tauri::command]
pub async fn suggest_tags(
    rel_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let api_key =
        secrets::read("anthropic_api_key").ok_or_else(|| "no Anthropic API key set".to_string())?;
    let vault = state.vault().ok_or("no vault open")?;
    let body = vault::read_page_body(&vault, &rel_path).map_err(err)?;
    let title = vault::read_page_data(&vault, &vault.root.join(&rel_path))
        .map(|d| d.title)
        .unwrap_or_else(|| rel_path.clone());
    let existing = match state.index.lock().as_ref() {
        Some(idx) => idx.all_tags().unwrap_or_default(),
        None => vec![],
    };
    autotag::suggest_tags(&api_key, &title, &body, &existing)
        .await
        .map_err(err)
}

#[tauri::command]
pub fn apply_tag(rel_path: String, tag: String, state: State<'_, AppState>) -> Result<(), String> {
    let vault = state.vault().ok_or("no vault open")?;
    let body = vault::read_page_body(&vault, &rel_path).map_err(err)?;
    let new_body = autotag::add_tag_to_body(&body, &tag).map_err(err)?;
    vault::write_page_body(&vault, &rel_path, &new_body).map_err(err)?;
    if let Some(data) = vault::read_page_data(&vault, &vault.root.join(&rel_path)) {
        if let Some(idx) = state.index.lock().as_mut() {
            let _ = idx.upsert_page(&data);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn chat_send<R: Runtime + 'static>(
    messages: Vec<ChatMessageIn>,
    model: String,
    context_mode: String,
    current_rel_path: Option<String>,
    max_tokens: Option<u32>,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let api_key =
        secrets::read("anthropic_api_key").ok_or_else(|| "no Anthropic API key set".to_string())?;
    let vault = state.vault();
    let prepared = chat::prepare_request(
        &messages,
        &model,
        max_tokens.unwrap_or(4096),
        &context_mode,
        current_rel_path.as_deref(),
        vault.as_ref(),
        &state.index,
    )
    .map_err(err)?;

    let turn_id = chat::spawn_chat(app, api_key, prepared.body, prepared.context);
    Ok(turn_id)
}

#[tauri::command]
pub async fn download_embedding_model<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<EmbeddingModelStatus, String> {
    let dir = app.path().app_data_dir().map_err(err)?;
    let cache = embedder::bge_cache_dir(&dir);

    let _ = app.emit(
        "embedding-model",
        &serde_json::json!({"state": "downloading"}),
    );

    // Loading + download is blocking; run it on a worker thread.
    let cache_for_task = cache.clone();
    let onnx =
        tokio::task::spawn_blocking(move || OnnxBgeEmbedder::load_or_download(cache_for_task))
            .await
            .map_err(err)?
            .map_err(err)?;

    let new_emb: SharedEmbedder = Arc::new(onnx);
    state.set_embedder(new_emb.clone());

    // Reopen the index against the new embedder, then rebuild from disk so
    // every page gets re-embedded under the new model name.
    let path = index::db_path(&dir);
    let new_idx = Index::open(&path, new_emb).map_err(err)?;
    {
        let mut slot = state.index.lock();
        *slot = Some(new_idx);
        if let Some(vault) = state.vault() {
            if let Some(idx) = slot.as_mut() {
                rebuild_index_for(idx, &vault)?;
            }
        }
    }

    let status = embedding_model_status(state);
    let _ = app.emit(
        "embedding-model",
        &serde_json::json!({"state": "ready", "name": status.name}),
    );
    let _ = app.emit("vault-changed", ());
    Ok(status)
}
