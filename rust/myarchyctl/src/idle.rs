use std::io::{self, Write};
use std::os::unix::net::UnixStream;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use idle_inhibitor::ipc::{self, Request, Response};

#[derive(Subcommand)]
pub enum Command {
    Enable,
    Disable,
    Status,
    Watch,
}

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Enable => enable(),
        Command::Disable => disable(),
        Command::Status => {
            let inhibited = status()?;
            println!("{}", label(inhibited));
            if inhibited {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Command::Watch => watch(),
    }
}

fn connect() -> Result<UnixStream> {
    match ipc::connect() {
        Ok(stream) => Ok(stream),
        Err(e)
            if matches!(
                e.kind(),
                io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound
            ) =>
        {
            bail!("idle-inhibitor daemon is not running; start it with `idle-inhibitord`")
        }
        Err(e) => Err(e).context("failed to connect to idle-inhibitor daemon"),
    }
}

fn oneshot(request: Request) -> Result<Response> {
    let stream = connect()?;
    ipc::send_request(&stream, &request).context("failed to send request")?;
    let response = ipc::read_response(&stream).context("failed to read response")?;
    if let Response::Err(msg) = &response {
        bail!("{msg}");
    }
    Ok(response)
}

pub fn enable() -> Result<()> {
    oneshot(Request::Enable)?;
    Ok(())
}

pub fn disable() -> Result<()> {
    oneshot(Request::Disable)?;
    Ok(())
}

pub fn status() -> Result<bool> {
    match oneshot(Request::Status)? {
        Response::Status(inhibited) => Ok(inhibited),
        _ => bail!("unexpected response from daemon"),
    }
}

pub fn watch() -> Result<()> {
    let stream = connect()?;
    ipc::send_request(&stream, &Request::Watch).context("failed to send request")?;

    let mut stdout = io::stdout().lock();
    let de = serde_json::Deserializer::from_reader(&stream);
    for response in de.into_iter::<Response>() {
        match response {
            Ok(Response::Status(inhibited)) => {
                writeln!(stdout, "{}", label(inhibited))?;
                stdout.flush()?;
            }
            Ok(Response::Err(msg)) => bail!(msg),
            Ok(_) => {}
            Err(_) => break,
        }
    }
    Ok(())
}

pub fn label(inhibited: bool) -> &'static str {
    if inhibited {
        "inhibited"
    } else {
        "not inhibited"
    }
}
