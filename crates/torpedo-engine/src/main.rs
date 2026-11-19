//! The IBus input method engine of torpedo.
//!
//! IBus starts it from the component file with `--ibus`. Without that flag it
//! only prints its version: the real address lookup would find the live
//! daemon, and a stray run must not register there.

mod address;
mod config_watch;
mod engine;
mod ibus_types;

use std::process::ExitCode;

use futures_util::StreamExt;
use zbus::{MessageStream, connection};

/// The bus name of the component in the component file.
const BUS_NAME: &str = "org.freedesktop.IBus.Torpedo";
const FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";

fn banner() -> String {
    format!("torpedo-engine {}", torpedo_core::version())
}

fn main() -> ExitCode {
    if !std::env::args().skip(1).any(|a| a == "--ibus") {
        println!("{}", banner());
        println!("IBus starts this engine with --ibus. It does nothing without it.");
        return ExitCode::SUCCESS;
    }
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("torpedo: cannot start: {e}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("torpedo: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Connect to the daemon, register, and serve until the connection closes.
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let address = address::ibus_address(&address::RealHost)?;
    engine::debug(format!("connecting to {address}"));
    // Serve the factory before the name exists: the daemon calls
    // CreateEngine as soon as it sees the name.
    let connection = connection::Builder::address(address.as_str())?
        .serve_at(FACTORY_PATH, engine::Factory::default())?
        .name(BUS_NAME)?
        .build()
        .await?;
    let mut messages = MessageStream::from(&connection);
    while messages.next().await.is_some() {}
    engine::debug("the daemon closed the connection");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_names_the_engine() {
        assert!(banner().starts_with("torpedo-engine "));
    }
}
