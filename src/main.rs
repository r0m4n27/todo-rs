use actions::{list_todos, todo_files};
use clap::ArgMatches;
use cli::create_cli;
use config::Config;

mod actions;
mod cli;
mod config;
mod project;
mod todo;
mod todo_parser;

extern crate clap;

fn main() {
    let cli_matches = create_cli();
    let conf = Config::default().unwrap();

    match cli_matches.subcommand() {
        ("list", Some(sub_matches)) => handle_list_todos(&conf, sub_matches),
        ("files", _) => todo_files(&conf),
        _ => {}
    }
}

fn handle_list_todos(conf: &Config, matches: &ArgMatches) {
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
