use std::io::{self, Write};
use std::net::Shutdown;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const SOCKET_NAME: &str = "myarchy-idle-inhibitor";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Request {
    Enable,
    Disable,
    Status,
    Watch,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Response {
    Ok,
    Status(bool),
    Err(String),
}

pub fn address() -> io::Result<SocketAddr> {
    SocketAddr::from_abstract_name(SOCKET_NAME.as_bytes())
}

pub fn bind() -> io::Result<UnixListener> {
    UnixListener::bind_addr(&address()?)
}

pub fn connect() -> io::Result<UnixStream> {
    UnixStream::connect_addr(&address()?)
}

pub fn read_request(stream: &UnixStream) -> Result<Request> {
    Ok(serde_json::from_reader(stream)?)
}

pub fn write_response(mut stream: &UnixStream, response: &Response) -> Result<()> {
    serde_json::to_writer(stream, response)?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}

pub fn send_request(stream: &UnixStream, request: &Request) -> Result<()> {
    serde_json::to_writer(stream, request)?;
    stream.shutdown(Shutdown::Write)?;
    Ok(())
}

pub fn read_response(stream: &UnixStream) -> Result<Response> {
    Ok(serde_json::from_reader(stream)?)
}
