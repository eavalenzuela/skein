mod archive;
mod attachments;
mod autotag;
mod books;
mod chat;
mod chunker;
mod commands;
mod daily;
mod embedder;
mod git_sync;
mod index;
mod pages;
mod secrets;
mod settings;
mod state;
mod vault;
mod watcher;

use std::io::Write;
use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use state::AppState;

fn install_panic_logger(app_data_dir: std::path::PathBuf) {
    // Append panics to a per-user log so a crashing build leaves a trail
    // the user can attach to a bug report. Best-effort — if we can't write
    // the file we fall back to the default panic behavior.
    let _ = std::fs::create_dir_all(&app_data_dir);
    let log_path = app_data_dir.join("skein.log");
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%z");
            let _ = writeln!(f, "[{}] PANIC {}", now, info);
        }
        prev(info);
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            if let Ok(dir) = handle.path().app_data_dir() {
                install_panic_logger(dir);
            }
            commands::try_load_local_embedding_model(&handle);
            commands::restore_last_vault(&handle);
            // Daily-note reminder loop runs for the lifetime of the app.
            let reminder_state = Arc::new(Mutex::new(daily::ReminderState::default()));
            daily::spawn_reminder(handle.clone(), reminder_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::current_vault,
            commands::open_vault,
            commands::close_vault,
            commands::list_books,
            commands::create_book,
            commands::rename_book,
            commands::delete_book,
            commands::set_book_order,
            commands::create_page,
            commands::rename_page,
            commands::delete_page_command,
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
            commands::dismiss_tag,
            commands::open_today_daily,
            commands::save_attachment,
            commands::save_attachment_from_path,
            commands::export_vault,
            commands::open_vault_from_archive,
            commands::git_status,
            commands::git_set_remote,
            commands::git_pull,
            commands::git_push,
            commands::git_commit_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
