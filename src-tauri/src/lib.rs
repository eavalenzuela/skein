mod autotag;
mod chat;
mod chunker;
mod commands;
mod embedder;
mod index;
mod secrets;
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
            let handle = app.handle().clone();
            commands::try_load_local_embedding_model(&handle);
            commands::restore_last_vault(&handle);
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
            commands::write_page,
            commands::search_pages,
            commands::find_related,
            commands::list_page_titles,
            commands::find_backlinks,
            commands::rebuild_index,
            commands::embedding_model_status,
            commands::download_embedding_model,
            commands::get_settings,
            commands::set_settings,
            commands::has_secret,
            commands::set_secret,
            commands::clear_secret,
            commands::chat_send,
            commands::suggest_tags,
            commands::apply_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
