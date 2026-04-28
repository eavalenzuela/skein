mod commands;
mod settings;
mod state;
mod vault;
mod watcher;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            commands::restore_last_vault(&app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::current_vault,
            commands::open_vault,
            commands::close_vault,
            commands::list_books,
            commands::list_loose_pages,
            commands::list_pages_in_book,
            commands::read_page,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
