mod lib;
use lib::Measurement;
use rusqlite::Connection;
use rocket::serde::json::Json;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> Json<Measurement> {
    let conn = Connection::open(lib::DB_FILE_NAME).unwrap();
    let mut db_service = lib::DbService::new(&conn);
    //TODO conn.close()
    let x = db_service.get_last().unwrap();
    conn.close().unwrap();
    return Json(x);
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}