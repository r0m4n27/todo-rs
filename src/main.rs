use std::io;

use thiserror::Error;
use tokio;

use actions::{list_todos, purge_todos, report_todos, todo_files};
use api::ApiError;
use clap::ArgMatches;
use cli::create_cli;
use config::{Config, ConfigError};
use project::{base_dir, ProjectError};

mod actions;
mod api;
mod cli;
mod config;
mod project;
mod todo;
mod todo_parser;

extern crate clap;

#[derive(Debug, Error)]
pub enum TodoError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Project(#[from] ProjectError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Api(#[from] ApiError),
}

type Result<T> = std::result::Result<T, TodoError>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_matches = create_cli();
    let root = base_dir()?;

    let conf = Config::default(root).await?;

    match cli_matches.subcommand() {
        ("list", Some(sub_matches)) => handle_list_todos(&conf, sub_matches)?,
        ("files", _) => todo_files(&conf),
        ("report", _) => report_todos(&conf).await?,
        ("purge", _) => purge_todos(&conf).await?,
        _ => {}
    }

    Ok(())
}

fn handle_list_todos(conf: &Config, matches: &ArgMatches) -> Result<()> {
    let mut unreported = false;
    let mut reported = false;

    if matches.is_present("reported") {
        reported = true
    }

    if matches.is_present("unreported") {
        unreported = true
    }

    if !unreported && !reported {
        reported = true;
        unreported = true
    }

    list_todos(conf, reported, unreported)
}
