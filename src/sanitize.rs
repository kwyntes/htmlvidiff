use html5ever::{serialize::SerializeOpts, tendril::TendrilSink, ParseOpts};
use markup5ever_rcdom::{RcDom, SerializableHandle};

/// Parses and re-serializes an HTML document using [`html5ever`].
pub fn sanitize(html: &str) -> String {
    let dom = html5ever::parse_document(RcDom::default(), ParseOpts::default()).one(html);
    let mut bytes = vec![];
    html5ever::serialize(
        &mut bytes,
        &SerializableHandle::from(dom.document),
        SerializeOpts::default(),
    )
    // pretty sure this can't fail
    .unwrap();
    String::from_utf8(bytes).unwrap()
}
