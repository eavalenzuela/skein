// Phase 10 — daily notes + OS reminders.
//
// Daily notes live in a configurable book ("Daily" by default) with one
// page per day named YYYY-MM-DD.md. The first time you open today's
// daily, we render the configured template into it; subsequent opens
// just hand back the existing path.
//
// The reminder thread polls once per minute and fires a native OS
// notification (via tauri-plugin-notification) when the configured
// reminder time matches the current local time. Duplicate firings
// inside the same calendar day are suppressed.

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::settings;
use crate::vault::Vault;

#[cfg(test)]
use chrono::TimeZone;

pub const DEFAULT_BOOK: &str = "Daily";
pub const DEFAULT_TEMPLATE: &str = "\
---
title: {{long_date}}
tags: [daily]
created: {{date}}
---

## Morning


## Notes


## Tomorrow

";

#[derive(Debug, Clone, Serialize)]
pub struct DailyResult {
    pub rel_path: String,
    pub created: bool,
}

pub fn today_local() -> DateTime<Local> {
    Local::now()
}

pub fn render_template(template: &str, now: &DateTime<Local>) -> String {
    template
        .replace("{{date}}", &now.format("%Y-%m-%d").to_string())
        .replace("{{weekday}}", &now.format("%A").to_string())
        .replace("{{long_date}}", &format_long_date(now))
        .replace("{{time}}", &now.format("%H:%M").to_string())
}

fn format_long_date(now: &DateTime<Local>) -> String {
    // chrono doesn't ship a non-padded day format on every platform via
    // strftime, so build it manually for portability.
    let weekday = now.format("%A");
    let day = now.day();
    let month = now.format("%B");
    let year = now.year();
    format!("{}, {} {} {}", weekday, day, month, year)
}

fn today_rel_path(book: &str, now: &DateTime<Local>) -> String {
    let date = now.format("%Y-%m-%d");
    format!("{}/{}.md", book, date)
}

/// Open (creating if necessary) today's daily note. Returns the rel_path
/// and whether it was just created.
pub fn ensure_today(
    vault: &Vault,
    book: Option<&str>,
    template: Option<&str>,
) -> Result<DailyResult> {
    let book = book.unwrap_or(DEFAULT_BOOK);
    let template = template.unwrap_or(DEFAULT_TEMPLATE);
    let now = today_local();
    let rel = today_rel_path(book, &now);
    let abs: PathBuf = vault.root.join(&rel);
    if abs.exists() {
        return Ok(DailyResult {
            rel_path: rel,
            created: false,
        });
    }
    if let Some(parent) = abs.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
        // Path-traversal guard.
        let parent_canonical = parent
            .canonicalize()
            .with_context(|| format!("canonicalizing {}", parent.display()))?;
        if !parent_canonical.starts_with(&vault.root) {
            return Err(anyhow!("path escapes vault: {}", rel));
        }
    }
    let body = render_template(template, &now);
    std::fs::write(&abs, body)?;
    Ok(DailyResult {
        rel_path: rel,
        created: true,
    })
}

#[derive(Default)]
pub struct ReminderState {
    last_fired: Option<NaiveDate>,
}

pub fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    let (h, m) = s.split_once(':')?;
    let h: u32 = h.parse().ok()?;
    let m: u32 = m.parse().ok()?;
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}

/// Background thread that fires the configured daily reminder once per
/// day. Reads settings on every tick so changes take effect without a
/// restart.
pub fn spawn_reminder<R: Runtime>(
    app: AppHandle<R>,
    state: Arc<Mutex<ReminderState>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let now = today_local();
        let s = settings::load(&app);
        if let Some(time_str) = &s.daily_reminder_time {
            if let Some((h, m)) = parse_hhmm(time_str) {
                if now.time().hour() == h && now.time().minute() == m {
                    let today = now.date_naive();
                    let mut guard = state.lock();
                    if guard.last_fired != Some(today) {
                        guard.last_fired = Some(today);
                        drop(guard);
                        let _ = app
                            .notification()
                            .builder()
                            .title("Skein — daily note")
                            .body("Time to write today's entry.")
                            .show();
                    }
                }
            }
        }
        // 30s tick. The minute-aligned check above means we'll catch the
        // reminder window in any 60s span.
        thread::sleep(Duration::from_secs(30));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_template_placeholders() {
        let dt = chrono::Local
            .with_ymd_and_hms(2026, 4, 21, 9, 30, 0)
            .single()
            .unwrap();
        let out = render_template("{{long_date}} | {{date}} | {{weekday}} | {{time}}", &dt);
        assert!(out.contains("2026-04-21"));
        assert!(out.contains("Tuesday"));
        assert!(out.contains("09:30"));
    }

    #[test]
    fn parses_hhmm_basic() {
        assert_eq!(parse_hhmm("09:30"), Some((9, 30)));
        assert_eq!(parse_hhmm("23:59"), Some((23, 59)));
        assert_eq!(parse_hhmm("24:00"), None);
        assert_eq!(parse_hhmm("9-30"), None);
    }
}
