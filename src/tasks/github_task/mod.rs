use anyhow::Result;
use deepseek_api::response::{
    ChatCompletion, ChatCompletionStream, ChatResponse, JSONChoiceStream,
};
use log::*;

pub mod watch_issue_list;
pub mod watch_milestones;
pub mod watch_pr;

// const BEVY_GITHUB: &str = "https://github.com/bevyengine/bevy";
const BEVY_OWNER: &str = "bevyengine";
const BEVY_REPO: &str = "bevy";

pub fn get_first_deepseek_response(
    response: ChatResponse<ChatCompletion, ChatCompletionStream<JSONChoiceStream>>,
) -> Result<String> {
    let response = response.must_response();
    info!("{:?}", response);

    let Some(choice) = response.choices.first() else {
        anyhow::bail!("获取choices失败");
    };
    let Some(message) = &choice.message else {
        anyhow::bail!("获取text失败");
    };

    if message.content.is_empty() {
        anyhow::bail!("文本为空");
    }

    Ok(message.content.clone())
}
