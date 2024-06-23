mod lib;
use lib::Measurement;
use rusqlite::Connection;
use rocket::serde::json::Json;

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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, today, current])
}