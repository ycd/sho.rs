use dotenv;

#[derive(Debug)]
pub struct Config {
    pub server_ip: String,
    pub dbname: String,
    pub username: String,
    pub password: String,
}

impl Config {
    // Create a new Config for your MongoDB,
    // Get's server_ip, dbname, username, password
    // from the ".env" file
    pub fn new() -> Result<Config, Box<dyn std::error::Error>> {
        // Get values from .env file if default
        match dotenv::dotenv().ok() {
            Some(p) => Some(p),
            None => dotenv::from_filename(".env").ok(),
        };

        let server_ip = dotenv::var("MONGO_SERVER_IP")?;
        let dbname = dotenv::var("MONGO_DBNAME")?;

        let username = dotenv::var("MONGO_USERNAME")?;
        let password = dotenv::var("MONGO_PASSWORD")?;

        Ok(Config {
            server_ip: server_ip,
            dbname: dbname,
            username: username,
            password: password,
        })
    }

    pub fn uri(&self) -> String {
        format!(
            "mongodb+srv://{}:{}@{}/{}?retryWrites=true&w=majority",
            self.username, self.password, self.server_ip, self.dbname
        )
    }
}
