use std::io::{self, Write};
use std::net::Shutdown;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

use serde::{Deserialize, Serialize};

pub const SOCKET_NAME: &str = "idle-inhibitor";

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

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Codec(serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "i/o error: {e}"),
            Error::Codec(e) => write!(f, "codec error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Codec(e) => Some(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Codec(e)
    }
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
