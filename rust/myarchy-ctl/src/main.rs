mod brightness;
mod cli;
mod cursor;
mod display;
mod host;
mod idle;
mod nightlight;
mod wallpaper;

use std::process::ExitCode;

use clap::{CommandFactory, Parser};

use cli::Cli;
use host::Host;
use myarchy_core::core::error::UserError;

fn main() -> ExitCode {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    let host = Host::new(cli.notify, cli.osd);
    let (summary, icon) = cli.command.subject();

    match cli.command.run(&host) {
        Ok(code) => code,
        Err(err) => {
            // The toast carries only what a person can act on; the whole chain, tool output and
            // all, goes to the terminal where it can be read and pasted.
            let message = err
                .downcast_ref::<UserError>()
                .unwrap_or(&UserError::Unexpected)
                .to_string();
            let _ = host.notifier().send(summary, &message, icon, None);
            eprintln!("{err:?}");
            ExitCode::FAILURE
        }
    }
}
