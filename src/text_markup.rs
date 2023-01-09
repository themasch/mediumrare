use crate::content::Content;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum RenderingError {
    #[error("No span found between {0} and {1}")]
    NoSuchSpan(usize, usize),
}

#[derive(Debug, PartialEq)]
enum SpanContent<'a> {
    Text(&'a str),
    Spans(Vec<TextSpan<'a>>),
}

#[derive(Debug, PartialEq)]
pub enum SpanWrap {
    Strong,
    Emphasized,
    Link { href: String },
}

#[derive(Debug, PartialEq)]
pub struct TextSpan<'a> {
    start: usize,
    end: usize,
    content: SpanContent<'a>,
    wraps: Vec<SpanWrap>,
}

impl SpanWrap {
    fn create_tag(&self, children: Vec<Content>) -> Content {
        let empty = HashMap::new();
        let (tag_name, attributes) = match self {
            SpanWrap::Strong => ("strong", empty),
            SpanWrap::Emphasized => ("em", empty),
            SpanWrap::Link { href } => {
                let mut attributes = HashMap::new();
                attributes.insert("href".into(), href.to_owned());
                ("a", attributes)
            }
        };

        Content::Tag {
            name: tag_name.to_string(),
            attributes,
            children: Some(children),
        }
    }
}

impl<'a> Into<Vec<Content>> for TextSpan<'a> {
    fn into(self) -> Vec<Content> {
        let inner = match self.content {
            SpanContent::Text(str) => vec![Content::text(str)],
            SpanContent::Spans(spans) => spans
                .into_iter()
                .flat_map::<Vec<Content>, _>(|s| s.into())
                .collect(),
        };

        if self.wraps.is_empty() {
            return inner;
        }

        let mut wrapped = inner;
        for wrapper in self.wraps {
            wrapped = vec![wrapper.create_tag(wrapped)];
        }

        wrapped
    }
}

impl<'a> TextSpan<'a> {
    pub fn create(content: &'a str) -> TextSpan<'a> {
        TextSpan {
            start: 0,
            end: utf16_len(content) - 1,
            content: SpanContent::Text(content),
            wraps: Vec::new(),
        }
    }

    fn from_split(content: &'a str, start: usize) -> TextSpan<'a> {
        TextSpan {
            start,
            end: start + utf16_len(content) - 1,
            content: SpanContent::Text(content),
            wraps: Vec::new(),
        }
    }

    pub fn add_wrap(&mut self, wrap: SpanWrap) {
        self.wraps.push(wrap);
    }

    pub fn get_sub_span_mut(&mut self, start: usize, end: usize) -> Result<&mut TextSpan<'a>> {
        debug_assert!(end >= start);
        // sometime they send us offsets outside the actual string.. thanks
        let end = end.min(self.end);
        if start == self.start && end == self.end {
            return Ok(self);
        }

        match self.content {
            SpanContent::Text(str_content) => {
                let (new_content, idx) = Self::split_str(str_content, self.start, start, end);
                self.content = SpanContent::Spans(new_content);
                if let SpanContent::Spans(ref mut spans) = &mut self.content {
                    return Ok(&mut spans[idx]);
                }

                panic!("something went wrong")
            }
            SpanContent::Spans(ref mut subspans) => {
                for span in subspans.iter_mut() {
                    if span.start <= start && span.end >= end {
                        return span.get_sub_span_mut(start, end);
                    }
                }

                Err(RenderingError::NoSuchSpan(start, end).into())
            }
        }
    }

    fn split_str(content: &str, offset: usize, start: usize, end: usize) -> (Vec<TextSpan>, usize) {
        let (prefix, remainder) = if start == offset {
            (None, content)
        } else {
            let (p, r) = split_at_utf16_offset(content, start - offset);
            (Some(TextSpan::from_split(p, offset)), r)
        };

        let (suffix, center) = if utf16_len(content) - 1 + offset == end {
            (None, remainder)
        } else {
            let (c, s) = split_at_utf16_offset(remainder, end - start + 1);
            (Some(TextSpan::from_split(s, end + 1)), c)
        };

        let center = TextSpan::from_split(
            center,
            if let Some(ts) = &prefix {
                ts.end + 1
            } else {
                offset
            },
        );

        match (prefix, suffix) {
            (None, None) => (vec![center], 0),
            (None, Some(s)) => (vec![center, s], 0),
            (Some(p), None) => (vec![p, center], 1),
            (Some(p), Some(s)) => (vec![p, center, s], 1),
        }
    }
}

fn split_at_utf16_offset(content: &str, u16_len: usize) -> (&str, &str) {
    let prefix_len = utf16_to_byte_offset(content, u16_len);

    let (p, r) = content.split_at(prefix_len);
    assert_eq!(utf16_len(p), u16_len);

    (p, r)
}

fn utf16_len(content: &str) -> usize {
    content.encode_utf16().count()
}

fn utf16_to_byte_offset(content: &str, utf16_offset: usize) -> usize {
    let mut utf16_count = 0;
    let mut buffer = [0, 0];
    for (index, chr) in content.char_indices() {
        let len = chr.encode_utf16(&mut buffer).len();
        if utf16_count >= utf16_offset {
            return index;
        }

        utf16_count += len;
    }

    panic!("not in string");
}

#[cfg(test)]
mod test {
    use crate::text_markup::{split_at_utf16_offset, utf16_to_byte_offset, SpanContent, TextSpan};

