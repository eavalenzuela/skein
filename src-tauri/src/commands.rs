use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime, State};

use crate::settings::{self, Settings};
use crate::state::AppState;
use crate::vault::{self, Book, Page, Vault};
use crate::watcher;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn install_vault<R: Runtime>(
    app: &AppHandle<R>,
    state: &State<'_, AppState>,
    vault: Vault,
) -> Result<Vault, String> {
    let watcher = watcher::spawn(app.clone(), &vault.root).map_err(err)?;
    let mut settings = settings::load(app);
    settings.vault_path = Some(vault.root.clone());
    settings::save(app, &settings).map_err(err)?;
    state.set_vault(vault.clone(), watcher);
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
    state.clear_vault();
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
    vault::write_page_body(&vault, &rel_path, &body).map_err(err)
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
    let Ok(watcher) = watcher::spawn(app.clone(), &vault.root) else {
        return;
    };
    let state = app.state::<AppState>();
    state.set_vault(vault, watcher);
}
