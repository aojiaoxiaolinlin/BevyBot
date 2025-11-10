use std::env;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::bots::qqbot_client::QQBotClient;


#[derive(Debug, Deserialize)]
pub struct SubChannel {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: u32,
    pub position: u32,
    pub parent_id: String,
    pub owner_id: String,
    pub sub_type: u32,
}


#[derive(Debug, Serialize)]
pub struct PrivateChannel {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: u32,
    pub position: u32,
    pub parent_id: String,
    pub private_type: u32,
    pub private_user_ids: Vec<String>,
    pub sub_type: u32,
    pub application_id: String,
    pub speak_permission: u32
}

impl QQBotClient {
    pub async fn get_sub_channels(&self) -> Result<Vec<SubChannel>> {
        let guild_id = env::var("GUILD_ID")?;

        let data: Vec<SubChannel> = self.get(
            format!("/guilds/{guild_id}/channels")
        ).await?;

        Ok(data)
    }

    pub async fn create_pub_sub_channel(&self, title: &str) -> Result<SubChannel> {
        let guild_id = env::var("GUILD_ID")?;

        let new_sub_channel: SubChannel = self.post(
            format!("/guilds/{}/channels", guild_id),
            PrivateChannel {
                name: title.to_string(),
                type_field: 10007,
                sub_type: 2,
                private_type: 0,
                position: 10,
                parent_id: "".to_string(),
                private_user_ids: Vec::new(),
                application_id: "".to_string(),
                speak_permission: 1
            }
        ).await?;

        Ok(new_sub_channel)
    }
}



#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use crate::bots::qqbot_client::QQBotClient;

    #[tokio::test]
    async fn test_create_sub_channel() {
        dotenv().ok();
        env_logger::init();

        let qq_client = QQBotClient::new_with_default(true).await.unwrap();

        let sub_channel = qq_client.create_pub_sub_channel("测试频道").await.unwrap();

        println!("{:?}", sub_channel);
    }

    #[tokio::test]
    async fn test_get_sub_channels() {
        dotenv().ok();
        env_logger::init();

        let qq_client = QQBotClient::new_with_default(true).await.unwrap();

        let sub_channel = qq_client.get_sub_channels().await.unwrap();

        println!("{:?}", sub_channel);
    }
}
