use std::{collections::HashMap, sync::Mutex, time::Duration};

use clap::Subcommand;
use once_cell::sync::Lazy;
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncWriteExt},
  time,
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
  let mut current_speed = -1;

  loop {
    let cpu_temp = get_cpu_temp().await?;
    let new_speed = get_new_speed(cpu_temp) as i32;

    if new_speed != current_speed {
      println!("CPU temp: {}°C, fan speed: {}%", cpu_temp, new_speed);
      fan::set_fan_speed(port, new_speed).await?;
      current_speed = new_speed;
    }

    time::sleep(Duration::from_secs(10)).await;
  }
}

pub async fn send_power_off(port: &mut SerialStream) -> anyhow::Result<()> {
  port.write(b"power_off").await?;
  Ok(())
}

fn get_new_speed(cpu_temp: i32) -> f64 {
  let data = GLOBAL_DATA.lock().unwrap();
  if data.contains_key(&cpu_temp) {
    return data[&cpu_temp].into();
  }

  let mut upper_point = (0, 0);
  let mut lower_point = (0, 0);

  // get the points that bracket the current CPU temp
  for (temp, speed) in data.iter() {
    if cpu_temp > *temp {
      lower_point = (*temp, *speed);
    } else {
      upper_point = (*temp, *speed);
    }
  }

  // calculate the new speed by interpolating between the two points
  let slope = (upper_point.1 - lower_point.1) as f64 / (upper_point.0 - lower_point.0) as f64;
  let y_intercept = upper_point.1 as f64 - slope * upper_point.0 as f64;
  slope * cpu_temp as f64 + y_intercept
}

async fn get_cpu_temp() -> anyhow::Result<i32> {
  let mut temp_file = File::open("/sys/class/thermal/thermal_zone0/temp").await?;
  let mut temp_str = String::new();

  temp_file.read_to_string(&mut temp_str).await?;
  let temp = temp_str.trim().parse::<i32>()?;
  Ok(temp / 1000)
}
