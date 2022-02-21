use std::{collections::HashMap, sync::Mutex, time::Duration};

use clap::Subcommand;
use once_cell::sync::Lazy;
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_serial::SerialStream;

use crate::fan;

static GLOBAL_DATA: Lazy<Mutex<HashMap<i32, i32>>> = Lazy::new(|| {
  let mut m = HashMap::new();
  m.insert(40, 25);
  m.insert(50, 50);
  m.insert(65, 75);
  m.insert(75, 100);
  Mutex::new(m)
});

#[derive(Subcommand, Debug)]
pub enum ServiceCommands {
  AutoFan,
  PowerOff,
}

pub async fn sync_fan_speed_with_cpu_temp(port: &mut SerialStream) -> anyhow::Result<()> {
  loop {
    let data = GLOBAL_DATA.lock().unwrap();
    let cpu_temp = get_cpu_temp().await?;

    let mut new_speed = 0;
    for (temp, speed) in data.iter() {
      if cpu_temp >= *temp {
        new_speed = *speed;
        break;
      }
    }

    println!("CPU temp: {}°C, fan speed: {}%", cpu_temp, new_speed);
    fan::set_fan_speed(port, new_speed).await?;
    tokio::time::sleep(Duration::from_secs(10)).await;
  }
}

pub async fn send_power_off(port: &mut SerialStream) -> anyhow::Result<()> {
  port.write(b"power_off").await?;
  Ok(())
}

async fn get_cpu_temp() -> anyhow::Result<i32> {
  let mut temp_file = File::open("/sys/class/thermal/thermal_zone0/temp").await?;
  let mut temp_str = String::new();

  temp_file.read_to_string(&mut temp_str).await?;
  let temp = temp_str.trim().parse::<i32>()?;
  Ok(temp / 1000)
}
