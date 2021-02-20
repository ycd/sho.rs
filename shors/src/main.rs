#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use shortener::shortener::Shortener;
use shortener::url::Url;
mod shortener;
mod storage;
use std::sync::Mutex;

#[derive(Serialize)]
struct Response {
    status_code: i16,
    data: shortener::url::Url,
    error: String,
}
#[derive(Deserialize, Debug)]
struct Shorten {
    url: String,
}

struct SharedShortener {
    url: Mutex<Shortener>,
}

#[post("/shorten", data = "<shorten>")]
fn index<'a>(shorten: Json<Shorten>, shortener: State<'a, SharedShortener>) -> Json<Response> {
    let shared_shortener: &SharedShortener = shortener.inner().clone();
    let url = shared_shortener
        .url
        .lock()
        .unwrap()
        .shorten(&shorten.url)
        .unwrap();

    Json(Response {
        status_code: 200,
        data: url,
        error: String::new(),
    })
}

fn main() {
    let mut shortener: Shortener = shortener::shortener::Shortener::new("shortener");
    rocket::ignite()
        .mount("/", routes![index])
        .manage(SharedShortener {
            url: Mutex::new(shortener),
        })
        .launch();
}
