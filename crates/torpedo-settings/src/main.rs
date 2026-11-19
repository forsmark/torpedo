//! The settings window of torpedo.
//!
//! It edits the trigger and the aliases in the config file, with the rules of
//! `torpedo_core::config`. With an action such as `--check` it opens no
//! window and needs no display. See `cli::USAGE`.

mod cli;
mod window;

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = match cli::parse_args(&args) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("torpedo-settings: {e}\n\n{}", cli::USAGE);
            return ExitCode::from(2);
        }
    };
    let report = match mode {
        cli::Mode::Help => {
            print!("{}", cli::USAGE);
            return ExitCode::SUCCESS;
        }
        cli::Mode::Version => {
            println!("torpedo-settings {}", torpedo_core::version());
            return ExitCode::SUCCESS;
        }
        cli::Mode::Window { config } => {
            return match cli::resolve(config) {
                Ok(path) => window::run(path),
                Err(e) => {
                    eprintln!("torpedo-settings: {e}");
                    ExitCode::from(2)
                }
            };
        }
        cli::Mode::Check { config } => cli::resolve(config).map(|p| cli::check(&p)),
        cli::Mode::Edit { config, edits } => cli::resolve(config).map(|p| cli::edit(&p, &edits)),
    };
    match report {
        Ok(r) => {
            let _ = std::io::stdout().write_all(r.out.as_bytes());
            let _ = std::io::stderr().write_all(r.err.as_bytes());
            ExitCode::from(r.status as u8)
        }
        Err(e) => {
            eprintln!("torpedo-settings: {e}");
            ExitCode::from(2)
        }
    }
}
