#![feature(proc_macro_hygiene, decl_macro)]
#![feature(option_unwrap_none)]

#[macro_use]
extern crate rocket;
extern crate woothee;

use log::info;
use response::Redirect;
use rocket::{response, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use utils::types::RequestHeaders;

use shortener::{
    shortener::Analytics,
    shortener::{AnalyticResults, Shortener},
    url::Url,
};
mod shortener;
mod storage;
mod utils;

use std::{net::SocketAddr, sync::Mutex};

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

    // Use 301 Moved Permanently as status code
    // to don't hurt website's SEO.
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
        status_code: 201,
        data: Some(url),
        error: String::new(),
    })
}

#[get("/api/<id>")]
fn get_analytics_of_id<'a>(
    id: String,
    shortener: State<'a, SharedShortener>,
) -> Json<AnalyticResults> {
    let shared_shortener: &SharedShortener = shortener.inner().clone();
    let analytics = shared_shortener.url.lock().unwrap().get_analytics(id);

    Json(analytics)
}

#[get("/<id>")]
fn redirect<'a>(
    id: String,
    shortener: State<'a, SharedShortener>,
    headers: RequestHeaders,
    client_ip: SocketAddr,
) -> ResponseOrRedirect {
    // FIXME: this is for debugging purposes, should be deleted later.
    info!("Got new request from {:?} to id: {}", client_ip, id);

    let shared_shortener: &SharedShortener = shortener.inner();
    let response: ResponseOrRedirect = match shared_shortener
        .url
        .lock()
        .unwrap()
        .get_original_url(id.clone())
    {
        Some(url) => ResponseOrRedirect::Redirect(Redirect::to(url)),
        None => ResponseOrRedirect::Response(Json(ShortenerResponse {
            status_code: 404,
            data: None,
            error: String::from("No URL found."),
        })),
    };

    match response {
        ResponseOrRedirect::Response(_) => {}
        ResponseOrRedirect::Redirect(_) => {
            match crossbeam::thread::scope(|scope| {
                scope.spawn(move |_| {
                    shared_shortener
                        .url
                        .try_lock()
                        .unwrap()
                        .process_analytics(Analytics::new(
                            id.clone(),
                            headers.headers,
                            client_ip.to_string(),
                        ));
                });
            }) {
                Ok(_) => info!("successfully proccessed analytics"),
                Err(e) => log::error!("error occured: {:#?}", e),
            }
        }
    };

    response
}

fn main() {
    let shortener: Shortener = shortener::shortener::Shortener::new("shortener");
    rocket::ignite()
        .mount("/", routes![index, redirect, get_analytics_of_id])
        .manage(SharedShortener {
            url: Mutex::new(shortener),
        })
        .launch();
}
