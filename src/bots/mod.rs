use botrs::*;
use log::*;


pub mod qqbot_client;
pub mod qqbot_github_impl;
pub mod qqbot_channel_impl;
pub mod deepseek_client;
pub mod github_client;
pub mod bsky_client;

// 定义机器人的事件处理器
struct MyBot;

#[async_trait::async_trait]
impl EventHandler for MyBot {
    // 当机器人成功连接时调用
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("🤖 机器人已就绪！登录为：{}", ready.user.username);



        let list = ctx.get_guilds(None, None, None).await.unwrap();
        info!("{:?}", list);

        let list = ctx.get_channels("6034175518672956741").await.unwrap();
        // info!("{:?}", list);
        //
        for item in list {
            info!("ID: {:?}, 名称： {:?}", item.id, item.name)
        }

        // let channel_id = "719710382";

        // list.first().unwrap()

        // match ctx. ("", "机器人测试", "测试帖子内容").await {
        //     Ok(thread) => {
        //         println!("论坛话题创建成功: {:?}", thread.thread_id);
        //     }
        //     Err(e) => {
        //         eprintln!("创建论坛话题失败: {}", e);
        //     }
        // }
    }

    // 当有人在消息中提及您的机器人时调用
    async fn message_create(&self, ctx: Context, message: Message) {
        // 忽略来自其他机器人的消息
        if message.is_from_bot() {
            return;
        }

        // 获取消息内容
        let content = match &message.content {
            Some(content) => content,
            None => return,
        };

        info!("📨 收到消息：{}", content);

        // 响应不同的命令
        let response = match content.trim() {
            "!ping" => "🏓 Pong!",
            "!hello" => "👋 你好！",
            "!help" => "🤖 可用命令：!ping, !hello, !help, !about",
            "!about" => "🦀 我是用 BotRS 构建的 QQ 机器人 - 一个用于 QQ 频道机器人的 Rust 框架！",
            _ => return, // 不回应其他消息
        };

        // 发送回复
        match message.reply(&ctx.api, &ctx.token, response).await {
            Ok(_) => info!("✅ 回复发送成功"),
            Err(e) => warn!("❌ 发送回复失败：{}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use botrs::{Client, Intents, Token};
    use dotenvy::dotenv;

    use crate::bots::MyBot;

    #[tokio::test]
    async fn test_mod_send() {
        println!("开始测试机器人发帖");
        dotenv().ok();
        env_logger::init();

        let app_id = std::env::var("QQ_BOT_APP_ID")
                .expect("未设置 QQ_BOT_APP_ID 环境变量");
        let secret = std::env::var("QQ_BOT_SECRET")
            .expect("未设置 QQ_BOT_SECRET 环境变量");

        let token = Token::new(app_id, secret);

        // 配置机器人想要接收的事件
        let intents = Intents::default()
            .with_public_guild_messages()  // 接收 @ 提及
            .with_guilds();                // 接收频道事件

        // 创建机器人客户端
        let mut client = Client::new(token, intents, MyBot, true).unwrap();

        println!("🔌 连接到 QQ 频道...");

        // 启动机器人（这将运行直到程序停止）
        client.start().await.unwrap();
    }
}
