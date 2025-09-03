use std::time::Duration;

use reqwest::Client;

pub struct HttpProviderFactory;
pub type HttpClient = Client;

impl HttpProviderFactory {

    pub fn client() -> reqwest::Result<Client> {
        let id_release = !cfg!(debug_assertions); // TODO v2 to env
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .https_only(id_release)
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(1)
            .connection_verbose(false)
            .build();

        return client;
    }

    pub fn http_client() -> reqwest::Result<Client> {
        // let retry_policy = ExponentialBackoff::builder()
        //     .retry_bounds(Duration::from_secs(3), Duration::from_secs(30))
        //     .build_with_max_retries(2);

        let client = HttpProviderFactory::client();

        // let builder = ClientBuilder::new(client)
        //     .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        //     .build();

        return client;
    }
}
