use rocket::{
    request::{FromRequest, Outcome},
    Request,
};
use std::{collections::HashMap, convert::Infallible};

// Get request headers for any
// incoming HTTP requests.
#[derive(Debug)]
pub struct RequestHeaders {
    pub headers: HashMap<String, String>,
}

impl<'a, 'r> FromRequest<'a, 'r> for RequestHeaders {
    type Error = Infallible;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let headers: HashMap<String, String> = request
            .headers()
            .iter()
            .map(|h| (h.name().to_string(), h.value().to_string()))
            .collect();

        rocket::Outcome::Success(Self { headers: headers })
    }
}
