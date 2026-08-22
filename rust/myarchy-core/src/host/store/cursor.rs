use std::fs;
use std::path::PathBuf;

use crate::core::store::{CursorStore, PreferredCursor};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::host::{file, xdg};

pub struct CursorFiles;

#[derive(Serialize, Deserialize)]
struct Stored {
    name: String,
    size: i64,
}

fn preferred_file() -> PathBuf {
    xdg::state_dir().join("cursor/current.json")
}

fn theme_roots() -> Vec<PathBuf> {
    let home = xdg::home();
    vec![
        PathBuf::from("/usr/share/icons"),
        home.join(".local/share/icons"),
        home.join(".icons"),
    ]
}

impl CursorStore for CursorFiles {
    fn list(&self) -> Result<Vec<String>> {
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

    fn load_preferred(&self) -> Option<PreferredCursor> {
        let text = fs::read_to_string(preferred_file()).ok()?;
        let stored: Stored = serde_json::from_str(&text).ok()?;
        Some(PreferredCursor {
            name: stored.name,
            size: stored.size,
        })
    }

    fn save_preferred(&self, name: &str, size: i64) -> Result<()> {
        let stored = Stored {
            name: name.to_string(),
            size,
        };
        file::write_atomic(&preferred_file(), &serde_json::to_string_pretty(&stored)?)
    }

    fn forget_preferred(&self) {
        let _ = fs::remove_file(preferred_file());
    }
}
