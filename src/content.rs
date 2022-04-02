use crate::client;
use crate::client::PostResult;
use std::collections::HashMap;

pub enum Content {
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

pub trait Render {
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

pub struct Page {
    post: PostResult,
}

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

impl Page {
    pub fn create(post_result: PostResult) -> Self {
        Page { post: post_result }
    }
}
