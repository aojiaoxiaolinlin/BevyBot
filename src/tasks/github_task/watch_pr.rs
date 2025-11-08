use crate::{
    bots::{
        deepseek_client::build_deepseek_client, github_client::build_github_client,
        qqbot_client::QQBotClient,
    },
    tasks::github_task::{BEVY_OWNER, BEVY_REPO},
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
    info!("开始定时抓取pr任务");

    // let mut now = Local::now().to_utc();

    let every_day_task = every(1).day().at(12, 00, 00)
        .perform(|| async {
            let spider = build_github_client().unwrap();

            let since = Local::now().to_utc().checked_sub_days(Days::new(1)).unwrap();

            let issue_list = spider
                .repos(BEVY_OWNER, BEVY_REPO)
                .list_commits()
                .since(since)
                .send()
                .await
                .unwrap();

            // 发送到AI进行总结
            let deepseek_client = build_deepseek_client().unwrap();

            let mut all_issue = issue_list.into_iter().map(|issue| {
                MessageRequest::user(
                    &format!("Commit Message内容: {:?}，发布者名称：{:?}, 文件更改列表：{:?}, 原文链接: {}",
                    issue.commit.message, issue.author, issue.files, issue.html_url)
                )
            }).collect::<Vec<_>>();

            all_issue.push(
                MessageRequest::user(
                    "上面是每日的Commits内容，是关于Bevy游戏引擎的，结合Bevy游戏引擎的背景，对上面的Commits列表使用中文进行总结和提炼，对于一些涉及引擎，图形学的专业术语，可以简单的附上解释，解释该功能的效果或者作用或者原理（根据术语来决定如何解释），请一定要使用Markdown格式进行回复所有的内容，记得原封不动的附上原文的链接地址，让用户可以点击查看。直接回复总结的内容即可，省略开头和结尾的客套话。"
                )
            );

            let res = CompletionsRequestBuilder::new(&all_issue)
                .use_model(DeepSeekReasoner)
                .stream(false)
                .do_request(&deepseek_client)
                .await;

            if let Ok(res) = res 
                && let Some (choice) = res.must_response().choices.first() 
                && let Some(message) = &choice.message 
                && !message.content.is_empty() {
                    // 发送到频道
                    let qq_client = match QQBotClient::new_with_default(false).await {
                        Ok(qq_client) => qq_client,
                        Err(err) => {
                            error!("{err:?}");
                            return ;
                        }
                    };

                    match qq_client.send_commit_summary("Commits", &message.content).await {
                        Ok(_) => (),
                        Err(err) => {
                            error!("{err:?}");
                        }
                    };
                
            }
        });

    spawn(every_day_task);

    Ok(())
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use crate::tasks::github_task::{BEVY_OWNER, BEVY_REPO};

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
