mod backend;
mod brightness;
mod cli;
mod cursor;
mod display;
mod idle;
mod nightlight;
mod wallpaper;

use clap::{CommandFactory, Parser};
use myarchy_core::error::AppError;
use myarchy_core::notify::NotifierCtl;

use cli::Cli;

fn main() {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    if let Err(err) = Cli::parse().run() {
        if let Some(notification) = err
            .downcast_ref::<AppError>()
            .and_then(AppError::notification)
        {
            let _ = backend::notifierctl().send(
                notification.summary,
                &err.to_string(),
                notification.icon.as_str(),
                None,
            );
        }
        eprintln!("{err}");
        std::process::exit(1);
    }
}
