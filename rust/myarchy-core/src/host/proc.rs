use std::io;
use std::process::{Command, Output, Stdio};

use crate::core::error::UserError;
use anyhow::{Result, bail};

/// A tool that is not installed is the one spawn failure a person can do something about.
fn spawn_failed(program: &str, args: &[&str], err: io::Error) -> anyhow::Error {
    let missing = err.kind() == io::ErrorKind::NotFound;
    let err =
        anyhow::Error::new(err).context(format!("failed to run {program} {}", args.join(" ")));
    if missing {
        return err.context(UserError::ToolMissing {
            tool: program.to_string(),
        });
    }
    err
}

pub fn output(program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .output()
        .map_err(|e| spawn_failed(program, args, e))
}

pub fn run(program: &str, args: &[&str]) -> Result<String> {
    let out = output(program, args)?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if !out.status.success() {
        bail!("{program} {} failed: {}", args.join(" "), detail(&out));
    }
    Ok(stdout)
}

pub fn ok(program: &str, args: &[&str]) -> bool {
    output(program, args).is_ok_and(|out| out.status.success())
}

/// Leaves the child's stdio attached, so whatever it prints reaches the terminal.
pub fn status(program: &str, args: &[&str]) -> Result<bool> {
    Ok(Command::new(program)
        .args(args)
        .status()
        .map_err(|e| spawn_failed(program, args, e))?
        .success())
}

pub fn spawn_detached(program: &str, args: &[&str]) -> Result<()> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| spawn_failed(program, args, e))?;
    Ok(())
}

/// Some of these tools report their failure on stdout and leave stderr empty.
pub fn detail(out: &Output) -> String {
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if stderr.is_empty() {
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else {
        stderr
    }
}
