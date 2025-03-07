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
    pub medium_url: String,
    pub title: String,
    clap_count: u32,
    created_at: usize,
    updated_at: usize,
    latest_published_at: usize,
    reading_time: f32,
    preview_image: PreviewImage,
    pub creator: Creator,
    pub tags: Vec<Tag>,
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
    pub(crate) id: String,
    pub(crate) href: Option<String>,
    pub(crate) layout: Option<String>,
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
    pub(crate) id: String,
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
    pub end: usize,
    pub start: usize,
    pub href: Option<String>,
    pub r#type: String,
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
    pub username: String,
    pub name: String,
    pub bio: String,
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
    fn get_post_data(&self, post_id: &str) -> Result<QueryResponse, ClientError>;
}

pub struct Client;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("not found: {0}")]
    NotFoundError(String),

    #[error("error on request: {0:?}")]
    RequestError(#[from] ureq::Error),

    #[error("failed decoding json")]
    EncodingError(#[from] serde_json::Error),
}

impl PostDataClient for Client {
    fn get_post_data(&self, post_id: &str) -> Result<QueryResponse, ClientError> {
        let mut response = ureq::post("https://medium.com/_/graphql")
            .header("Content-Type", "application/json")
            .send_json(create_post_query(post_id))?;

        if response.status() == 404 {
            return Err(ClientError::NotFoundError(post_id.to_string()));
        }

        let response_text = response.body_mut().read_to_string().unwrap();

        if response_text == "{\"data\":{\"postResult\":{}}}\n" {
            return Err(ClientError::NotFoundError(post_id.to_string()));
        }

        Ok(serde_json::from_str::<QueryResponse>(&response_text)?)
    }
}
