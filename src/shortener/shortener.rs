use std::collections::HashMap;

use harsh::Harsh;
use mongodb::{bson::doc, options::UpdateOptions};

use log::{error, info};
use storage::storage::Storage;

use super::url::Url;
use crate::storage;
use mongodb::bson::{to_bson, Document};
use serde::Serialize;

pub struct Shortener {
    pub id: u64,
    pub generator: Harsh,
    pub storage: Storage,
}

impl Shortener {
    pub fn new(db_name: &str) -> Shortener {
        let harsh = Harsh::default();
        let storage = Storage::new(db_name);

        // To create unique id every time we need an atomic
        // counter to get or restore id from there
        //
        // It follows these steps
        // 1- check mongodb for the key
        //    1.2- if None found
        //        1.3 - Create new object and return 0
        // 2- return the current count.
        let collection = storage.db.collection("counter");
        let id: u64 = match collection.find_one(doc! {"name": "counter"}, None).unwrap() {
            Some(document) => document.get_i64("count").unwrap() as u64,
            None => match collection.insert_one(
                doc! {
                    "name": "counter",
                    "count": 0 as u64,
                },
                None,
            ) {
                Ok(res) => {
                    info!("successfully created count {:#?}", res);
                    0
                }
                Err(e) => {
                    error!("error occured, creating counter {:#?}", e);
                    panic!("counter creation failed, exiting");
                }
            },
        };

        Shortener {
            id: id,
            generator: harsh,
            storage: storage,
        }
    }

    pub fn next_id(&mut self) -> String {
        let hashed = self.generator.encode(&[self.id]);
        let _ = match self.increment_counter() {
            Ok(_) => self.id += 1,
            Err(e) => error!("error occured, calling next_id : {}", e),
        };

        hashed
    }

    pub fn get_original_url(&self, id: String) -> Option<String> {
        let collection = self.storage.db.collection("shortener");

        let original_url: Option<String> =
            match collection.find_one(doc! {"id": &id}, None).unwrap() {
                Some(document) => Some(document.get_str("long_url").unwrap().to_string()),
                None => {
                    info!("no document found for id={}", &id);
                    None
                }
            };

        original_url
    }

    fn increment_counter(&self) -> Result<(), mongodb::error::Error> {
        let collection = self.storage.db.collection("counter");

        match collection.update_one(doc! {"name": "counter"}, doc! {"$inc": {"count": 1}}, None) {
            Ok(result) => info!("successfully incremented counter: {:#?}", result),
            Err(e) => error!("error occured, incrementing atomic counter: {}", e),
        };

        Ok(())
    }

    pub fn shorten(&mut self, url: &str) -> Result<Url, mongodb::error::Error> {
        let collection = self.storage.db.collection("shortener");

        // Create new URL record from the input URL.
        let url_record = Url::new_record(self.next_id(), String::from(url));

        match collection.insert_one(url_record.to_document(), None) {
            Ok(result) => info!("successfully shortened url: {:#?}", result),
            Err(e) => error!("error occured, inserting shortened url: {}", e),
        }

        Ok(url_record)
    }
}

#[derive(Debug, Serialize)]
pub struct Analytics {
    pub id: String,
    pub headers: HashMap<String, String>,
    pub ip: String,
    time: chrono::DateTime<chrono::Utc>,
}

impl Analytics {
    pub fn new(id: String, headers: HashMap<String, String>, ip: String) -> Analytics {
        Analytics {
            id: id,
            headers: headers,
            ip: ip,
            time: chrono::Utc::now(),
        }
    }

    pub fn to_document(&self) -> Document {
        to_bson(&self).unwrap().as_document().unwrap().clone()
    }
}

impl Shortener {
    pub fn process_analytics(&self, analytics: Analytics) {
        let analytics_db = self.storage.db.collection("analytics");

        match analytics_db.insert_one(analytics.to_document(), None) {
            Ok(res) => info!("result from analytics process {:#?}", res),
            Err(e) => error!("error occured, analytics process {:#?}", e),
        };
        println!("{:#?}", analytics.to_document());
    }

    pub fn get_analytics(&self, id: String) {
        let analytics_db = self.storage.db.collection("analytics");
    }
}
