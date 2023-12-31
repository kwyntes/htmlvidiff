use std::{fmt::Debug, hash::Hash, ops::Range};

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

pub fn tokenize(html: &str) -> Vec<HtmlToken> {
    let mut parser =
        NaiveParser::new_with_emitter(PosTrackingReader::new(html), TracingEmitter::default())
            .flatten();

    let mut tokens = Vec::new();
    let mut push_token = |span, kind| tokens.push(HtmlToken::new(span, kind));
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
                            HtmlTokenKind::StringSegment(strbuf),
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
                        HtmlTokenKind::StringSegment(strbuf),
                    );
                }
                match token {
                    html5tokenizer::Token::Char(_) => unreachable!(),
                    html5tokenizer::Token::StartTag(start_tag) => push_token(
                        token_span,
                        HtmlTokenKind::StartTag {
                            name: start_tag.name,
                            self_closing: start_tag.self_closing,
                            attrs: collect_attributes_normalized(start_tag.attributes),
                        },
                    ),
                    html5tokenizer::Token::EndTag(end_tag) => {
                        push_token(token_span, HtmlTokenKind::EndTag(end_tag.name));
                    }
                    _ => {}
                }
            }
        }
    }

    tokens
}

pub struct HtmlToken {
    pub span: Range<usize>,
    pub kind: HtmlTokenKind,
}
impl HtmlToken {
    fn new(span: Range<usize>, kind: HtmlTokenKind) -> Self {
        Self { span, kind }
    }

    pub(crate) fn clone_with_diff_tag(&self, diff_tag: SimplifiedDiffTag) -> HtmlTokenWithDiff {
        HtmlTokenWithDiff {
            diff_tag,
            span: self.span.clone(),
            kind: self.kind.clone(),
        }
    }
}

impl Debug for HtmlToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlTokenKind::StringSegment(s) => write!(f, "{:?}", s)?,
            HtmlTokenKind::StartTag {
                name,
                self_closing,
                attrs,
            } => write!(
                f,
                "<{}{}> with attrs {:#?}",
                name,
                self_closing.then_some("/").unwrap_or_default(),
                attrs
            )?,
            HtmlTokenKind::EndTag(name) => write!(f, "</{}>", name)?,
        }
        write!(f, " at {:?}", self.span)
    }
}
impl Hash for HtmlToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}
impl PartialEq for HtmlToken {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
impl Eq for HtmlToken {}

// NOTE: As of now, the Ord trait is required for capture_diff_slices
// from the similar crate, but none of its methods are used there yet.
// (see https://github.com/mitsuhiko/similar/issues/50)
// Since tokens cannot be greater than or less than each other, it is not
// possible to actually implement Ord, but since it won't be used anyway we can
// just leave this unimplemented.
impl PartialOrd for HtmlToken {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        unimplemented!()
    }
}
impl Ord for HtmlToken {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        unimplemented!()
    }
}

pub(crate) struct HtmlTokenWithDiff {
    pub span: Range<usize>,
    pub kind: HtmlTokenKind,
    pub diff_tag: SimplifiedDiffTag,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum HtmlTokenKind {
    StringSegment(String),
    StartTag {
        name: String,
        self_closing: bool,
        attrs: Box<[(String, String)]>,
    },
    EndTag(String),
}
impl HtmlTokenKind {
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

/// "Simplified" because [`similar::DiffTag`] also has a `Replace` variant,
/// which is the same as a `Delete` followed by an `Insert` (or the other way
/// around, whatever you like).
pub(crate) enum SimplifiedDiffTag {
    Equal,
    Delete,
    Insert,
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
