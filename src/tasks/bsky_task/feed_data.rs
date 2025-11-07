use serde::{Deserialize, Serialize};

// 整个JSON响应的根结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub feed: Vec<FeedItem>,
    pub cursor: Option<String> // 如果是none, 代表没有下一页
}

// "feed"数组中的每个元素
#[derive(Debug, Serialize, Deserialize)]
pub struct FeedItem {
    pub post: Post,
    pub reason: Option<Reason>,
}

// "reason"字段的枚举，根据`$type`的值来区分
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum Reason {
    #[serde(rename = "app.bsky.feed.defs#reasonRepost")]
    ReasonRepost(ReasonRepostData),
}

// "reasonRepost"类型的数据结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasonRepostData {
    pub by: Author,
    pub uri: String,
    pub cid: String,
    pub indexed_at: String,
}

// "post"对象的结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub uri: String,
    pub cid: String,
    pub author: Author,
    pub record: Record,
    // 这个embed是视图模型(view model)，与record中的embed结构不同
    pub embed: Option<EmbedView>,
    pub bookmark_count: u64,
    pub reply_count: u64,
    pub repost_count: u64,
    pub like_count: u64,
    pub quote_count: u64,
    pub indexed_at: String,
    pub labels: Vec<Label>,
}

// 作者/用户信息结构 (可被`author`和`by`字段复用)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub did: String,
    pub handle: String,
    pub display_name: String,
    pub avatar: String,
    pub associated: Associated,
    pub labels: Vec<Label>,
    pub created_at: String,
}

// 作者关联信息，包含可选字段
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Associated {
    // chat字段是可选的
    pub chat: Option<Chat>,
    pub activity_subscription: ActivitySubscription,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub allow_incoming: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySubscription {
    pub allow_subscriptions: String,
}

// "record"对象的结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub created_at: String,
    // 这个embed是记录模型(record model)，与post顶层的embed结构不同
    pub embed: Option<RecordEmbed>,
    pub facets: Option<Vec<Facet>>,
    pub langs: Option<Vec<String>>,
    pub text: String,
}

// "record"内部的"embed"结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordEmbed {
    #[serde(rename = "$type")]
    pub embed_type: String,
    pub external: Option<RecordEmbedExternal>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordEmbedExternal {
    pub description: String,
    pub title: String,
    pub uri: String,
    // thumb字段是可选的，并且是一个对象
    pub thumb: Option<ThumbObject>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbObject {
    #[serde(rename = "$type")]
    pub thumb_type: String,
    #[serde(rename = "ref")]
    pub reference: Link,
    pub mime_type: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    #[serde(rename = "$link")]
    pub link: String,
}

// "post"顶层的"embed"视图结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedView {
    #[serde(rename = "$type")]
    pub embed_type: String,
    pub external: Option<ViewEmbedExternal>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewEmbedExternal {
    pub uri: String,
    pub title: String,
    pub description: String,
    // thumb字段是可选的，并且是一个字符串URL
    pub thumb: Option<String>,
}

// "facets"数组中的元素
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Facet {
    pub features: Vec<Feature>,
    pub index: Index,
}

// "features"数组中的元素，使用枚举来处理不同类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum Feature {
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag { tag: String },
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link { uri: String },
    #[serde(rename = "app.bsky.richtext.facet#mention")]
    Mention { did: String },
}

// "index"对象的结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub byte_end: u64,
    pub byte_start: u64,
}

// "labels"数组为空，定义一个空结构体作为占位符
#[derive(Debug, Serialize, Deserialize)]
pub struct Label {}
