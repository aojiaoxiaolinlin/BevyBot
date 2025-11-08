use std::{sync::Arc, time::Instant};

use actix_rt::spawn;
use anyhow::Result;
use deepseek_api::{
    CompletionsRequestBuilder, DeepSeekClient, RequestBuilder,
    request::{MessageRequest, SystemMessageRequest},
};
use log::{debug, error, info};
use octocrab::{
    Octocrab, Page,
    models::{Milestone, issues::Issue},
};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, QueryFilter,
};
use tokio_schedule::{Job, every};

use crate::{
    AppState,
    bots::{
        deepseek_client::build_deepseek_client, github_client::build_github_client,
        qqbot_client::QQBotClient,
    },
    tasks::github_task::{BEVY_OWNER, BEVY_REPO, get_first_deepseek_response},
};

const MAX_PER_PAGE: u8 = 100;

pub fn spawn_milestone_task(app_state: AppState) -> Result<()> {
    info!("开始定时抓取milestone任务");

    // let mut now = Local::now().to_utc();

    let app_state = Arc::new(app_state);

    let every_day_task = every(1).day().at(13, 00, 00).perform(move || {
        let state = app_state.as_ref().clone();
        async {
            if let Err(err) = get_changed_milestone(state).await {
                error!("{err:?}");
            }
        }
    });

    spawn(every_day_task);

    Ok(())
}

pub async fn get_changed_milestone(app_state: AppState) -> Result<()> {
    let spider = build_github_client()?;
    let deepseek_client = build_deepseek_client()?;
    let qq_client = QQBotClient::new_with_default(false).await?;

    // 读取子频道列表
    let sub_channels = qq_client.get_sub_channels().await?;

    let milestone_list = get_milestone_list(&spider).await?;

    for milestone in milestone_list {
        let milestone_title = milestone.title;
        let milestone_id = milestone.number;

        // 是否需要创建里程碑子频道
        let sub_channel_name = get_channel_name(&milestone_title);
        let mut need_create = true;
        let mut target_sub_channel_id = "".to_string();
        for sub_channel in &sub_channels {
            if sub_channel.name == sub_channel_name {
                need_create = false;
                target_sub_channel_id = sub_channel.id.clone();
                break;
            }
        }

        if need_create {
            let sub_channel = qq_client.create_pub_sub_channel(&sub_channel_name).await?;
            target_sub_channel_id = sub_channel.id;
        }

        let mut cur_page = 0_u32;

        // 记录该milestone所有的issue列表
        let mut milestone_all_issues = Vec::new();

        loop {
            // 根据里程碑ID获取issue列表
            let issue_list = spider
                .issues(BEVY_OWNER, BEVY_REPO)
                .list()
                .state(octocrab::params::State::All)
                .milestone(milestone_id as u64)
                .page(cur_page)
                .per_page(MAX_PER_PAGE)
                .send()
                .await?;

            let total_issue_this_page = issue_list.items.len();

            for issue in issue_list {
                milestone_all_issues.push(*issue.id);

                // 避免重复发布帖子
                let exist = entity::milestone_post::Entity::find()
                    .filter(entity::milestone_post::Column::IssueId.eq(*issue.id))
                    .filter(entity::milestone_post::Column::Milestone.eq(&milestone_title))
                    .one(&app_state.mysql)
                    .await?;

                if exist.is_none() {
                    if let Err(err) = process_single_issue(
                        &app_state,
                        &deepseek_client,
                        &qq_client,
                        &issue,
                        &milestone_title,
                        &target_sub_channel_id,
                    )
                    .await
                    {
                        error!("处理里程碑Issue发生错误：{err:?}");
                    }
                } else {
                    debug!("已经发布过issue: {}，跳过处理", issue.id);
                }
            }

            if total_issue_this_page < MAX_PER_PAGE as usize {
                // 没有下一页
                break;
            }

            cur_page += 1;
        }

        // 删除不在milestone列表的帖子
        let not_in_milestone_posts = entity::milestone_post::Entity::find()
            .filter(entity::milestone_post::Column::IssueId.is_not_in(milestone_all_issues))
            .filter(entity::milestone_post::Column::Milestone.eq(&milestone_title))
            .all(&app_state.mysql)
            .await?;
        // TODO 删除逻辑
    }

    todo!()
}

