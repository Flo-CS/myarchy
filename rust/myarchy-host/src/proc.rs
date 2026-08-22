use std::process::{Command, Output, Stdio};

use anyhow::{bail, Context, Result};

pub fn output(program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {program} {}", args.join(" ")))
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
        .with_context(|| format!("failed to run {program} {}", args.join(" ")))?
        .success())
}

pub fn spawn_detached(program: &str, args: &[&str]) -> Result<()> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to start {program}"))?;
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
