use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::backend::theme;
use crate::backend::wallpaper::Wallpaperctl;
use crate::backend::xdg;
use crate::error::AppError;

pub(crate) fn dir() -> PathBuf {
    xdg::myarchy_dir().join("wallpapers")
}

const CURRENT_LINK_NAME: &str = "current";
fn current_link() -> PathBuf {
    dir().join(CURRENT_LINK_NAME)
}

fn state_file() -> PathBuf {
    xdg::state_dir().join("wallpaper").join(CURRENT_LINK_NAME)
}

fn load_preferred() -> Option<String> {
    let text = fs::read_to_string(state_file()).ok()?;
    let name = text.trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn write_preferred(name: &str) -> Result<()> {
    let file = state_file();
    fs::create_dir_all(file.parent().unwrap())?;
    let tmp = PathBuf::from(format!("{}.tmp{}", file.display(), std::process::id()));
    fs::write(&tmp, name).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, &file).with_context(|| format!("failed to replace {}", file.display()))
}

pub(crate) fn list() -> Result<Vec<String>> {
    let mut names: Vec<String> = fs::read_dir(dir())
        .context("failed to read wallpapers directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|t| !t.is_dir()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != CURRENT_LINK_NAME && !name.starts_with('.'))
        .collect();
    names.sort();
    Ok(names)
}

pub(crate) fn get() -> Result<Option<String>> {
    match fs::read_link(current_link()) {
        Ok(target) => Ok(target.file_name().map(|n| n.to_string_lossy().into_owned())),
        Err(_) => Ok(None),
    }
}

fn apply(wallpaper: &dyn Wallpaperctl, name: &str) -> Result<()> {
    let path = dir().join(name);
    if !path.is_file() {
        bail!(AppError::UnknownWallpaper {
            name: name.to_string()
        });
    }

    let link = current_link();
    let tmp_link = PathBuf::from(format!("{}.tmp{}", link.display(), std::process::id()));
    let _ = fs::remove_file(&tmp_link);
    std::os::unix::fs::symlink(&path, &tmp_link)
        .with_context(|| format!("failed to symlink {}", tmp_link.display()))?;
    fs::rename(&tmp_link, &link)
        .with_context(|| format!("failed to replace {}", link.display()))?;

    // hyprpaper.conf preloads this symlink on its own startup, before myarchyctl ever runs; the
    // path passed below only tells an already-running hyprpaper to switch live over IPC.
    wallpaper.apply(&path)
}

pub(crate) fn set(wallpaper: &dyn Wallpaperctl, name: &str) -> Result<()> {
    apply(wallpaper, name)?;
    save_preferred()
}

pub(crate) fn init(wallpaper: &dyn Wallpaperctl) -> Result<()> {
    if current_link().is_file() {
        return Ok(());
    }
    let name = list()?.into_iter().next().context("no wallpapers found")?;
    set(wallpaper, &name)
}

pub(crate) fn apply_preferred(wallpaper: &dyn Wallpaperctl) -> Result<()> {
    let name = match load_preferred() {
        Some(name) => name,
        None => theme::get_var("wallpaper")?.ok_or(AppError::NoPreferredWallpaper)?,
    };
    apply(wallpaper, &name)
}

pub(crate) fn save_preferred() -> Result<()> {
    let name = get()?.context("no current wallpaper set")?;
    write_preferred(&name)?;
    theme::render()
}

pub(crate) fn reset(wallpaper: &dyn Wallpaperctl) -> Result<()> {
    let _ = fs::remove_file(state_file());
    apply_preferred(wallpaper)?;
    theme::render()
}
