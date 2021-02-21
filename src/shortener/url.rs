use mongodb::bson::{doc, to_bson, Document};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Url {
    pub archived: bool,
    pub created_at: String,
    pub id: String,
    pub link: String,
    pub long_url: String,
}

impl Url {
    pub fn new_record(id: String, long_url: String) -> Url {
        Url {
            created_at: chrono::Utc::now().to_rfc2822(),
            id: String::from(&id),
            archived: false,
            long_url: long_url,
            link: format!("https://sho.rs/{}", String::from(&id)),
        }
    }

    pub fn to_document(&self) -> Document {
        to_bson(&self).unwrap().as_document().unwrap().clone()
    }
}
