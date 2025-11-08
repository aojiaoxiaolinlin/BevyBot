use std::env;

use anyhow::Result;
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
use url::Url;

pub struct BskyClient {
    pub client: Client,
    pub_api_url: String,
    api_url: String,
}

impl BskyClient {
    pub fn new() -> Self {
        let client = ClientBuilder::new().build().unwrap();

        Self {
            client,
            pub_api_url: env::var("BSKY_PUB_API_URL").unwrap(),
            api_url: env::var("BSKY_API_URL").unwrap(),
        }
    }

    fn get_url(&self, path: &str) -> String {
        Url::parse(&self.pub_api_url)
            .unwrap()
            .join(path)
            .unwrap()
            .to_string()
    }

    pub async fn get_pub<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.get_url(path);

        let res = self.client.get(url).send().await?;

        Ok(res.json().await?)
    }

    /// 获取帖子详情
    pub async fn get_pub_post_thread<T>(&self, post_uri: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let path = format!("app.bsky.feed.getPostThread?uri={}", post_uri);
        self.get_pub(&path).await
    }
}
