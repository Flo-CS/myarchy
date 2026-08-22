use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn tmp_beside(file: &Path) -> PathBuf {
    PathBuf::from(format!("{}.tmp{}", file.display(), std::process::id()))
}

pub fn write_atomic(file: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = tmp_beside(file);
    fs::write(&tmp, contents).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, file).with_context(|| format!("failed to replace {}", file.display()))
}

pub fn symlink_atomic(target: &Path, link: &Path) -> Result<()> {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = tmp_beside(link);
    let _ = fs::remove_file(&tmp);
    std::os::unix::fs::symlink(target, &tmp)
        .with_context(|| format!("failed to symlink {}", tmp.display()))?;
    fs::rename(&tmp, link).with_context(|| format!("failed to replace {}", link.display()))
}
