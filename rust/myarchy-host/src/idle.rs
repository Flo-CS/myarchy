use std::io;
use std::os::unix::net::UnixStream;

use anyhow::{bail, Context, Result};
use idle_inhibitor::ipc::{self, Request, Response};
use myarchy_core::idle::IdleCtl;

pub struct IdleInhibitor;

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

impl IdleCtl for IdleInhibitor {
    fn enable(&self) -> Result<()> {
        oneshot(Request::Enable)?;
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        oneshot(Request::Disable)?;
        Ok(())
    }

    fn status(&self) -> Result<bool> {
        match oneshot(Request::Status)? {
            Response::Status(inhibited) => Ok(inhibited),
            _ => bail!("unexpected response from daemon"),
        }
    }

    fn watch(&self, on_change: &mut dyn FnMut(bool) -> Result<()>) -> Result<()> {
        let stream = connect()?;
        ipc::send_request(&stream, &Request::Watch).context("failed to send request")?;

        let de = serde_json::Deserializer::from_reader(&stream);
        for response in de.into_iter::<Response>() {
            match response {
                Ok(Response::Status(inhibited)) => on_change(inhibited)?,
                Ok(Response::Err(msg)) => bail!(msg),
                Ok(_) => {}
                Err(_) => break,
            }
        }
        Ok(())
    }
}
