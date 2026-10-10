use std::process::ExitCode;

mod cli;
mod compile;
mod config;
mod error;
mod flash;
mod lex;
mod parse;
mod schedule;
mod tree;

fn main() -> ExitCode {
    cli::run()
}
