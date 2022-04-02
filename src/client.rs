use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const QUERY_TEXT: &str = "query PostHandler($postId:ID!) {
    postResult(id: $postId) { 
        ... on Post { 
            title, 
            id, 
            mediumUrl, 
            previewImage { 
                id, 
                originalHeight, 
                originalWidth  
            }, 
            latestPublishedAt, 
            updatedAt, 
            createdAt, 
            creator { 
                id, 
                name, 
                username, 
                bio
            }, 
            readingTime, 
            clapCount, 
            tags { 
                id, 
                displayTitle, 
                normalizedTagSlug
            }, 
            topics { 
                topicId, 
                name
            }, 
            content { 
                bodyModel { 
                    paragraphs { 
                        id, 
                        text, 
                        href, 
                        type, 
                        layout, 
                        iframe {
                            mediaResource {
                                id,
                                iframeSrc,
                                iframeHeight,
                                iframeWidth,
                                title,
                            }
                        },
                        metadata { 
                            id, 
                            originalHeight, 
                            originalWidth, 
                            alt 
                        }, 
                        markups {
                            start, 
                            end, 
                            type, 
                            href 
                        } 
                    }
                }
            } 
        } 
    } 
}";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest<'a> {
    operation_name: &'a str,
    query: &'a str,
    variables: HashMap<&'a str, &'a str>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostResult {
    id: String,
    medium_url: String,
    pub title: String,
    clap_count: u32,
    created_at: usize,
    updated_at: usize,
    latest_published_at: usize,
    reading_time: f32,
    preview_image: PreviewImage,
    creator: Creator,
    tags: Vec<Tag>,
    topics: Vec<Topic>,
    content: Content,
}

impl PostResult {
    pub fn paragraphs(&self) -> &Vec<Paragraph> {
        &self.content.body_model.paragraphs
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    body_model: BodyModel,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BodyModel {
    paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Paragraph {
    id: String,
    href: Option<String>,
    layout: Option<String>,
    pub text: Option<String>,
    pub r#type: String,
    pub markups: Vec<Markup>,
    pub metadata: Option<Metadata>,
    pub iframe: Option<IFrame>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IFrame {
    pub media_resource: IFrameMediaResource,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IFrameMediaResource {
    id: String,
    pub iframe_src: String,
    pub iframe_height: usize,
    pub iframe_width: usize,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    alt: Option<String>,
    pub id: String,
    original_width: usize,
    original_height: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Markup {
    end: usize,
    start: usize,
    href: Option<String>,
    r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    topic_id: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    id: String,
    display_title: String,
    normalized_tag_slug: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    id: String,
    username: String,
    name: String,
    bio: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewImage {
    id: String,
    original_width: Option<usize>,
    original_height: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseData {
    post_result: PostResult,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    data: ResponseData,
}

impl QueryResponse {
    pub fn get_post(self) -> PostResult {
        self.data.post_result
    }
}

fn create_post_query(post_id: &str) -> QueryRequest {
    let mut hash_map = HashMap::new();
    hash_map.insert("postId", post_id);
    QueryRequest {
        operation_name: "PostHandler",
        query: QUERY_TEXT,
        variables: hash_map,
    }
}

pub trait PostDataClient {
    fn get_post_data(&self, post_id: &str) -> Result<QueryResponse, ()>;
}

pub struct Client;

impl PostDataClient for Client {
    fn get_post_data(&self, post_id: &str) -> Result<QueryResponse, ()> {
        let response_text = ureq::post("https://medium.com/_/graphql")
            .set("Content-Type", "application/json")
            .send_json(&create_post_query(post_id))
            .unwrap()
            .into_string()
            .unwrap();

        println!("{response_text}");

        Ok(serde_json::from_str::<QueryResponse>(&response_text).unwrap())
    }
}
