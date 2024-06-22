extern crate bme280;
extern crate linux_embedded_hal as hal;
use rusqlite::{Connection, Result};
use bme280::i2c::BME280;
use hal::{Delay, I2cdev};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
mod lib;
use lib::{DbService, Measurement};
fn main() -> Result<(), rusqlite::Error> {
    let conn = Connection::open(lib::DB_FILE_NAME)?;
    let mut db_service = DbService::new(&conn);
    db_service.create_table()?;

    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();
    let mut bme280 = BME280::new_secondary(i2c_bus);
    let mut delay = Delay;
    bme280.init(&mut delay).unwrap();
    loop {
        let measurements = bme280.measure(&mut delay).unwrap();
        println!("Relative Humidity = {}%", measurements.humidity);
        println!("Temperature = {} deg C", measurements.temperature);
        println!("Pressure = {} pascals", measurements.pressure);

        let start = SystemTime::now();
        let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs();
        db_service.save(Measurement {
            timestamp: since_the_epoch as i64,
            pressure: measurements.pressure as f64,
            temperature: measurements.temperature as f64,
            humidity: measurements.humidity as f64
        })?;
        thread::sleep(Duration::from_secs(30));
    }
    // conn.close()?;
}