use harsh::Harsh;
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
};

use storage::storage::Storage;

use crate::storage;

use super::url::Url;

pub struct Shortener {
    pub id: u64,
    pub generator: Harsh,
    pub storage: Storage,
}

impl Shortener {
    pub fn new(db_name: &str) -> Shortener {
        let harsh = Harsh::default();
        let storage = Storage::new(db_name);

        // The default collection is shortener.
        let collection = storage.db.collection("shortener");
        let find_options = FindOptions::builder().sort(doc! {"_id": -1}).build();
        let cursor = collection.find(None, find_options);
        for result in cursor {
            println!("{:?}", result);
        }
        Shortener {
            id: 0,
            generator: harsh,
            storage: storage,
        }
    }

    pub fn next_id(&mut self) -> String {
        let hashed = self.generator.encode(&[self.id]);
        self.id += 1;
        hashed
    }

    pub fn shorten(&mut self, url: &str) -> Result<Url, mongodb::error::Error> {
        let collection = self.storage.db.collection("shortener");
        let url_record = Url::new_record(self.next_id(), String::from(url));

        match collection.insert_one(url_record.to_document(), None) {
            Ok(result) => println!("result: {:#?}", result),
            Err(e) => println!("Some error occured: {}", e),
        }

        Ok(url_record)
    }
}
