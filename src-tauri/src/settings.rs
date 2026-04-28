use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Settings {
    pub vault_path: Option<PathBuf>,
}

fn settings_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    let dir = app.path().app_config_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir.join("settings.json"))
}

pub fn load<R: Runtime>(app: &AppHandle<R>) -> Settings {
    let Ok(path) = settings_path(app) else {
        return Settings::default();
    };
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<()> {
    let path = settings_path(app)?;
    fs::write(path, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}
