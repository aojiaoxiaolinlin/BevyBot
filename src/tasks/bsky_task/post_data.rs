use serde::{Deserialize, Serialize};

// 顶层结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadPost {
    pub thread: ThreadViewPost,
}

// 核心的递归结构，代表一个帖子及其回复线程
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadViewPost {
    #[serde(rename = "$type")]
    pub bsky_type: String,
    pub post: PostView,
    pub replies: Option<Vec<ThreadViewPost>>,
    // threadContext在某些嵌套层级不存在，所以是可选的
    pub thread_context: Option<ThreadContext>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostView {
    pub uri: String,
    pub cid: String,
    pub author: Author,
    pub record: Record,
    // embed 在顶层帖子中不存在，但在回复中存在，所以是可选的
    pub embed: Option<EmbedView>,
    pub bookmark_count: u64,
    pub reply_count: u64,
    pub repost_count: u64,
    pub like_count: u64,
    pub quote_count: u64,
    pub indexed_at: String,
    pub labels: Vec<Label>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub did: String,
    pub handle: String,
    // displayName 和 avatar 在某些用户对象中可能不存在
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub associated: Associated,
    pub labels: Vec<Label>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Associated {
    pub activity_subscription: ActivitySubscription,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySubscription {
    pub allow_subscriptions: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    #[serde(rename = "$type")]
    pub bsky_type: String,
    pub created_at: String,
    pub text: String,
    pub langs: Vec<String>,
    // facets, reply, embed 都是可选的
    pub facets: Option<Vec<Facet>>,
    pub reply: Option<ReplyRef>,
    pub embed: Option<RecordEmbed>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Facet {
    pub index: ByteSlice,
    pub features: Vec<Feature>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByteSlice {
    pub byte_start: u64,
    pub byte_end: u64,
}

// 使用标记枚举来处理不同类型的 feature
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum Feature {
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag { tag: String },
    #[serde(rename = "app.bsky.richtext.facet#mention")]
    Mention { did: String },
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link { uri: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyRef {
    pub root: PostRef,
    pub parent: PostRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRef {
    pub cid: String,
    pub uri: String,
}

// Record中的Embed
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum RecordEmbed {
    #[serde(rename = "app.bsky.embed.external")]
    External { external: ExternalRecord },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalRecord {
    pub description: String,
    pub thumb: Thumb,
    pub title: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumb {
    #[serde(rename = "$type")]
    pub bsky_type: String,
    pub mime_type: String,
    pub r#ref: LinkRef, // 'ref' is a keyword in Rust, so we use r#ref
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkRef {
    #[serde(rename = "$link")]
    pub link: String,
}

// PostView中的Embed
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum EmbedView {
    #[serde(rename = "app.bsky.embed.external#view")]
    ExternalView { external: ExternalViewData },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalViewData {
    pub uri: String,
    pub title: String,
    pub description: String,
    pub thumb: String,
}

// 虽然 labels 数组为空，但我们仍需定义其类型
#[derive(Debug, Serialize, Deserialize)]
pub struct Label {}

// 虽然 threadContext 对象为空，但我们仍需定义其类型
#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadContext {}
