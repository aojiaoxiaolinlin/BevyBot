use actix_rt::spawn;
use deepseek_api::response::ModelType::DeepSeekReasoner;
use anyhow::Result;
use chrono::{Days, Local};
use deepseek_api::{CompletionsRequestBuilder, RequestBuilder, request::MessageRequest};
use log::{error, info};
use octocrab::{Octocrab, models::pulls::PullRequest};
use tokio_schedule::{Job, every};

use crate::{bots::{deepseek_client::build_deepseek_client, github_client::build_github_client, qqbot_client::QQBotClient}, tasks::github_task::{BEVY_OWNER, BEVY_REPO, get_first_deepseek_response}};



pub fn get_new_prs() -> Result<()> {
    info!("开始定时抓取prs任务");

    // let mut now = Local::now().to_utc();

    let every_day_task = every(1).day().at(12, 00, 00)
        .perform(|| async {
            if let Err(err) = run_pr_task().await {
                error!("{err:?}");
            }
        });

    spawn(every_day_task);

    Ok(())
}

pub async fn run_pr_task() -> Result<()> {
    let spider = build_github_client()?;

    let pr_list = get_latest_pr_list(&spider).await?;

    let deepseek_client = build_deepseek_client()?;

    let mut all_issue = pr_list.iter().map(|pr| {
        MessageRequest::user(
            &format!("PR 内容: {:?}，发布者名称：{:?}, PR 标题：{:?}, 原文链接: {:?}",
            pr.body, pr.user, pr.title, pr.html_url)
        )
    }).collect::<Vec<_>>();


    all_issue.push(
        MessageRequest::user(
            "你是一个Bevy游戏引擎的社区宣传工作者，你需要根据用户提供的每日的PRs列表信息进行分类总结，总结中需要包含PRs的标题，内容，发布者名称，时间UTC，状态，原文链接，并进行翻译，对其中的游戏引擎底层原理、图形学等专业知识（术语）进行恰当的解释，挑选比较难的，常用的、都比较了解Commit略过。
            示例PR：
            假设一个PR：

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
            链接:  [GitHub PR #xxxx](原文链接)
            术语解释: ECS（Entity-Component-System）是一种游戏开发架构，用于管理游戏对象（实体）及其属性（组件）和行为（系统）。内存泄漏是指程序在分配内存后未能释放，导致内存使用不断增加。
            对于多个PR，可以列出列表。"
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
        qq_client.send_commit_summary("PRs", &text).await?;
    }

    Ok(())
}


pub async fn get_latest_pr_list(
    spider: &Octocrab
) -> Result<Vec<PullRequest>> {

    let pr_list = spider.pulls(BEVY_OWNER, BEVY_REPO)
        .list()
        .state(octocrab::params::State::All)
        .send()
        .await?;

    let last_day = Local::now().checked_sub_days(Days::new(1)).unwrap().to_utc();

    // 筛选今日的
    let latest_pr_list = pr_list.items.iter().filter(|pr| {
        if let Some(created_at) = pr.created_at {
            if created_at > last_day {
                return true;
            }
        }

        false
    })
    .cloned()
    .collect::<Vec<_>>();

    if latest_pr_list.is_empty() {
        anyhow::bail!("今日Pr列表为空");
    }

    Ok(latest_pr_list)
}



#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use crate::tasks::github_task::watch_pr::get_latest_pr_list;

    #[tokio::test]
    async fn test_get_latest_pr_list() {
        dotenv().ok();

        let token = std::env::var("GITHUB_PERSON_TOKEN").unwrap();
        let spider = octocrab::Octocrab::builder()
            .personal_token(token)
            .build()
            .unwrap();

        let pr_list = get_latest_pr_list(&spider).await.unwrap();

        println!("{:?}", pr_list);
    }
}
