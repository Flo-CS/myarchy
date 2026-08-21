use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::process::ExitCode;

use clap::Parser;
use idle_inhibitor::ipc::{self, Request, Response};

#[derive(Parser, Debug)]
#[command(name = "idle-inhibitor", bin_name = "idle-inhibitor")]
enum Cli {
    Enable,
    Disable,
    Status,
    Watch,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let stream = match ipc::connect() {
        Ok(s) => s,
        Err(e)
            if matches!(
                e.kind(),
                io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound
            ) =>
        {
            eprintln!("error: daemon is not running. start it with `idle-inhibitord`.");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("error: failed to connect to daemon: {e}");
            return ExitCode::FAILURE;
        }
    };

    match cli {
        Cli::Enable => run_oneshot(stream, Request::Enable),
        Cli::Disable => run_oneshot(stream, Request::Disable),
        Cli::Status => run_oneshot(stream, Request::Status),
        Cli::Watch => run_watch(stream),
    }
}

fn run_oneshot(stream: UnixStream, request: Request) -> ExitCode {
    if let Err(e) = ipc::send_request(&stream, &request) {
        eprintln!("error: failed to send request: {e}");
        return ExitCode::FAILURE;
    }

    let response = match ipc::read_response(&stream) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: failed to read response: {e}");
            return ExitCode::FAILURE;
        }
    };

    match response {
        Response::Ok => ExitCode::SUCCESS,
        Response::Status(inhibited) => {
            println!("{}", state_label(inhibited));
            if inhibited {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Response::Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run_watch(stream: UnixStream) -> ExitCode {
    if let Err(e) = ipc::send_request(&stream, &Request::Watch) {
        eprintln!("error: failed to send request: {e}");
        return ExitCode::FAILURE;
    }

    let mut stdout = io::stdout().lock();
    let de = serde_json::Deserializer::from_reader(&stream);
    for response in de.into_iter::<Response>() {
        match response {
            Ok(Response::Status(inhibited)) => {
                if writeln!(stdout, "{}", state_label(inhibited)).is_err()
                    || stdout.flush().is_err()
                {
                    return ExitCode::FAILURE;
                }
            }
            Ok(Response::Err(msg)) => {
                eprintln!("error: {msg}");
                return ExitCode::FAILURE;
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }

    ExitCode::FAILURE
}

fn state_label(inhibited: bool) -> &'static str {
    if inhibited {
        "inhibited"
    } else {
        "not inhibited"
    }
}
