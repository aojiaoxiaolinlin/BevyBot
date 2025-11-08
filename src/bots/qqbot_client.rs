use std::env;

use anyhow::Result;
use reqwest::{Client, ClientBuilder};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use url::Url;

use crate::{
    AppState,
    util::cache::{get, put_ttl},
};

const ACCESS_TOKEN_KEY_NAME: &str = "QQ_BOT_ACCESS_TOKEN";

pub struct QQBotClient {
    pub client: Client,
    pub base_url: String,
}

impl QQBotClient {
    pub async fn new_with_default(sandbox: bool) -> Result<Self> {
        let redis_url = env::var("REDIS").expect("请配置Redis链接");
        let redis_client = redis::Client::open(redis_url).expect("连接Redis失败");

        QQBotClient::new(
            &AppState {
                mysql: DatabaseConnection::default(),
                redis: redis_client,
            },
            sandbox,
        )
        .await
    }

    pub async fn new(state: &AppState, sandbox: bool) -> Result<Self> {
        Ok(Self {
            client: build_qq_bot_client(state).await?,
            base_url: if sandbox {
                "https://sandbox.api.sgroup.qq.com/".to_string()
            } else {
                "https://api.sgroup.qq.com/".to_string()
            },
        })
    }

    fn get_url(&self, path: String) -> String {
        Url::parse(&self.base_url)
            .unwrap()
            .join(&path)
            .unwrap()
            .to_string()
    }

    pub async fn post<T>(&self, path: String, data: impl Serialize) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.get_url(path);

        let res = self.client.post(url).json(&data).send().await?;

        Ok(res.json().await?)
    }

    pub async fn get<T>(&self, path: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.get_url(path);

        let res = self.client.get(url).send().await?;

        Ok(res.json().await?)
    }

    pub async fn put<T>(&self, path: String, data: impl Serialize) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.get_url(path);

        let res = self.client.put(url).json(&data).send().await?;

        Ok(res.json().await?)
    }

    pub async fn patch<T>(&self, path: String, data: impl Serialize) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.get_url(path);

        let res = self.client.patch(url).json(&data).send().await?;

        Ok(res.json().await?)
    }

    pub async fn delete<T>(&self, path: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.get_url(path);

        let res = self.client.delete(url).send().await?;

        Ok(res.json().await?)
    }
}

pub async fn build_qq_bot_client(state: &AppState) -> Result<Client> {
    let token = fetch_access_token(state).await?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", format!("QQBot {}", token).parse()?);

    Ok(ClientBuilder::new().default_headers(headers).build()?)
}

#[derive(Deserialize)]
struct QQBotAccessToeknRes {
    access_token: String,
    expires_in: String,
}

async fn fetch_access_token(state: &AppState) -> Result<String> {
    let app_id = std::env::var("QQ_BOT_APP_ID").expect("未设置 QQ_BOT_APP_ID 环境变量");
    let secret = std::env::var("QQ_BOT_SECRET").expect("未设置 QQ_BOT_SECRET 环境变量");

    let token = get(&state.redis, ACCESS_TOKEN_KEY_NAME).await?;

    let token = if let Some(token) = token {
        token
    } else {
        let client = reqwest::Client::new();
        let res = client
            .post("https://bots.qq.com/app/getAppAccessToken")
            .json(&json!({
                "appId": app_id,
                "clientSecret": secret
            }))
            .send()
            .await?;

        let data = res.json::<QQBotAccessToeknRes>().await?;

        put_ttl(
            &state.redis,
            ACCESS_TOKEN_KEY_NAME,
            &data.access_token,
            data.expires_in.parse()?,
        )
        .await?;

        data.access_token.clone()
    };

    Ok(token)
}

#[cfg(test)]
mod tests {
    use std::env;

    use dotenvy::dotenv;
    use log::info;
    use sea_orm::DatabaseConnection;
    use serde_json::json;

    use crate::{AppState, bots::qqbot_client::QQBotClient};

    #[tokio::test]
    async fn create_thread() {
        dotenv().ok();
        env_logger::init();

        let redis_url = env::var("REDIS").expect("请配置Redis链接");
        let redis_client = redis::Client::open(redis_url).expect("连接Redis失败");

        let app_state = AppState {
            redis: redis_client,
            mysql: DatabaseConnection::default(),
        };

        let qq_client = QQBotClient::new(&app_state, true).await.unwrap();

        let res: serde_json::Value = qq_client.put(
            format!("/channels/{}/threads", 719710382),
            json!({
                "title": "title",
                "content": "<html lang=\"en-US\"><body><a href=\"https://bot.q.qq.com/wiki\" title=\"QQ机器人文档Title\">QQ机器人文档</a>\n<ul><li>主动消息：发送消息时，未填msg_id字段的消息。</li><li>被动消息：发送消息时，填充了msg_id字段的消息。</li></ul></body></html>",
                "format": 2
            })
        )
        .await
        .unwrap();

        info!("{:?}", res);
    }
}
