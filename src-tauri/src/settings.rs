use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Settings {
    pub vault_path: Option<PathBuf>,
    pub theme: Option<String>,
    pub shelf_style: Option<String>,
    pub sidebar: Option<String>,
    pub page_font: Option<String>,
    pub daily_book: Option<String>,
    pub daily_template: Option<String>,
    pub daily_reminder_time: Option<String>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct SettingsPatch {
    pub theme: Option<String>,
    pub shelf_style: Option<String>,
    pub sidebar: Option<String>,
    pub page_font: Option<String>,
    pub daily_book: Option<String>,
    pub daily_template: Option<String>,
    pub daily_reminder_time: Option<String>,
}

impl Settings {
    pub fn apply(&mut self, patch: SettingsPatch) {
        if let Some(v) = patch.theme {
            self.theme = Some(v);
        }
        if let Some(v) = patch.shelf_style {
            self.shelf_style = Some(v);
        }
        if let Some(v) = patch.sidebar {
            self.sidebar = Some(v);
        }
        if let Some(v) = patch.page_font {
            self.page_font = Some(v);
        }
        if let Some(v) = patch.daily_book {
            self.daily_book = Some(v);
        }
        if let Some(v) = patch.daily_template {
            self.daily_template = Some(v);
        }
        if let Some(v) = patch.daily_reminder_time {
            // Empty string clears it.
            self.daily_reminder_time = if v.is_empty() { None } else { Some(v) };
        }
    }
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
