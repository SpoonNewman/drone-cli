use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{debug, info};
use dronecore::{drone::Drone, transport::FakeTransport};

#[derive(Parser, Debug)]
#[command(name = "dronectl", about = "CLI to control your drone")]
struct Cli {
    /// Increase verbosity (use multiple times for more detail)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Connection endpoint (placeholder; swap for real)
    #[arg(long, default_value = "fake://")]
    endpoint: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Establish a connection
    Connect,
    /// Arm the drone (from Disarmed -> Armed)
    Arm,
    /// Take off to a target altitude in meters
    Takeoff { #[arg(default_value_t = 5.0)] altitude: f32 },
    /// Land the drone
    Land,
    /// Get current status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose)?;

    // Choose transport based on endpoint scheme; we'll use Fake for now.
    let transport = match cli.endpoint.as_str() {
        "fake://" => dronecore::transport::FakeTransport::new(),
        _ => dronecore::transport::FakeTransport::new(), // TODO: udp://, serial://
    };

    let mut drone = Drone::new(transport);
    drone.connect().await.context("failed to connect")?;
    info!("connected to {}", cli.endpoint);

    match cli.command {
        Commands::Connect => {
            println!("Connected.");
        }
        Commands::Arm => {
            drone.arm().await.context("arm failed")?;
            println!("Armed.");
        }
        Commands::Takeoff { altitude } => {
            drone.takeoff(altitude).await.with_context(|| format!("takeoff to {altitude}m failed"))?;
            println!("Takeoff initiated to {altitude}m.");
        }
        Commands::Land => {
            drone.land().await.context("land failed")?;
            println!("Landing.");
        }
        Commands::Status => {
            let s = drone.status().await.context("status failed")?;
            println!("{s}");
        }
    }

    Ok(())
}

fn init_tracing(verbosity: u8) -> Result<()> {
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(format!("{}{}", level, ",h2=off,hyper=off"))
        .with_target(false)
        .try_init()
        .ok(); // ignore "set more than once" in tests
    debug!("logging initialized at {}", level);
    Ok(())
}
