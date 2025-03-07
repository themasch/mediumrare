use crate::client;
use crate::client::{Markup, PostResult};
use crate::text_markup::{SpanWrap, TextSpan};
use anyhow::{Context, Result};
use std::collections::HashMap;

macro_rules! attributes {
    ($($name:expr => $value:expr),+) => {
        {
            let mut attributes = HashMap::<String, String>::new();
            $(attributes.insert($name.to_string(), $value.to_string());)+

            attributes
        }
    }
}

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

                let child_html: Option<String> = children
                    .as_ref()
                    .map(|elements| elements.iter().map(|child| child.to_string()).collect());

                if let Some(child_html) = child_html {
                    format!("<{name} {}>{}</{name}>", attrs, child_html, name = name)
                } else {
                    format!("<{} {}/>", name, attrs)
                }
            }
        }
    }
}

impl Content {
    pub fn text<S: Into<String>>(txt: S) -> Content {
        Content::Text(
            txt.into()
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;"),
        )
    }

    pub fn hyperlink<S: Into<String>>(
        href: S,
        children: Vec<Content>,
        attr: Option<HashMap<String, String>>,
    ) -> Content {
        let mut attributes = attr.unwrap_or_default();
        attributes.insert("href".into(), href.into());
        Content::Tag {
            name: "a".into(),
            attributes,
            children: Some(children),
        }
    }

    pub fn tag<S: Into<String>>(
        name: S,
        attr: Option<HashMap<String, String>>,
        children: Option<Vec<Content>>,
    ) -> Content {
        Content::Tag {
            name: name.into(),
            attributes: attr.unwrap_or_default(),
            children,
        }
    }
}

fn render_text(text: &str, markups: &[Markup]) -> Result<Vec<Content>> {
    if markups.is_empty() {
        return Ok(vec![Content::text(text)]);
    }

    let mut span = TextSpan::create(text);

    // temp workaround, we should find a better way to handle this
    let mut sorted_markup = Vec::from(markups);
    sorted_markup.sort_by(|l, r| (r.end - r.start).cmp(&(l.end - l.start)));

    for markup in &sorted_markup {
        let subspan = span
            .get_sub_span_mut(markup.start, markup.end)
            .context(format!("failed to get span for markup {:?}", markup))?;
        let wrap = match markup.r#type.as_str() {
            "STRONG" => SpanWrap::Strong,
            "CODE" => SpanWrap::Code,
            "EM" => SpanWrap::Emphasized,
            "A" => SpanWrap::Link {
                href: markup.href.as_ref().unwrap_or(&"".to_string()).to_string(),
            },
            _ => panic!("unknown markup type {}", markup.r#type),
        };

        subspan.add_wrap(wrap);
    }

    Ok(span.into())
}

pub trait Render {
    fn render(&self) -> Result<Content>;
}

impl Render for client::Paragraph {
    fn render(&self) -> Result<Content> {
        Ok(match self.r#type.as_str() {
            "IMG" => {
                let attr = Some(attributes! {
                    "src" => format!("https://miro.medium.com/v2/resize:fit:2000/{}",self.metadata.as_ref().unwrap().id),
                    "loading" => "lazy"
                });
                Content::tag("img", attr, None)
            }
            // TODO: wrap these in a grouping <ul> or <ol> tag
            "ULI" | "OLI" => Content::tag(
                "li",
                None,
                Some(
                    render_text(self.text.as_ref().map_or("", |t| t.as_str()), &self.markups)
                        .context("on rendering LI tag")?,
                ),
            ),
            "IFRAME" => {
                let attr = Some(attributes! {
                    "href" => self.iframe
                        .as_ref()
                        .unwrap()
                        .media_resource
                        .iframe_src
                        .clone()
                });
                Content::tag(
                    "a",
                    attr,
                    Some(vec![
                        Content::text("IFRAME: "),
                        Content::text(self.iframe.as_ref().unwrap().media_resource.title.clone()),
                    ]),
                )
            }
            "BQ" => Content::tag(
                "blockquote",
                None,
                Some(render_text(
                    self.text.as_ref().map_or("", |t| t.as_str()),
                    &self.markups,
                )?),
            ),
            "P" | "H1" | "H2" | "H3" | "H4" | "H5" | "H6" | "PRE" => Content::tag(
                self.r#type.to_lowercase(),
                None,
                Some(render_text(
                    self.text.as_ref().map_or("", |t| t.as_str()),
                    &self.markups,
                )?),
           ),
            _ => {
                let attr = Some(attributes! {"x-real-tag" => self.r#type});
                Content::tag(
                    "div",
                    attr,
                    Some(
                        render_text(self.text.as_ref().map_or("", |t| t.as_str()), &self.markups)
                            .context(format!("on rendering a {} tag", self.r#type))?,
                    ),
                )
            }
        })
    }
}

impl Render for client::PostResult {
    fn render(&self) -> Result<Content> {
        let mut content = self.render_header()?;
        let mut body: Vec<Content> = self
            .paragraphs()
            .iter()
            .map(|p| p.render())
            .collect::<Result<Vec<Content>>>()?;
        content.append(&mut body);
        Ok(Content::tag("article", None, Some(content)))
    }
}

impl client::PostResult {
    fn render_header(&self) -> Result<Vec<Content>> {
        Ok(vec![Content::tag(
            "div",
            Some(attributes!( "class" => "post-head")),
            Some(vec![
                Content::text("published by "),
                Content::hyperlink(
                    format!(
                        "https://medium.com/@{username}",
                        username = self.creator.username
                    ),
                    vec![Content::text(self.creator.name.clone())],
                    None,
                ),
                Content::text(" on medium "),
                Content::hyperlink(self.medium_url.clone(), vec![Content::text("here")], None),
                Content::text("."),
            ]),
        )])
    }
}

const CSS: &str =
    "body { background-color: #222; color: #ddd; font-family: sans-serif; font-size: 130%; }
article { width: 60rem; margin: auto }
img { max-width: 100% }
pre { background-color: #111; padding: 1rem; border-radius: .5rem; }
blockquote { background-color: #333; margin: 0; padding: 1rem;  padding-left: 2rem; border-left: 5px solid gray; }
a { color: cornflowerblue }
.post-head {  background-color: #333; margin: 0; padding: 1rem; font-size: 80%; }";

pub struct Page {
    post: PostResult,
}

impl Render for Page {
    fn render(&self) -> Result<Content> {
        Ok(Content::tag(
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
                Content::tag("body", None, Some(vec![self.post.render()?])),
            ]),
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::client::{Markup, Paragraph};

    use super::Render;

    #[test]
    fn test_renders_incorrectly_sorted_markup_correctly() {
        let p = Paragraph {
            id: String::from(""),
            href: None,
            layout: None,
            text: Some(String::from("This is a test with some text")),
            r#type: "P".into(),
            markups: vec![
                Markup {
                    start: 10,
                    end: 13,
                    r#type: String::from("EM"),
                    href: None,
                },
                Markup {
                    start: 8,
                    end: 13,
                    r#type: String::from("STRONG"),
                    href: None,
                },
            ],
            metadata: None,
            iframe: None,
        };

        assert_eq!(
            "<p >This is <strong >a <em >test</em></strong> with some text</p>",
            p.render().unwrap().to_string()
        );
    }
}
