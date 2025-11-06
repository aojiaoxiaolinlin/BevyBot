use std::env;

use crate::bots::qqbot_client::QQBotClient;
use anyhow::Result;
use chrono::Local;
use log::info;
use serde_json::json;


impl QQBotClient {
    pub async fn send_thread(&self, title: &str, text: &String, sub_channel_id: &str) -> Result<()> {
        let res: serde_json::Value = self.put(
            format!("/channels/{}/threads", sub_channel_id),
            json!({
                "title": title,
                "content": text,
                "format": 3
            })
        ).await?;

        info!("{:?}", res);

        Ok(())
    }

    pub async fn send_issue_summary(&self, title: &str, text: &String) -> Result<()> {
        let title = format!("每日 {} 总结：{}", title, Local::now().format("%Y-%m-%d"));

        let res: serde_json::Value = self.put(
            format!("/channels/{}/threads", env::var("ISSUE_CHANNEL_ID").unwrap_or_default()),
            json!({
                "title": title,
                "content": text,
                "format": 3
            })
        )
        .await
        ?;

        info!("{:?}", res);

        Ok(())
    }

    pub async fn send_commit_summary(&self, title: &str, text: &String) -> Result<()> {
        let title = format!("每日 {} 总结：{}", title, Local::now().format("%Y-%m-%d"));

        let res: serde_json::Value = self.put(
            format!("/channels/{}/threads", env::var("COMMINT_CHANNEL_ID").unwrap_or_default()),
            json!({
                "title": title,
                "content": text,
                "format": 3
            })
        )
        .await
        ?;

        info!("{:?}", res);

        Ok(())
    }

}
