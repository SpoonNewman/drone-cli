use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{debug, info};
use dronecore::{drone::Drone};

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
// new: using HttpDroneApi
use dronecore::http_api::HttpDroneApi; // add this at the top

// FIXME: This `endpoint` should probably be changed to `base` or `base_address`
// it will effectively mean `http://<address>` and the CLI will use "endpoints" like `/status`
// or `/commands/land`.
// The effect in conclusion looks like `http://<address>/status` or `http://<address>/commands/land`
// which is why `base` or some similar naming makes more sense than `endpoint` right now
let api = HttpDroneApi::new(cli.endpoint.clone());
let mut drone = Drone::new(api);
;

    // TODO: For each of these commands we need to check whether the command handler, i.e. `drone.arm().await.context("arm failed")?;` actually
    // succeeded or not and handle that failure.
    // TODO: Additionally we need to have a test for each of these commands that confirms the command works successfully.
    match cli.command {
        Commands::Connect => {
            // TODO: This needs to evaluate the state of whether we are connected or not
            // Questions:
            //  - How do we know if we're connected? Do we instantiate another connection to test? Probably
            println!("Connected.");
        }
        Commands::Arm => {
            drone.arm().await.context("arm failed")?;
            println!("Armed.");
        }
        Commands::Takeoff { altitude } => {
            // TODO: Confirm we actually are working with meters.
            drone.takeoff(altitude).await.with_context(|| format!("takeoff to {altitude}m failed"))?;
            println!("Takeoff initiated to {altitude}m.");
        }
        Commands::Land => {
            // TODO: What happens if landing failed? What's the fallback behavior? That behavior handling needs to be written in.
            drone.land().await.context("land failed")?;
            println!("Landing.");
        }
        Commands::Status => {
            // TODO: This needs to get a TelemetrySnapshot DTO from the FlightBridge
            // then it should print it out. Could probably just print out the JSON model dump of TelemetrySnasphot
            // and be done with it. 
            let s = drone.status().await.context("status failed")?;
            println!("{:#?}", s);
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
