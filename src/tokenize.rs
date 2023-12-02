use std::{hash::Hash, ops::Range};

use html5tokenizer::{
    offset::PosTrackingReader, token::AttributeMap, trace::Trace, NaiveParser, TracingEmitter,
};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;

trait TraceGenericSpan {
    fn span(self) -> Range<usize>;
}
impl TraceGenericSpan for Trace {
    fn span(self) -> Range<usize> {
        match self {
            Trace::Char(span) => span,
            Trace::StartTag(trace) => trace.span,
            Trace::EndTag(trace) => trace.span,
            Trace::Comment(trace) => trace.data_span,
            Trace::Doctype(trace) => trace.span(),
            Trace::EndOfFile(pos) => pos..pos,
        }
    }
}

pub fn tokenize(html: &str) -> Vec<HtmlViDiffToken> {
    let mut parser =
        NaiveParser::new_with_emitter(PosTrackingReader::new(html), TracingEmitter::default())
            .flatten();

    let mut tokens = Vec::new();
    let mut push_token = |span, kind| tokens.push(HtmlViDiffToken::new(span, kind));
    let mut strbuf = None;
    let mut last_char_was_whitespace = false;
    let mut str_start = 0;
    while let Some((token, trace)) = parser.next() {
        let token_span = trace.span();

        match token {
            html5tokenizer::Token::Char(c) => {
                // TODO: Might want to document this code a bit.
                if c.is_whitespace() != last_char_was_whitespace {
                    if let Some(strbuf) = strbuf.take() {
                        push_token(
                            str_start..token_span.start,
                            HtmlViDiffTokenKind::StringSegment(strbuf),
                        );
                    }
                }
                if strbuf.is_none() {
                    strbuf = Some(String::new());
                    last_char_was_whitespace = c.is_whitespace();
                    str_start = token_span.start;
                }
                strbuf.as_mut().unwrap().push(c);
            }
            _ => {
                if let Some(strbuf) = strbuf.take() {
                    push_token(
                        str_start..token_span.start,
                        HtmlViDiffTokenKind::StringSegment(strbuf),
                    );
                }
                match token {
                    html5tokenizer::Token::Char(_) => unreachable!(),
                    html5tokenizer::Token::StartTag(start_tag) => push_token(
                        token_span,
                        HtmlViDiffTokenKind::StartTag {
                            name: start_tag.name,
                            self_closing: start_tag.self_closing,
                            attrs: collect_attributes_normalized(start_tag.attributes),
                        },
                    ),
                    html5tokenizer::Token::EndTag(end_tag) => {
                        push_token(token_span, HtmlViDiffTokenKind::EndTag(end_tag.name));
                    }
                    _ => {}
                }
            }
        }
    }

    tokens
}

#[derive(Debug)]
pub struct HtmlViDiffToken {
    pub span: Range<usize>,
    pub kind: HtmlViDiffTokenKind,

    has_been_inserted: bool,
}
impl HtmlViDiffToken {
    fn new(span: Range<usize>, kind: HtmlViDiffTokenKind) -> Self {
        Self {
            span,
            kind,
            has_been_inserted: false,
        }
    }

    // REVIEW: This is probably a bad way of doing it and we might also not be
    // seperating concerns properly anymore.

    /// Mark the token's source HTML as being included in the output string.
    pub fn mark_inserted(&mut self) {
        self.has_been_inserted = true;
    }

    /// Whether or not the token's source HTML has been included in the output string.
    pub fn has_been_inserted(&self) -> bool {
        self.has_been_inserted
    }
}

impl Hash for HtmlViDiffToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}
impl PartialEq for HtmlViDiffToken {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
impl Eq for HtmlViDiffToken {}

// NOTE: As of now, the Ord trait is required for capture_diff_slices
// from the similar crate, but none of its methods are used there yet.
// (see https://github.com/mitsuhiko/similar/issues/50)
// Since tokens cannot be greater than or less than each other, it is not
// possible to actually implement Ord, but since it won't be used anyway we can
// just leave this unimplemented.
impl PartialOrd for HtmlViDiffToken {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        unimplemented!()
    }
}
impl Ord for HtmlViDiffToken {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        unimplemented!()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum HtmlViDiffTokenKind {
    StringSegment(String),
    StartTag {
        name: String,
        self_closing: bool,
        attrs: Box<[(String, String)]>,
    },
    EndTag(String),
}
impl HtmlViDiffTokenKind {
    /// Checks if the start tag is a void tag.
    /// Assumes the token is a start tag, panics otherwise.
    pub fn start_tag_is_void(&self) -> bool {
        let Self::StartTag { name, .. } = self else {
            panic!();
        };

        match name.as_str() {
            // See https://developer.mozilla.org/en-US/docs/Glossary/Void_element
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "param" | "source" | "track" | "wbr" => true,
            _ => false,
        }
    }
}

/// Collects [`html5tokenizer::token::AttributeMap`] into a boxed slice of
/// key-value pairs, sorting values for class, style, etc. attributes.
fn collect_attributes_normalized(attributes: AttributeMap) -> Box<[(String, String)]> {
    attributes
        .into_iter()
        .map(|a| {
            let normalized_value = match a.name.as_str() {
                // TODO: There's probably more attributes we should normalize here.
                "class" => a.value.split_ascii_whitespace().sorted().join(" "),
                "style" => {
                    // NOTE: This is probably not sufficient for all cases but I
                    // don't think it's worth to include a full spec-compliant
                    // inline CSS parser just for this.
                    static RE_SPLIT_RULES: Lazy<Regex> =
                        Lazy::new(|| Regex::new(r"\s*;+\s*").unwrap());
                    static RE_TRIM_INNER: Lazy<Regex> =
                        Lazy::new(|| Regex::new(r"([a-zA-Z0-9_-]+)\s*:\s*(.+)").unwrap());
                    RE_SPLIT_RULES
                        .split(&a.value)
                        .map(|rule| RE_TRIM_INNER.replace(rule, "$1:$2"))
                        .sorted()
                        .join(" ")
                        .trim()
                        .to_owned()
                }
                _ => a.value,
            };

            (a.name, normalized_value)
        })
        .collect()
}
