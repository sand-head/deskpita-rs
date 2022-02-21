use clap::{Parser, Subcommand};
use tokio::io::AsyncWriteExt;
use tokio_serial::SerialPortBuilderExt;

const DEFAULT_TTY: &str = "/dev/ttyUSB0";

/// Services for controlling the DeskPi Pro
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Control the DeskPi Pro's CPU fan speed
  Fan { speed: Option<i32> },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let mut port = tokio_serial::new(DEFAULT_TTY, 9699).open_native_async()?;

  match &cli.command {
    Commands::Fan { speed } => {
      // todo: automagically control fan speed based on CPU temp
      if let Some(speed) = speed {
        let speed_msg = format!("pwm_{:03}", speed);
        port.write(speed_msg.as_bytes()).await?;
      }
    }
  }

  Ok(())
}
