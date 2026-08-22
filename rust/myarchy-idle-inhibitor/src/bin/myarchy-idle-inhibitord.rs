use std::os::unix::net::UnixStream;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use myarchy_idle_inhibitor::ipc::{self, Request, Response};
use zbus::blocking::Connection;

const APP_NAME: &str = "myarchy-idle-inhibitor";
const INHIBIT_REASON: &str = "user requested";

/// Bounds how long a connection that never sends anything keeps its reader thread alive.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

#[zbus::proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/org/freedesktop/ScreenSaver",
    gen_blocking = true
)]
trait ScreenSaver {
    fn inhibit(&self, app_name: &str, reason: &str) -> zbus::Result<u32>;
    fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
}

struct Daemon<'p> {
    proxy: ScreenSaverProxyBlocking<'p>,
    cookie: Option<u32>,
    watchers: Vec<mpsc::Sender<bool>>,
}

impl<'p> Daemon<'p> {
    fn new(connection: &Connection) -> Result<Self> {
        Ok(Self {
            proxy: ScreenSaverProxyBlocking::new(connection)
                .context("failed to connect to org.freedesktop.ScreenSaver")?,
            cookie: None,
            watchers: Vec::new(),
        })
    }

    fn handle(&mut self, request: Request) -> Response {
        match request {
            Request::Enable => match self.enable() {
                Ok(()) => Response::Ok,
                Err(e) => Response::Err(e.to_string()),
            },
            Request::Disable => match self.disable() {
                Ok(()) => Response::Ok,
                Err(e) => Response::Err(e.to_string()),
            },
            Request::Status => Response::Status(self.is_inhibited()),
            Request::Watch => Response::Err("watch is a streaming request".into()),
        }
    }

    fn is_inhibited(&self) -> bool {
        self.cookie.is_some()
    }

    fn enable(&mut self) -> Result<()> {
        if self.cookie.is_none() {
            self.cookie = Some(self.proxy.inhibit(APP_NAME, INHIBIT_REASON)?);
            self.broadcast(true);
        }
        Ok(())
    }

    fn disable(&mut self) -> Result<()> {
        if let Some(cookie) = self.cookie.take() {
            self.proxy.un_inhibit(cookie)?;
            self.broadcast(false);
        }
        Ok(())
    }

    fn subscribe(&mut self, stream: UnixStream) {
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(self.is_inhibited());
        self.watchers.push(tx);

        thread::spawn(move || {
            for state in rx {
                let response = Response::Status(state);
                if ipc::write_response(&stream, &response).is_err() {
                    break;
                }
            }
        });
    }

    fn broadcast(&mut self, state: bool) {
        self.watchers.retain(|tx| tx.send(state).is_ok());
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let listener = ipc::bind().context("failed to bind ipc socket")?;
    let connection = Connection::session().context("failed to connect to session bus")?;
    let mut daemon = Daemon::new(&connection)?;

    // Reading a request blocks, so it happens off the accept loop: one client that connects and
    // never sends would otherwise hold up every other client for as long as it stayed open.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("accept error: {e}");
                    continue;
                }
            };
            let tx = tx.clone();
            thread::spawn(move || {
                let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
                match ipc::read_request(&stream) {
                    Ok(request) => {
                        let _ = stream.set_read_timeout(None);
                        let _ = tx.send((request, stream));
                    }
                    Err(e) => eprintln!("read error: {e}"),
                }
            });
        }
    });

    for (request, stream) in rx {
        match request {
            Request::Watch => daemon.subscribe(stream),
            other => {
                let response = daemon.handle(other);
                if let Err(e) = ipc::write_response(&stream, &response) {
                    eprintln!("write error: {e}");
                }
            }
        }
    }

    Ok(())
}
