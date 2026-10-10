use std::{env, fs, process::ExitCode};

use crate::{config, lex, parse};

pub fn run() -> ExitCode {
    let mut args = env::args().skip(1);

    let Some(command) = args.next() else {
        usage();
        return ExitCode::FAILURE;
    };

    match command.as_str() {
        "init" => {
            if args.next().is_some() {
                usage();
                return ExitCode::FAILURE;
            }

            init()
        }

        "check" => {
            let Some(path) = args.next() else {
                usage();
                return ExitCode::FAILURE;
            };

            if args.next().is_some() {
                usage();
                return ExitCode::FAILURE;
            }

            check(&path)
        }

        "build" => {
            let Some(path) = args.next() else {
                usage();
                return ExitCode::FAILURE;
            };

            if args.next().is_some() {
                usage();
                return ExitCode::FAILURE;
            }

            build(&path)
        }

        "install" => {
            if args.next().is_some() {
                usage();
                return ExitCode::FAILURE;
            }

            install()
        }

        "update" => {
            if args.next().is_some() {
                usage();
                return ExitCode::FAILURE;
            }

            update()
        }

        "guide" => {
            if args.next().is_some() {
                usage();
                return ExitCode::FAILURE;
            }

            guide()
        }

        "help" | "--help" | "-h" => {
            usage();
            ExitCode::SUCCESS
        }

        _ => {
            eprintln!("ringc: unknown command '{command}'");
            usage();
            ExitCode::FAILURE
        }
    }
}

fn guide() -> ExitCode {
    println!(
        "\
ringc guide

Create a project:
    ringc init

Check a project:
    ringc check <path>

Build a schedule:
    ringc build <path>

Install the system:
    ringc install

Update an installed schedule:
    ringc update

Project files:
    ring.cat
    bell/
        standard.bell
        ramadan.bell
        holidays.bell
        ...
"
    );

    ExitCode::SUCCESS
}

fn usage() {
    eprintln!(
        "\
usage:
    ringc init
    ringc check <path>
    ringc build <path>
    ringc install
    ringc update"
    );
}

fn init() -> ExitCode {
    let root = std::path::Path::new(".");

    if root.join("ring.cat").exists() || root.join("bell").exists() {
        eprintln!("ringc: project already exists");
        return ExitCode::FAILURE;
    }

    if let Err(error) = fs::create_dir(root.join("bell")) {
        eprintln!("ringc: cannot create bell directory: {error}");
        return ExitCode::FAILURE;
    }

    let files = [
        ("ring.cat", include_str!("../templates/ring.schedule")),
        (
            "schedule/standard.bell",
            include_str!("../templates/schedule/standard.time"),
        ),
        (
            "schedule/ramadan.bell",
            include_str!("../templates/schedule/ramadan.time"),
        ),
        (
            "schedule/holidays.bell",
            include_str!("../templates/schedule/holidays.time"),
        ),
    ];

    for (path, source) in files {
        if let Err(error) = fs::write(root.join(path), source) {
            eprintln!("ringc: cannot create {path}: {error}");
            return ExitCode::FAILURE;
        }
    }

    println!("ringc: project initialized");
    ExitCode::SUCCESS
}

fn check(path: &str) -> ExitCode {
    let path = std::path::Path::new(path);
    let ring = path.join("ring.cat");

    let source = match fs::read_to_string(&ring) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("ringc: cannot read {}: {error}", ring.display());
            return ExitCode::FAILURE;
        }
    };

    let tokens = match lex::lex(&source) {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("ringc: {error}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(error) = config::parse(&tokens) {
        eprintln!("ringc: {error}");
        return ExitCode::FAILURE;
    }

    println!("ringc: check passed");
    ExitCode::SUCCESS
}

fn build(_path: &str) -> ExitCode {
    eprintln!("ringc: build is not implemented");
    ExitCode::FAILURE
}

fn install() -> ExitCode {
    eprintln!("ringc: install is not implemented");
    ExitCode::FAILURE
}

fn update() -> ExitCode {
    eprintln!("ringc: update is not implemented");
    ExitCode::FAILURE
}
