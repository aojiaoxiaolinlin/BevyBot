use std::env;

use reqwest::{Client, ClientBuilder};


pub struct BskyClient {
    client: Client,
    pub_api_url: String,
    api_url: String
}

impl BskyClient {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .build()
            .unwrap();

        Self {
            client,
            pub_api_url: env::var("BSKY_PUB_API_URL").unwrap(),
            api_url: env::var("BSKY_API_URL").unwrap(),
        }
    }
}
