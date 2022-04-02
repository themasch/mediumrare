mod client;

use client::PostResult;
use lambda_http::{
    http::StatusCode, service_fn, Error, IntoResponse, Request, RequestExt, Response,
};
use std::collections::HashMap;
use std::string::ToString;

enum Content {
    Text(String),
    Tag {
        name: String,
        attributes: HashMap<String, String>,
        children: Option<Vec<Content>>,
    },
}

impl ToString for Content {
    fn to_string(&self) -> String {
        match self {
            Self::Text(t) => t.to_owned(),
            Self::Tag {
                name,
                attributes,
                children,
            } => {
                let attrs: String = attributes
                    .iter()
                    .map(|(name, value)| format!(r#"{}="{}" "#, name, value))
                    .collect();

                let children = if let Some(elements) = children {
                    elements.iter().map(|child| child.to_string()).collect()
                } else {
                    "".to_string()
                };

                format!("<{} {}>{}</{}>", name, attrs, children, name)
            }
        }
    }
}

impl Content {
    pub fn text<S: Into<String>>(txt: S) -> Content {
        Content::Text(txt.into())
    }

    pub fn tag<S: Into<String>>(
        name: S,
        attr: Option<HashMap<String, String>>,
        children: Option<Vec<Content>>,
    ) -> Content {
        Content::Tag {
            name: name.into(),
            attributes: attr.unwrap_or_else(|| HashMap::new()),
            children,
        }
    }
}

trait Render {
    fn render(&self) -> Content;
}

impl Render for client::Paragraph {
    fn render(&self) -> Content {
        match self.r#type.as_str() {
            "IMG" => {
                let mut attributes: HashMap<String, String> = HashMap::new();
                attributes.insert(
                    "src".into(),
                    format!(
                        "https://miro.medium.com/max/2000/{}",
                        self.metadata.as_ref().unwrap().id
                    ),
                );
                Content::tag("img", Some(attributes), None)
            }
            "OLI" => Content::tag(
                "li",
                None,
                Some(vec![Content::text(
                    self.text.as_ref().map_or("", |t| t.as_str()),
                )]),
            ),
            "IFRAME" => {
                let mut attributes: HashMap<String, String> = HashMap::new();
                attributes.insert(
                    "href".into(),
                    self.iframe
                        .as_ref()
                        .unwrap()
                        .media_resource
                        .iframe_src
                        .clone(),
                );
                Content::tag(
                    "a",
                    Some(attributes),
                    Some(vec![
                        Content::text("IFRAME: "),
                        Content::text(self.iframe.as_ref().unwrap().media_resource.title.clone()),
                    ]),
                )
            }
            _ => Content::tag(
                &self.r#type,
                None,
                Some(vec![Content::text(
                    self.text.as_ref().map_or("", |t| t.as_str()),
                )]),
            ),
        }
    }
}

impl Render for client::PostResult {
    fn render(&self) -> Content {
        Content::tag(
            "article",
            None,
            Some(self.paragraphs().iter().map(|p| p.render()).collect()),
        )
    }
}

const CSS: &str =
    "body { background-color: #222; color: #ddd; font-family: sans-serif; font-size: 180%; }
article { width: 70rem; margin: auto }
img { max-width: 100% }";

impl Render for Page {
    fn render(&self) -> Content {
        Content::tag(
            "html",
            None,
            Some(vec![
                Content::tag(
                    "head",
                    None,
                    Some(vec![
                        Content::tag("style", None, Some(vec![Content::text(CSS)])),
                        Content::tag(
                            "title",
                            None,
                            Some(vec![Content::text(self.post.title.clone())]),
                        ),
                    ]),
                ),
                Content::tag("body", None, Some(vec![self.post.render()])),
            ]),
        )
    }
}

struct Page {
    post: PostResult,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(event: Request) -> Result<impl IntoResponse, Error> {
    let params = event.path_parameters();

    let postid = params.first("postid").unwrap_or("633ff591866e");

    let page = Page {
        post: client::get_post_data(postid).get_post(),
    };

    let builder = Response::builder()
        .header("Content-Type", "text/html;charset=utf-8")
        .status(StatusCode::OK);

    Ok(builder
        .body(page.render().to_string())
        .expect("failed to build response"))
}