    #[test]
    fn utf16_index_one_byte_chars() {
        let input = "0123456789";
        assert_eq!(0, utf16_to_byte_offset(input, 0));
        assert_eq!(5, utf16_to_byte_offset(input, 5));
        assert_eq!(9, utf16_to_byte_offset(input, 9));
    }

    #[test]
    fn utf16_index_mixed_byte_chars() {
        let input = "L 👋🏽 R";
        // | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 0 | 1 | utf8 byte index
        // |---|---|---|---|---|---|---|---|---|---|---|---|
        // | L |   | hand wave     | skin tone     |   | R | utf8 codepoint
        // | 0 | 1 |   2   |   3   |   4   |   5   | 6 | 7 | utf16 index
        assert_eq!(0, utf16_to_byte_offset(input, 0));
        assert_eq!(1, utf16_to_byte_offset(input, 1));
        assert_eq!(2, utf16_to_byte_offset(input, 2));
        //assert_eq!(4, utf16_to_byte_offset(input, 3)); // not really a legal position
        assert_eq!(6, utf16_to_byte_offset(input, 4));
        //assert_eq!(8, utf16_to_byte_offset(input, 5)); // not really a legal position
        assert_eq!(10, utf16_to_byte_offset(input, 6));
        assert_eq!(11, utf16_to_byte_offset(input, 7));
    }

    #[test]
    fn utf16_split() {
        let input = "L 👋🏽 R";
        assert_eq!(("L ", "👋🏽 R"), split_at_utf16_offset(input, 2));
        assert_eq!(("L 👋🏽", " R"), split_at_utf16_offset(input, 6));
    }

    #[test]
    fn test_does_not_split_for_full_range() {
        let input = "0123456789";

        let mut span = TextSpan::create(input);
        let sub_span = span.get_sub_span_mut(0, 9);

        assert_eq!(SpanContent::Text(input), sub_span.unwrap().content);
    }

    #[test]
    fn test_real_example() {
        let input = "hi 👋🏽 there\nthis is a test";

        let mut span = TextSpan::create(input);

        assert_eq!(
            SpanContent::Text("hi "),
            span.get_sub_span_mut(0, 2).unwrap().content
        );
        assert_eq!(
            SpanContent::Text("there"),
            span.get_sub_span_mut(8, 12).unwrap().content
        );
        assert_eq!(
            SpanContent::Text("test"),
            span.get_sub_span_mut(24, 27).unwrap().content
        );
    }

