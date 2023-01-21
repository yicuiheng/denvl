#![feature(assert_matches, box_patterns)]

mod commandline_client;
mod consts;
mod diagnostic;
mod lex;
mod named_pipe;
mod parse;
mod server;
mod source;
mod syntax_node;
use clap::{command, Arg, Command};

const RUN_COMMAND: &str = "run";
const SHUTDOWN_COMMAND: &str = "shutdown";
const SERVER_COMMAND: &str = "__server";

fn main() {
    let matches = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new(RUN_COMMAND)
                .about("compile and run specified denvl source file")
                .arg(Arg::new("filename").required(true)),
        )
        .subcommand(Command::new(SHUTDOWN_COMMAND).about("shutdown denvl server"))
        .subcommand(Command::new(SERVER_COMMAND).hide(true))
        .get_matches();

    match matches.subcommand() {
        Some((RUN_COMMAND, sub_matches)) => {
            let filename = sub_matches
                .get_one::<String>("filename")
                .expect("<filename> required");
            commandline_client::run(filename);
        }
        Some((SHUTDOWN_COMMAND, _)) => commandline_client::shutdown(),
        Some((SERVER_COMMAND, _)) => server::run(),
        _ => unreachable!(),
    }
}
