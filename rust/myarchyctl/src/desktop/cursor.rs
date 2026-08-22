use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::backend;
use crate::backend::theme;
use crate::backend::xdg;
use crate::error::AppError;

pub(crate) trait CursorCtl {
    fn set(&self, name: &str, size: i64) -> Result<()>;
}

#[derive(Serialize, Deserialize)]
struct PreferredCursor {
    name: String,
    size: i64,
}

fn state_file() -> PathBuf {
    xdg::state_dir().join("cursor/current.json")
}

fn load_preferred() -> Option<PreferredCursor> {
    let text = fs::read_to_string(state_file()).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_preferred(name: &str, size: i64) -> Result<()> {
    let file = state_file();
    fs::create_dir_all(file.parent().unwrap())?;
    let tmp = PathBuf::from(format!("{}.tmp{}", file.display(), std::process::id()));
    let state = PreferredCursor {
        name: name.to_string(),
        size,
    };
    fs::write(&tmp, serde_json::to_string_pretty(&state)?)
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, &file).with_context(|| format!("failed to replace {}", file.display()))
}

fn theme_roots() -> Vec<PathBuf> {
    let home = xdg::home();
    vec![
        PathBuf::from("/usr/share/icons"),
        home.join(".local/share/icons"),
        home.join(".icons"),
    ]
}

pub(crate) fn list() -> Result<Vec<String>> {
    let mut names = Vec::new();
    for root in theme_roots() {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let is_cursor_theme = entry.file_type().is_ok_and(|t| t.is_dir())
                && fs::symlink_metadata(entry.path().join("cursors")).is_ok_and(|m| m.is_dir());
            if is_cursor_theme {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

pub(crate) fn set(name: &str, size: i64) -> Result<()> {
    let applied = backend::cursorctl().set(name, size);
    save_preferred(name, size)?;
    applied
}

pub(crate) fn apply_preferred() -> Result<()> {
    let (name, size) = match load_preferred() {
        Some(state) => (state.name, state.size),
        None => {
            let name = theme::get_var("cursor-name")?.ok_or(AppError::NoPreferredCursor)?;
            let size = theme::get_var("cursor-size")?
                .context("no cursor-size found for current theme")?
                .parse::<i64>()
                .context("cursor-size in current theme is not a number")?;
            (name, size)
        }
    };
    backend::cursorctl().set(&name, size)
}

pub(crate) fn save_preferred(name: &str, size: i64) -> Result<()> {
    write_preferred(name, size)?;
    theme::render()
}

pub(crate) fn reset() -> Result<()> {
    let _ = fs::remove_file(state_file());
    apply_preferred()?;
    theme::render()
}
