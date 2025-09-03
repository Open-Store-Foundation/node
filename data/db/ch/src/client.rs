use clickhouse::Client;
use tracing::info;

pub struct ChClient {
    pub client: Client,
}

impl ChClient {

    pub fn new_client(
        url: String,
        database: Option<String>,
        user: Option<String>,
        password: Option<String>,
    ) -> ChClient {
        info!("Configuring ClickHouse client for URL: {}", url);

        let mut builder = Client::default()
            .with_url(url);

        if let Some(db) = database {
            info!("Using ClickHouse database: {}", db);
            builder = builder.with_database(db);
        }

        if let Some(u) = user {
            info!("Using ClickHouse user: {}", u);
            builder = builder.with_user(u);
        }

        if let Some(p) = password {
            builder = builder.with_password(p);
            info!("Using ClickHouse password (set)");
        }

        let client = builder;

        ChClient { client }
    }
}