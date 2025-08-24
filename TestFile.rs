// This file is to demonstrate changes made to core/src/drone.rs
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use tracing::{debug, info, warn};

use std::time::{Duration, Instant};

// Project crates
use core::drone::Drone;
use core::http_api::HttpDroneApi;
use dronecore::transport::FakeTransport;

use serde_json; // pretty printing of snapshots

/// dronectl: CLI to control your drone
#[derive(Parser, Debug)]
#[command(name = "dronectl", about = "CLI to control your drone")]
struct Cli {
    /// Increase verbosity (use multiple times for more detail)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Connection endpoint. Use `fake://` for the in-memory stub, or an http(s) URL for the real API.
    #[arg(long, default_value = "fake://")]
    endpoint: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Connect to a drone
    Connect,

    /// Take off to a target altitude (meters AGL)
    Takeoff { #[arg(long)] altitude: f32 },

    /// Land (will attempt a safe landing and verify)
    Land,

    /// Fetch and print status / telemetry snapshot (pretty JSON)
    Status,
}

fn main() {
    if let Err(e) = main_inner() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn main_inner() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose)?;

    // Endpoint: "fake://" -> in-memory stub; otherwise expect http(s) base URL.
    let endpoint = cli.endpoint.trim();
    let is_fake = endpoint.eq_ignore_ascii_case("fake://");

    // Create the drone facade backed by the chosen transport.
    let mut drone: Box<dyn Drone> = if is_fake {
        info!("Using FAKE transport");
        Box::new(FakeTransport::new())
    } else {
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            bail!("unsupported endpoint scheme for '{endpoint}'. Use 'fake://' or 'http(s)://...'");
        }
        info!("Using HTTP transport -> {endpoint}");
        Box::new(HttpDroneApi::new(endpoint))
    };

    match cli.command {
        Commands::Connect => {
            info!("Connecting to {}", endpoint);
            drone.connect().context("connect failed")?;
            // Simple profile sanity check (if supported by your backend)
            match drone.profile().context("failed to fetch profile")? {
                Some(profile) => info!("connected profile: {}", profile),
                None => warn!("connected, but profile unknown"),
            }
        }

        Commands::Takeoff { altitude } => {
            ensure_connected(&mut *drone)?;
            let status = drone
                .status()
                .context("fetch status before takeoff failed")?;
            if status.armed.unwrap_or(false) {
                bail!("cannot takeoff: drone already armed");
            }

            info!("Arming…");
            drone.arm().context("arm failed")?;

            info!("Taking off to {altitude} m AGL…");
            drone
                .takeoff(altitude)
                .context("takeoff command failed")?;

            // Verify climb within timeout
            verify_within(Duration::from_secs(20), || {
                drone
                    .altitude()
                    .map(|a| a.unwrap_or(0.0) >= altitude * 0.8)
                    .context("read altitude")
            })
            .context("takeoff did not reach expected altitude in time")?;

            info!("Takeoff verified");
        }

        Commands::Land => {
            ensure_connected(&mut *drone)?;

            info!("Landing…");
            drone.land().context("land command failed")?;

            // Post-conditions: near ground & disarmed within timeout
            verify_within(Duration::from_secs(30), || {
                let s = drone.status().context("read status during landing")?;
                let alti_ok = drone
                    .altitude()
                    .context("read altitude during landing")?
                    .unwrap_or(1.0)
                    <= 0.5;
                Ok(alti_ok && !s.armed.unwrap_or(true))
            })
            .context("landing did not complete successfully in time")?;

            info!("Landing verified");
        }

        Commands::Status => {
            ensure_connected(&mut *drone)?;
            let snap = drone
                .snapshot()
                .context("fetch status/telemetry failed")?;
            println!("{}", serde_json::to_string_pretty(&snap)?);
        }
    }

    Ok(())
}

fn init_tracing(verbosity: u8) -> Result<()> {
    // info (0), debug (1), trace (>=2)
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    // Keep noise down from h2/hyper unless explicitly enabled via RUST_LOG
    tracing_subscriber::fmt()
        .with_env_filter(format!("{}{}", level, ",h2=off,hyper=off"))
        .with_target(false)
        .try_init()
        .ok(); // ignore "set more than once" in tests

    debug!("logging initialized at {}", level);
    Ok(())
}

/// Ensure transport is connected; attempt auto-connect if not.
fn ensure_connected(drone: &mut dyn Drone) -> Result<()> {
    if !drone.is_connected()? {
        drone.connect().context("auto-connect failed")?;
    }
    Ok(())
}

/// Poll a condition until it returns Ok(true) or we time out.
fn verify_within<F>(timeout: Duration, mut f: F) -> Result<()>
where
    F: FnMut() -> Result<bool>,
{
    let start = Instant::now();
    loop {
        if f()? {
            return Ok(());
        }
        if start.elapsed() > timeout {
            bail!("verification timed out after {:?}", timeout);
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_takeoff() {
        let cli = Cli::parse_from([
            "dronectl",
            "--endpoint",
            "fake://",
            "takeoff",
            "--altitude",
            "10",
        ]);
        match cli.command {
            Commands::Takeoff { altitude } => assert_eq!(altitude, 10.0),
            _ => panic!("expected takeoff subcommand"),
        }
    }

    #[test]
    fn status_on_fake_should_connect_and_fetch() {
        init_tracing(0).unwrap();
        let mut d: Box<dyn Drone> = Box::new(FakeTransport::new());
        ensure_connected(&mut *d).unwrap();
        let snap = d.snapshot().unwrap();
        // basic sanity: presence of sequence or similar field is enough here
        // (exact expectations depend on your FakeTransport behavior)
        let _ = serde_json::to_string(&snap).unwrap();
    }
}
