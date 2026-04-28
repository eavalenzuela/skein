use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime, State};

use crate::index::{self, Index, RelatedHit, SearchHit};
use crate::settings::{self, Settings};
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
    Index::open(&path, state.embedder.clone()).map_err(err)
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
