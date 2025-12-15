extern crate bme280;
extern crate linux_embedded_hal as hal;

use core::time;

use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};
use now::DateTimeNow;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Measurement {
    pub(crate) timestamp: i64,
    pub(crate) pressure: f64,
    pub(crate) temperature: f64,
    pub(crate) humidity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AverageData {
    pub label: String,
    pub avg_temperature: f64,
    pub avg_humidity: f64,
    pub count: i64,
}

pub const DB_FILE_NAME: &str = "db.sqlite";//"/home/passi/.weatherapp/db.sqlite";

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

    fn get_by_timestamp(&mut self, timestamp: i64) -> Result<Vec<Measurement>, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT timestamp, pressure, temperature, humidity FROM Measurement WHERE timestamp > (?) ORDER BY timestamp desc")?;
        //let now = Utc::now();
        //let beginning_of_day = now.beginning_of_day().timestamp();
        //now.beginning_of_week();
        let measurement_iter = stmt.query_map([timestamp], |row| {
            Ok(Measurement {
                timestamp: row.get(0)?,
                pressure: row.get(1)?,
                temperature: row.get(2)?,
                humidity: row.get(3)?,
            })
        })?;

        // Here we unwrap
        return Ok(measurement_iter.filter_map(|measurement| measurement.ok()).collect::<Vec<Measurement>>());
    }

    pub fn get_by_start_of_day(&mut self) -> Result<Vec<Measurement>, rusqlite::Error> {
        let now = Utc::now();
        let beginning_of_day = now.beginning_of_day().timestamp();
        return self.get_by_timestamp(beginning_of_day);
    }

    pub fn get_hourly_averages(&mut self, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> Result<Vec<AverageData>, rusqlite::Error> {
        let (query, params_vec): (&str, Vec<i64>) = match (start_timestamp, end_timestamp) {
            (Some(start), Some(end)) => {
                ("SELECT strftime('%H', datetime(timestamp, 'unixepoch')) as hour, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 WHERE timestamp >= ? AND timestamp <= ?
                 GROUP BY hour 
                 ORDER BY hour", vec![start, end])
            },
            (Some(start), None) => {
                ("SELECT strftime('%H', datetime(timestamp, 'unixepoch')) as hour, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 WHERE timestamp >= ?
                 GROUP BY hour 
                 ORDER BY hour", vec![start])
            },
            _ => {
                ("SELECT strftime('%H', datetime(timestamp, 'unixepoch')) as hour, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 GROUP BY hour 
                 ORDER BY hour", vec![])
            }
        };

        let mut stmt = self.connection.prepare(query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        
        let result_iter = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(AverageData {
                label: format!("Hour {}", row.get::<_, String>(0)?),
                avg_temperature: row.get(1)?,
                avg_humidity: row.get(2)?,
                count: row.get(3)?,
            })
        })?;

        Ok(result_iter.filter_map(|r| r.ok()).collect())
    }

    pub fn get_weekday_averages(&mut self, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> Result<Vec<AverageData>, rusqlite::Error> {
        let (query, params_vec): (&str, Vec<i64>) = match (start_timestamp, end_timestamp) {
            (Some(start), Some(end)) => {
                ("SELECT strftime('%w', datetime(timestamp, 'unixepoch')) as weekday, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 WHERE timestamp >= ? AND timestamp <= ?
                 GROUP BY weekday 
                 ORDER BY weekday", vec![start, end])
            },
            (Some(start), None) => {
                ("SELECT strftime('%w', datetime(timestamp, 'unixepoch')) as weekday, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 WHERE timestamp >= ?
                 GROUP BY weekday 
                 ORDER BY weekday", vec![start])
            },
            _ => {
                ("SELECT strftime('%w', datetime(timestamp, 'unixepoch')) as weekday, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 GROUP BY weekday 
                 ORDER BY weekday", vec![])
            }
        };

        let mut stmt = self.connection.prepare(query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        
        let weekday_names = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        
        let result_iter = stmt.query_map(params_refs.as_slice(), |row| {
            let weekday_num: String = row.get(0)?;
            let weekday_idx = weekday_num.parse::<usize>().unwrap_or(0);
            Ok(AverageData {
                label: weekday_names.get(weekday_idx).unwrap_or(&"Unknown").to_string(),
                avg_temperature: row.get(1)?,
                avg_humidity: row.get(2)?,
                count: row.get(3)?,
            })
        })?;

        Ok(result_iter.filter_map(|r| r.ok()).collect())
    }

    pub fn get_monthly_averages(&mut self, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> Result<Vec<AverageData>, rusqlite::Error> {
        let (query, params_vec): (&str, Vec<i64>) = match (start_timestamp, end_timestamp) {
            (Some(start), Some(end)) => {
                ("SELECT strftime('%m', datetime(timestamp, 'unixepoch')) as month, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 WHERE timestamp >= ? AND timestamp <= ?
                 GROUP BY month 
                 ORDER BY month", vec![start, end])
            },
            (Some(start), None) => {
                ("SELECT strftime('%m', datetime(timestamp, 'unixepoch')) as month, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 WHERE timestamp >= ?
                 GROUP BY month 
                 ORDER BY month", vec![start])
            },
            _ => {
                ("SELECT strftime('%m', datetime(timestamp, 'unixepoch')) as month, 
                 AVG(temperature) as avg_temp, AVG(humidity) as avg_hum, COUNT(*) as count
                 FROM Measurement 
                 GROUP BY month 
                 ORDER BY month", vec![])
            }
        };

        let mut stmt = self.connection.prepare(query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        
        let month_names = ["January", "February", "March", "April", "May", "June", 
                          "July", "August", "September", "October", "November", "December"];
        
        let result_iter = stmt.query_map(params_refs.as_slice(), |row| {
            let month_num: String = row.get(0)?;
            let month_idx = month_num.parse::<usize>().unwrap_or(1).saturating_sub(1);
            Ok(AverageData {
                label: month_names.get(month_idx).unwrap_or(&"Unknown").to_string(),
                avg_temperature: row.get(1)?,
                avg_humidity: row.get(2)?,
                count: row.get(3)?,
            })
        })?;

        Ok(result_iter.filter_map(|r| r.ok()).collect())
    }
}
