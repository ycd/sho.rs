#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use response::Redirect;
use rocket::{response, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use shortener::{shortener::Shortener, url::Url};
mod shortener;

mod storage;
use std::sync::Mutex;

#[derive(Debug, Serialize)]
struct ShortenerResponse {
    status_code: i16,
    data: Option<Url>,
    error: String,
}
#[derive(Deserialize, Debug)]
struct Shorten {
    url: String,
}

struct SharedShortener {
    url: Mutex<Shortener>,
}

#[derive(Debug, Responder)]
enum ResponseOrRedirect {
    Response(Json<ShortenerResponse>),
    #[response(status = 301)]
    Redirect(Redirect),
}

#[post("/api/shorten", data = "<shorten>")]
fn index<'a>(
    shorten: Json<Shorten>,
    shortener: State<'a, SharedShortener>,
) -> Json<ShortenerResponse> {
    let shared_shortener: &SharedShortener = shortener.inner().clone();
    let url = shared_shortener
        .url
        .lock()
        .unwrap()
        .shorten(&shorten.url)
        .unwrap();

    Json(ShortenerResponse {
        status_code: 200,
        data: Some(url),
        error: String::new(),
    })
}

#[get("/<id>")]
fn redirect<'a>(id: String, shortener: State<'a, SharedShortener>) -> ResponseOrRedirect {
    let shared_shortener: &SharedShortener = shortener.inner().clone();

    let response: ResponseOrRedirect =
        match shared_shortener.url.lock().unwrap().get_original_url(id) {
            Some(url) => ResponseOrRedirect::Redirect(Redirect::to(url)),
            None => ResponseOrRedirect::Response(Json(ShortenerResponse {
                status_code: 404,
                data: None,
                error: String::from("No URL found."),
            })),
        };

    response
}

fn main() {
    let shortener: Shortener = shortener::shortener::Shortener::new("shortener");
    rocket::ignite()
        .mount("/", routes![index, redirect])
        .manage(SharedShortener {
            url: Mutex::new(shortener),
        })
        .launch();
}