    #[test]
    fn test_split_first_part() {
        let input = String::from("0123456789");

        let mut span = TextSpan::create(&input);
        span.get_sub_span_mut(0, 3);

        assert_eq!(
            TextSpan {
                start: 0,
                end: 9,
                content: SpanContent::Spans(vec![
                    TextSpan {
                        start: 0,
                        end: 3,
                        content: SpanContent::Text("0123"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 4,
                        end: 9,
                        content: SpanContent::Text("456789"),
                        wraps: vec![],
                    },
                ]),
                wraps: vec![],
            },
            span
        );
    }

    #[test]
    fn test_split_last_part() {
        let input = String::from("0123456789");

        let mut span = TextSpan::create(&input);
        span.get_sub_span_mut(6, 9);

        assert_eq!(
            TextSpan {
                start: 0,
                end: 9,
                content: SpanContent::Spans(vec![
                    TextSpan {
                        start: 0,
                        end: 5,
                        content: SpanContent::Text("012345"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 6,
                        end: 9,
                        content: SpanContent::Text("6789"),
                        wraps: vec![],
                    },
                ]),
                wraps: vec![],
            },
            span
        );
    }

    #[test]
    fn test_split_middle_part() {
        let input = String::from("0123456789");

        let mut span = TextSpan::create(&input);
        span.get_sub_span_mut(4, 6);

        assert_eq!(
            TextSpan {
                start: 0,
                end: 9,
                content: SpanContent::Spans(vec![
                    TextSpan {
                        start: 0,
                        end: 3,
                        content: SpanContent::Text("0123"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 4,
                        end: 6,
                        content: SpanContent::Text("456"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 7,
                        end: 9,
                        content: SpanContent::Text("789"),
                        wraps: vec![],
                    },
                ]),
                wraps: vec![],
            },
            span
        );
    }

    #[test]
    fn test_split_single_char_middle() {
        let input = String::from("0123456789");

        let mut span = TextSpan::create(&input);
        span.get_sub_span_mut(5, 5);

        assert_eq!(
            TextSpan {
                start: 0,
                end: 9,
                content: SpanContent::Spans(vec![
                    TextSpan {
                        start: 0,
                        end: 4,
                        content: SpanContent::Text("01234"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 5,
                        end: 5,
                        content: SpanContent::Text("5"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 6,
                        end: 9,
                        content: SpanContent::Text("6789"),
                        wraps: vec![],
                    },
                ]),
                wraps: vec![],
            },
            span
        );
    }

    #[test]
    fn test_split_second_layer() {
        let input = String::from("0123456789");

        let mut span = TextSpan::create(&input);
        span.get_sub_span_mut(3, 7);

        assert_eq!(
            TextSpan {
                start: 0,
                end: 9,
                content: SpanContent::Spans(vec![
                    TextSpan {
                        start: 0,
                        end: 2,
                        content: SpanContent::Text("012"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 3,
                        end: 7,
                        content: SpanContent::Text("34567"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 8,
                        end: 9,
                        content: SpanContent::Text("89"),
                        wraps: vec![],
                    },
                ]),
                wraps: vec![],
            },
            span
        );

        span.get_sub_span_mut(5, 6);

        assert_eq!(
            TextSpan {
                start: 0,
                end: 9,
                content: SpanContent::Spans(vec![
                    TextSpan {
                        start: 0,
                        end: 2,
                        content: SpanContent::Text("012"),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 3,
                        end: 7,
                        content: SpanContent::Spans(vec![
                            TextSpan {
                                start: 3,
                                end: 4,
                                content: SpanContent::Text("34"),
                                wraps: vec![],
                            },
                            TextSpan {
                                start: 5,
                                end: 6,
                                content: SpanContent::Text("56"),
                                wraps: vec![],
                            },
                            TextSpan {
                                start: 7,
                                end: 7,
                                content: SpanContent::Text("7"),
                                wraps: vec![],
                            },
                        ]),
                        wraps: vec![],
                    },
                    TextSpan {
                        start: 8,
                        end: 9,
                        content: SpanContent::Text("89"),
                        wraps: vec![],
                    },
                ]),
                wraps: vec![],
            },
            span
        );
    }
}
