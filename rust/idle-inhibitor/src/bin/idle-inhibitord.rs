use std::error::Error;
use std::os::unix::net::UnixStream;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;

use idle_inhibitor::ipc::{self, Request, Response};
use zbus::blocking::Connection;

const APP_NAME: &str = "idle-inhibitor";
const INHIBIT_REASON: &str = "user requested";

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
    fn new(connection: &Connection) -> zbus::Result<Self> {
        Ok(Self {
            proxy: ScreenSaverProxyBlocking::new(connection)?,
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

    fn enable(&mut self) -> zbus::Result<()> {
        if self.cookie.is_none() {
            self.cookie = Some(self.proxy.inhibit(APP_NAME, INHIBIT_REASON)?);
            self.broadcast(true);
        }
        Ok(())
    }

    fn disable(&mut self) -> zbus::Result<()> {
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

fn run() -> std::result::Result<(), Box<dyn Error>> {
    let listener = ipc::bind()?;
    let connection = Connection::session()?;
    let mut daemon = Daemon::new(&connection)?;

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };

        let request = match ipc::read_request(&stream) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("read error: {e}");
                continue;
            }
        };

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
