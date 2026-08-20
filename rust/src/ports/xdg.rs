use std::env;
use std::path::PathBuf;

fn home() -> PathBuf {
    PathBuf::from(env::var_os("HOME").expect("HOME is not set"))
}

pub(crate) fn state_dir() -> PathBuf {
    let base = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"));
    base.join("myarchy")
}

pub(crate) fn runtime_dir() -> PathBuf {
    let base = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("myarchy")
}