pub async fn process_single_issue(
    app_state: &AppState,
    deepseek_client: &DeepSeekClient,
    qq_client: &QQBotClient,
    issue: &Issue,
    milestone_title: &str,
    target_sub_channel_id: &str,
) -> Result<()> {
    // AI 总结
    let issue_main_message = format!(
        "标题: {}, 内容: {:?}，发布者名称：{:?}, 时间UTC: {}, 状态: {:?}， 原文链接: {}, 里程碑: {}",
        issue.title,
        issue.body,
        issue.user.name,
        issue.created_at,
        issue.state,
        issue.html_url,
        milestone_title
    );

    let mut chat_messages = vec![
        MessageRequest::System(SystemMessageRequest::new(
            r"你是一个Bevy游戏引擎的社区宣传工作者，你需要下面这一个issue的详细信息进行内容翻译，翻译过程中的原文所描述的内容尽可能完整保留，内容中需要包含issue的标题，内容，发布者名称，时间UTC，状态，原文链接，并进行翻译，对其中的游戏引擎底层原理、图形学等专业知识（术语）进行恰当的解释。
            示例issue：
            假设一个issue：

            标题: Fix memory leak in ECS system
            内容: There is a memory leak when entities are despawned in the ECS. This causes the game to crash after prolonged play.
            发布者: john_doe
            时间: 2023-10-05T12:00:00Z
            状态: open
            链接: https://github.com/bevyengine/bevy/issues/1234
            里程碑：0.18
            总结：
            分类: Bug报告
            标题: 修复ECS系统中的内存泄漏（翻译）
            内容: 当实体在ECS中被销毁时，存在内存泄漏问题，导致游戏在长时间运行后崩溃。（翻译和总结）
            发布者: john_doe
            时间: 2023-10-05 12:00:00 UTC
            状态: 开启
            链接: [原文链接]
            术语解释: ECS（Entity-Component-System）是一种游戏开发架构，用于管理游戏对象（实体）及其属性（组件）和行为（系统）。内存泄漏是指程序在分配内存后未能释放，导致内存使用不断增加。",
        )),
        MessageRequest::user(&issue_main_message),
    ];

    // AI请求
    let now = Instant::now();
    info!("开始请求AI总结");

    let ds_res = CompletionsRequestBuilder::new(&chat_messages)
        .use_model(deepseek_api::response::ModelType::DeepSeekReasoner)
        .stream(false)
        .do_request(deepseek_client)
        .await?;

    let ds_res_text = get_first_deepseek_response(ds_res)?;
    info!("AI总结完成, 耗时: {}秒", now.elapsed().as_secs_f32());

    // 帖子发布
    qq_client
        .send_thread(&issue.title, &ds_res_text, target_sub_channel_id)
        .await?;

    // 数据库保存
    let new_milestone = entity::milestone_post::ActiveModel {
        id: NotSet,
        title: Set(issue.title.clone()),
        milestone: Set(milestone_title.to_string()),
        issue_id: Set(*issue.id),
    };

    new_milestone.insert(&app_state.mysql).await?;

    Ok(())
}

pub async fn get_milestone_list(spider: &Octocrab) -> Result<Page<Milestone>> {
    let milestones: Page<Milestone> = spider
        .get(
            format!("/repos/{}/{}/milestones", BEVY_OWNER, BEVY_REPO),
            None::<&()>,
        )
        .await?;

    Ok(milestones)
}

// 计算QQ频道的子频道名称
// 子频道名称必须长度为5
// 其中数字，小数点，英文占用0.5个长度
// 中文占用1个长度
fn get_channel_name(milestone_title: &str) -> String {
    let milestone_len = milestone_title.chars().count() as f32 * 0.5;
    let used_len = milestone_len.ceil() as usize;

    let chinese_len = (5_usize).checked_sub(used_len).unwrap_or_default();

    let text: String = "里程碑".chars().take(chinese_len).collect();

    format!("{}{}", milestone_title, text)
}

#[cfg(test)]
mod tests {
    use crate::{
        bots::github_client::build_github_client,
        tasks::github_task::{BEVY_OWNER, BEVY_REPO, watch_milestones::get_milestone_list},
    };

    #[tokio::test]
    async fn test_get_milestone_list() {
        dotenvy::dotenv().ok();

        let spider = build_github_client().unwrap();
        let list = get_milestone_list(&spider).await.unwrap();
        // println!("{:?}", list);
        for item in list {
            println!("版本: {}, id: {}", item.title, item.id);
            println!("{:?}", item);

            // let issue_list = spider
            //     .issues(BEVY_OWNER, BEVY_REPO)
            //     .list()
            //     .state(octocrab::params::State::All)
            //     .milestone(35)
            //     .page(0_u32)
            //     .per_page(100)
            //     .send()
            //     .await
            //     .unwrap();
            // ;
            // println!("总issue: {:?}", issue_list.items.len());
        }
    }
}
