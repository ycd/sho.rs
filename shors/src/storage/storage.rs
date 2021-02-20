use super::config;
use mongodb::sync::{Client, Database};

// Storage represents mongodb client and the config for it.
pub struct Storage {
    pub config: config::Config,
    pub client: mongodb::sync::Client,
    pub db: Database,
}

impl Storage {
    pub fn new(db_name: &str) -> Storage {
        let config = config::Config::new().unwrap();
        let config_uri = config.uri();
        let client = Client::with_uri_str(config_uri.as_str()).unwrap();
        let db = client.database(db_name);

        Storage {
            config: config,
            client: client,
            db: db,
        }
    }

    pub fn database<'a>(&self, db_name: &'a str) -> Result<Database, Box<dyn std::error::Error>> {
        Ok(self.client.database(db_name))
    }
}
