use actix_rt::spawn;
use anyhow::Result;
use chrono::{Days, Local};
use deepseek_api::{
    CompletionsRequestBuilder, RequestBuilder, request::MessageRequest, response::AssistantMessage,
};
use log::{error, info};
use tokio_schedule::{Job, every};

use crate::{
    bots::{
        deepseek_client::build_deepseek_client, github_client::build_github_client,
        qqbot_client::QQBotClient,
    },
    tasks::github_task::{BEVY_OWNER, BEVY_REPO},
};

pub fn get_new_issuse() -> Result<()> {
    info!("开始定时抓取issue任务");

    // let mut now = Local::now().to_utc();

    let every_day_task = every(1).day().at(12, 00, 00).perform(|| async {
        match run_issue_async_task().await {
            Ok(_) => (),
            Err(err) => {
                error!("{err:?}");
            }
        }
    });

    spawn(every_day_task);

    Ok(())
}

pub async fn run_issue_async_task() -> Result<()> {
    info!("开始任务");

    let spider = build_github_client()?;

    let since = Local::now()
        .to_utc()
        .checked_sub_days(Days::new(1))
        .unwrap();

    let issue_list = spider
        .issues(BEVY_OWNER, BEVY_REPO)
        .list()
        .since(since)
        .send()
        .await?;

    let issue_main_message = issue_list
        .into_iter()
        .map(|issue| {
            format!(
                "标题: {}, 内容: {:?}，发布者名称：{:?}, 时间UTC: {}, 状态: {:?}， 原文链接: {}",
                issue.title,
                issue.body,
                issue.user.name,
                issue.created_at,
                issue.state,
                issue.html_url.to_string()
            )
        })
        .collect::<Vec<_>>();

    // 写入到issue.json 文件
    let issue_json = serde_json::to_string(&issue_main_message)?;
    std::fs::write("issue.json", issue_json)?;

    // 发送到AI进行总结
    let deepseek_client = build_deepseek_client()?;

    let mut chat_messages = vec![];
    chat_messages.push(
        MessageRequest::Assistant(AssistantMessage::new(
            r"你是一个Bevy游戏引擎的社区宣传工作者，你需要根据用户提供的每日的issue列表信息进行分类总结，总结中需要包含issue的标题，内容，发布者名称，时间UTC，状态，原文链接，并进行翻译，对其中的游戏引擎底层原理、图形学等专业知识（术语）进行恰当的解释，挑选比较难的，常用的、都比较了解的略过。
            示例issue：
            假设一个issue：

            标题: Fix memory leak in ECS system
            内容: There is a memory leak when entities are despawned in the ECS. This causes the game to crash after prolonged play.
            发布者: john_doe
            时间: 2023-10-05T12:00:00Z
            状态: open
            链接: https://github.com/bevyengine/bevy/issues/1234

            总结：
            标题: 修复ECS系统中的内存泄漏（翻译）
            内容: 当实体在ECS中被销毁时，存在内存泄漏问题，导致游戏在长时间运行后崩溃。（翻译和详细解释）
            发布者: john_doe
            时间: 2023-10-05 12:00:00 UTC
            状态: 开启
            链接:  [GitHub Issue #xxxx]
            术语解释: ECS（Entity-Component-System）是一种游戏开发架构，用于管理游戏对象（实体）及其属性（组件）和行为（系统）。内存泄漏是指程序在分配内存后未能释放，导致内存使用不断增加。
            对于多个issue，可以列出列表。

            最终响应结构：
                每日Bevy Issue总结
                    总结日期: {{}}年{{}}月{{}}日
                    统计: 共{{}}个issue,每类多少个。
                详细报告：以类别为一个标题开始罗列每个issue的详细信息。分类可以加上Unicode图标。",
        ))
    );
    chat_messages.push(MessageRequest::user(&issue_main_message.join("\n")));

    info!("开始请求AI总结");

    let res = CompletionsRequestBuilder::new(&chat_messages)
        .use_model(deepseek_api::response::ModelType::DeepSeekReasoner)
        .stream(false)
        .max_tokens(8192)
        .unwrap()
        .do_request(&deepseek_client)
        .await;

    info!("AI总结完成");

    let Ok(res) = res else {
        error!("请求失败");
        return Ok(());
    };
    let response = res.must_response();
    info!("{:?}", response);
    let Some(choice) = response.choices.first() else {
        error!("获取choices失败");
        return Ok(());
    };
    let Some(message) = &choice.message else {
        error!("获取message失败");
        return Ok(());
    };
    if !message.content.is_empty() {
        info!("开始发布帖子");
        // 发送到频道
        let qq_client = QQBotClient::new_with_default(false).await?;
        qq_client
            .send_issue_summary("Issues", &message.content)
            .await?;
        info!("帖子发布完成");
    } else {
        error!("文本为空");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use crate::tasks::github_task::{
        BEVY_OWNER, BEVY_REPO, watch_issue_list::run_issue_async_task,
    };

    #[tokio::test]
    async fn test_issue_list() {
        println!("开始测试issue列表获取");
        dotenv().ok();

        let token = std::env::var("GITHUB_PERSON_TOKEN").unwrap();

        println!("测试Token： {}", token);

        let spider = octocrab::Octocrab::builder()
            .personal_token(token)
            .build()
            .unwrap();

        let issue_list = spider
            .issues(BEVY_OWNER, BEVY_REPO)
            .list()
            // .since(since)
            .page(0_u32)
            .per_page(10)
            .send()
            .await
            .unwrap();

        println!("Total Issue: {:?}", issue_list.total_count);

        for issue in issue_list {
            println!("{}", issue.title);
        }
    }

    #[tokio::test]
    async fn test_task() {
        dotenv().ok();
        env_logger::init();

        run_issue_async_task().await.unwrap();
    }
}
