mod lib;
use lib::{Measurement, AverageData};
use rusqlite::Connection;
use rocket::serde::json::Json;
use rocket::State;

#[macro_use] extern crate rocket;
#[get("/current")]
fn current() -> Json<Measurement> {
    let conn = Connection::open(lib::DB_FILE_NAME).unwrap();
    let mut db_service = lib::DbService::new(&conn);
    //TODO conn.close()
    let x = db_service.get_last().unwrap();
    conn.close().unwrap();
    return Json(x);
}

#[get("/today")]
fn today() -> Json<Vec<Measurement>> {
    let conn = Connection::open(lib::DB_FILE_NAME).unwrap();
    let mut db_service = lib::DbService::new(&conn);
    //TODO conn.close()
    let x = db_service.get_by_start_of_day().unwrap();
    conn.close().unwrap();
    return Json(x);
}

use rocket::fs::NamedFile;
use rocket::get;

#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("html/index.html").await
}

#[get("/averages/hourly?<start>&<end>")]
fn hourly_averages(start: Option<i64>, end: Option<i64>) -> Json<Vec<AverageData>> {
    let conn = Connection::open(lib::DB_FILE_NAME).unwrap();
    let mut db_service = lib::DbService::new(&conn);
    let result = db_service.get_hourly_averages(start, end).unwrap_or_else(|_| vec![]);
    conn.close().unwrap();
    Json(result)
}

#[get("/averages/weekday?<start>&<end>")]
fn weekday_averages(start: Option<i64>, end: Option<i64>) -> Json<Vec<AverageData>> {
    let conn = Connection::open(lib::DB_FILE_NAME).unwrap();
    let mut db_service = lib::DbService::new(&conn);
    let result = db_service.get_weekday_averages(start, end).unwrap_or_else(|_| vec![]);
    conn.close().unwrap();
    Json(result)
}

#[get("/averages/monthly?<start>&<end>")]
fn monthly_averages(start: Option<i64>, end: Option<i64>) -> Json<Vec<AverageData>> {
    let conn = Connection::open(lib::DB_FILE_NAME).unwrap();
    let mut db_service = lib::DbService::new(&conn);
    let result = db_service.get_monthly_averages(start, end).unwrap_or_else(|_| vec![]);
    conn.close().unwrap();
    Json(result)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, today, current, hourly_averages, weekday_averages, monthly_averages])
}