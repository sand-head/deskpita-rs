use tokio::io::AsyncWriteExt;
use tokio_serial::SerialStream;

pub async fn set_fan_speed(port: &mut SerialStream, speed: i32) -> anyhow::Result<()> {
  // ensure the fan speed is valid
  if (0..=100).contains(&speed) {
    let speed_msg = format!("pwm_{:03}", speed);
    port.write(speed_msg.as_bytes()).await?;
    println!("Set fan speed to {}%", speed);
  } else {
    eprintln!("Invalid fan speed: {}%", speed);
  }

  Ok(())
}
