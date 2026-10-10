use std::{env, fs, process::ExitCode};

use crate::schedule;

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
    ring.schedule
    schedule/
        standard.time
        ramadan.time
        holidays.time
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

    if root.join("ring.schedule").exists() || root.join("schedule").exists() {
        eprintln!("ringc: project already exists");
        return ExitCode::FAILURE;
    }

    if let Err(error) = fs::create_dir(root.join("schedule")) {
        eprintln!("ringc: cannot create schedule directory: {error}");
        return ExitCode::FAILURE;
    }

    let files = [
        ("ring.schedule", include_str!("../templates/ring.schedule")),
        (
            "schedule/standard.time",
            include_str!("../templates/schedule/standard.time"),
        ),
        (
            "schedule/ramadan.time",
            include_str!("../templates/schedule/ramadan.time"),
        ),
        (
            "schedule/holidays.time",
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
    match schedule::load_project(std::path::Path::new(path)) {
        Ok(_) => {
            println!("ringc: check passed");
            ExitCode::SUCCESS
        }

        Err(failure) => {
            eprintln!("ringc: {failure}");
            ExitCode::FAILURE
        }
    }
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
