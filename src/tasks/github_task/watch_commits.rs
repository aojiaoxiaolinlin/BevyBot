use crate::{
    bots::{
        deepseek_client::build_deepseek_client, github_client::build_github_client,
        qqbot_client::QQBotClient,
    },
    tasks::github_task::{BEVY_OWNER, BEVY_REPO, get_first_deepseek_response},
};
use actix_rt::spawn;
use anyhow::Result;
use chrono::{Days, Local};
use deepseek_api::{
    CompletionsRequestBuilder, RequestBuilder, request::MessageRequest,
    response::ModelType::DeepSeekReasoner,
};
use log::{error, info};
use tokio_schedule::{Job, every};

pub fn get_new_commits() -> Result<()> {
    info!("开始定时抓取commits任务");

    // let mut now = Local::now().to_utc();

    let every_day_task = every(1).day().at(12, 00, 00)
        .perform(|| async {
            if let Err(err) = run_commits_task().await {
                error!("{err:?}");
            }
        });

    spawn(every_day_task);

    Ok(())
}


pub async fn run_commits_task() -> Result<()> {
    let spider = build_github_client()?;

    let since = Local::now().to_utc().checked_sub_days(Days::new(1)).unwrap();

    let issue_list = spider
        .repos(BEVY_OWNER, BEVY_REPO)
        .list_commits()
        .since(since)
        .send()
        .await?;

    if issue_list.items.is_empty() {
        anyhow::bail!("今日Commits为空");
    }

    // 发送到AI进行总结
    let deepseek_client = build_deepseek_client()?;

    let mut all_issue = issue_list.into_iter().map(|issue| {
        MessageRequest::user(
            &format!("Commit Message内容: {:?}，发布者名称：{:?}, 文件更改列表：{:?}, 原文链接: {}",
            issue.commit.message, issue.author, issue.files, issue.html_url)
        )
    }).collect::<Vec<_>>();

    all_issue.push(
        MessageRequest::user(
            "你是一个Bevy游戏引擎的社区宣传工作者，你需要根据用户提供的每日的Commits列表信息进行分类总结，总结中需要包含Commits的标题，内容，发布者名称，时间UTC，状态，原文链接，并进行翻译，对其中的游戏引擎底层原理、图形学等专业知识（术语）进行恰当的解释，挑选比较难的，常用的、都比较了解Commit略过。
            示例Commit：
            假设一个Commit：

            标题: Fix memory leak in ECS system
            内容: There is a memory leak when entities are despawned in the ECS. This causes the game to crash after prolonged play.
            发布者: john_doe
            时间: 2023-10-05T12:00:00Z
            链接: https://github.com/bevyengine/bevy/commit/2facb2572d84e9b9923edef7f35bae2c26308081

            总结：
            标题: 修复ECS系统中的内存泄漏（翻译）
            内容: 当实体在ECS中被销毁时，存在内存泄漏问题，导致游戏在长时间运行后崩溃。（翻译和详细解释）
            发布者: john_doe
            时间: 2023-10-05 12:00:00 UTC
            状态: 开启
            链接:  [GitHub Commit #xxxx](原文链接)
            术语解释: ECS（Entity-Component-System）是一种游戏开发架构，用于管理游戏对象（实体）及其属性（组件）和行为（系统）。内存泄漏是指程序在分配内存后未能释放，导致内存使用不断增加。
            对于多个Commit，可以列出列表。"
        )
    );

    let res = CompletionsRequestBuilder::new(&all_issue)
        .use_model(DeepSeekReasoner)
        .stream(false)
        .do_request(&deepseek_client)
        .await?;

    let text = get_first_deepseek_response(res)?;

    if !text.is_empty() {
        // 发送到频道
        let qq_client = QQBotClient::new_with_default(false).await?;
        qq_client.send_commit_summary("Commits", &text).await?;
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use crate::tasks::github_task::{BEVY_REPO, BEVY_OWNER};

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
}
