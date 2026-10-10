mod cli;
mod config;
mod error;
mod lex;
mod parse;
mod schedule;
mod tree;

fn main() {
    cli::run();
}
