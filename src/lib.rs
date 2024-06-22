extern crate bme280;
extern crate linux_embedded_hal as hal;

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Measurement {
    pub(crate) timestamp: i64,
    pub(crate) pressure: f64,
    pub(crate) temperature: f64,
    pub(crate) humidity: f64,
}

pub const DB_FILE_NAME: &str = "./db.sqlite";

pub struct DbService<'a> {
    connection: &'a Connection,
}
impl<'a> DbService<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create_table(&mut self) -> Result<(), rusqlite::Error> {
        self.connection.execute(
        "CREATE TABLE IF NOT EXISTS Measurement (id INTEGER PRIMARY KEY, timestamp INTEGER, pressure REAL, temperature REAL, humidity REAL)",
        [],
    )?;
        return Ok(());
    }

    pub fn save(&mut self, measurement: Measurement) -> Result<(), rusqlite::Error> {
        self.connection.execute("INSERT INTO Measurement (timestamp, pressure, temperature, humidity) VALUES (?, ?, ?, ?)", params![measurement.timestamp, measurement.pressure, measurement.temperature, measurement.humidity])?;
        return Ok(());
    }

    pub fn print_all(&mut self) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT timestamp, pressure, temperature, humidity FROM Measurement")?;
        let measurement_iter = stmt.query_map([], |row| {
            Ok(Measurement {
                timestamp: row.get(0)?,
                pressure: row.get(1)?,
                temperature: row.get(2)?,
                humidity: row.get(3)?,
            })
        })?;

        for measurement in measurement_iter {
            println!("Found measurement: {:?}", measurement);
        }
        Ok(())
    }

    pub fn get_last(&mut self) -> Result<Measurement, rusqlite::Error> {
        let last_row_id = self.connection.last_insert_rowid();
        let mut stmt = self
            .connection
            .prepare("SELECT timestamp, pressure, temperature, humidity FROM Measurement ORDER BY timestamp desc LIMIT 1")?;
        print!("{}", last_row_id);
        let measurement_iter = stmt.query_map([], |row| {
            Ok(Measurement {
                timestamp: row.get(0)?,
                pressure: row.get(1)?,
                temperature: row.get(2)?,
                humidity: row.get(3)?,
            })
        })?;
        for measurement in measurement_iter {
            return measurement;
        }
        !panic!("Test");
    }
}
