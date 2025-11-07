use std::{env, sync::Arc, time::Instant};

use actix_rt::spawn;
use anyhow::Result;
use deepseek_api::{CompletionsRequestBuilder, DeepSeekClient, RequestBuilder, request::MessageRequest, response::AssistantMessage};
use log::{error, info};
use sea_orm::{ActiveValue::{NotSet, Set}, ColumnTrait, EntityTrait, QueryFilter, ActiveModelTrait};
use tokio_schedule::{Job, every};

use crate::{AppState, bots::{bsky_client::BskyClient, deepseek_client::build_deepseek_client, qqbot_client::QQBotClient}, tasks::{bsky_task::{BEVY_MERGE_TRAIN_API, feed_data::Feed}, github_task::get_first_deepseek_response}};



pub fn spawn_mergetrain_task(
    app_state: AppState
) {
    info!("开始定时抓取mergetrain任务");

    let app_state = Arc::new(app_state);

    let every_day_task = every(1).day().at(12, 20, 00).perform( move || {
        let state = app_state.as_ref().clone();
        let future = async {
            if let Err(err) = run_mergetrain_task(state).await {
                error!("{err:?}");
            }
        };
        future
    });

    spawn(every_day_task);
}

pub async fn run_mergetrain_task(app_state: AppState) -> Result<()> {
    let deepseek_client = build_deepseek_client()?;
    let qq_client = QQBotClient::new_with_default(false).await?;
    let bsk_client = BskyClient::new();


    let mergetrain_list = get_first_page_mergetrain(&bsk_client).await?;

    process_post_thread(&app_state, &bsk_client, mergetrain_list, &deepseek_client, &qq_client).await?;

    Ok(())
}


pub async fn process_post_thread(
    app_state: &AppState,
    client: &BskyClient,
    post_list: Vec<MergeTrainPost>,
    deepseek_client: &DeepSeekClient,
    qq_client: &QQBotClient
) -> Result<()> {
    for post in post_list {

        let title = format!("MergeTrain: {}", post.date);

        let exist = entity::merge_train::Entity::find()
            .filter(entity::merge_train::Column::Cid.eq(&post.cid))
            .one(&app_state.mysql)
            .await?
        ;

        if !exist.is_none() {
            continue;
        }

        let thread_post: serde_json::Value = client.get_pub_post_thread(&post.uri).await?;
        // let thread_post_text = &thread_post.thread.post.record.text;

        let text = serde_json::to_string(&thread_post)?;

        let mut chat_messages = vec![];
        chat_messages.push(
            MessageRequest::Assistant(AssistantMessage::new(
                r"你是一个专业的数据分析师和内容摘要专家。你的任务是解析给定的JSON数据，该数据代表一个社交媒体帖子及其回复。你需要从中提取关键信息，并以清晰的Markdown格式生成一份摘要，向读者解释这个帖子的主要内容和讨论。

                请遵循以下步骤：

                1.  **识别主贴内容**：
                    *   找到帖子的主要作者：Alice I Cecile。
                    *   检查主贴是否包含文本。如果主贴包含的是一个嵌入式外链（类型为 `app.bsky.embed.external`），请提取该外链的以下信息：
                        *   链接标题 (`title`)
                        *   链接描述 (`description`)
                        *   链接地址 (`uri`)

                2.  **总结回复串**：
                    *   JSON数据中包含一个嵌套的回复链（`replies`）。这些回复详细阐述了Alice I Cecile的观点。
                    *   请按顺序阅读这些回复（`text` 字段），并将其内容整合成一个连贯的段落或几个要点。这部分是帖子的核心思想。
                    *   注意，这个帖子是Alice I Cecile在解释自己审查一个技术性PR（Pull Request）时的思考过程。

                3.  **识别其他回复**：
                    *   检查是否有来自其他用户的独立回复，并简要提及。

                4.  **格式化输出**：
                    *   使用Markdown格式。
                    *   为摘要起一个合适的标题。
                    *   使用标题、列表和引用块来组织内容，使其易于阅读。
                    *   将提取的外链格式化为Markdown链接。

                请根据下面的JSON数据生成摘要：",
            ))
        );
        chat_messages.push(MessageRequest::user(&text));

        let now = Instant::now();
        info!("开始请求AI总结");

        let ds_res = CompletionsRequestBuilder::new(&chat_messages)
            .use_model(deepseek_api::response::ModelType::DeepSeekReasoner)
            .stream(false)
            .max_tokens(8192)
            .unwrap()
            .do_request(&deepseek_client)
            .await?;

        let ds_res_text = get_first_deepseek_response(ds_res)?;
        info!("AI总结完成, 耗时: {}秒", now.elapsed().as_secs_f32());

        // 帖子发布
        qq_client.send_thread(&title, &ds_res_text, &env::var("MERGETRAIN_CHANNEL_ID").unwrap()).await?;

        // 数据库保存
        let new_milestone = entity::merge_train::ActiveModel {
            id: NotSet,
            cid: Set(post.cid.clone()),
            title: Set(title)
        };

        new_milestone.insert(&app_state.mysql).await?;
    }

    Ok(())
}


pub struct MergeTrainPost {
    pub uri: String,
    pub cid: String,
    pub date: String
}

pub async fn get_first_page_mergetrain(
    client: &BskyClient
) -> Result<Vec<MergeTrainPost>> {
    let feed_data: Feed = client.get_pub(BEVY_MERGE_TRAIN_API).await?;

    let cid_list = feed_data.feed
        .iter()
        .filter(|feed| {
            if let Some(facets) = &feed.post.record.facets {
                for facet in facets {
                    for feature in &facet.features  {
                        match feature {
                            crate::tasks::bsky_task::feed_data::Feature::Tag { tag } => {
                                if tag == "bevymergetrain" {
                                    return true;
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }

            false
        }).map(|feed| {
            MergeTrainPost {
                uri: feed.post.uri.clone(),
                cid: feed.post.cid.clone(),
                date: (&feed.post.record.created_at[0..10]).to_string()
            }
        })
        .collect::<Vec<_>>();

    Ok(cid_list)
}


#[cfg(test)]
mod tests {
    use crate::{bots::bsky_client::BskyClient, tasks::bsky_task::{BEVY_MERGE_TRAIN_API, feed_data::Feed, post_data::ThreadPost}};


    #[tokio::test]
    async fn test_mergetrain() {
        dotenvy::dotenv().ok();

        let bsky_client = BskyClient::new();

        let feed_data: Feed = bsky_client.get_pub(BEVY_MERGE_TRAIN_API).await.unwrap();

        println!("{:?}", feed_data);
    }


    #[tokio::test]
    async fn test_post_thread() {
        dotenvy::dotenv().ok();

        let bsky_client = BskyClient::new();

        let post: ThreadPost = bsky_client.get_pub_post_thread("at://did:plc:fjg6pzaigjmfpsfnbyp6m5oc/app.bsky.feed.post/3m4qp6zgvzc2j").await.unwrap();

        println!("{:?}", post);
    }
}
