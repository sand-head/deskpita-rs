use clap::{Parser, Subcommand};
use services::ServiceCommands;
use tokio_serial::SerialPortBuilderExt;

mod fan;
mod services;

const DEFAULT_TTY: &str = "/dev/ttyUSB0";

/// Services for controlling the DeskPi Pro.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Handles various services.
  Service {
    #[clap(subcommand)]
    service: ServiceCommands,
  },
  /// Control the DeskPi Pro's CPU fan speed.
  Fan {
    /// Sets the fan speed to the given percentage.
    /// If unset, the speed will be determined by the CPU temperature.
    #[clap(short, long)]
    speed: Option<i32>,
  },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let mut port = tokio_serial::new(DEFAULT_TTY, 9600).open_native_async()?;

  match &cli.command {
    Commands::Service { service } => match &service {
      ServiceCommands::Fan => services::sync_fan_speed_with_cpu_temp(&mut port).await?,
      ServiceCommands::PowerOff => services::send_power_off(&mut port).await?,
    },
    Commands::Fan { speed } => {
      if let Some(speed) = speed {
        fan::set_fan_speed(&mut port, *speed).await?;
      } else {
        // automagically control fan speed based on CPU temp
        // this requires starting a service that polls the CPU temp
        // and sets the fan speed accordingly
        todo!();
      }
    }
  }

  Ok(())
}
