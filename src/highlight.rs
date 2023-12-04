use crate::tokendiff::diff_tokens_from_source;

/// Highlights the changes in the new document using the algorithm described in
/// README.md.
///
/// Expects 'sane' HTML (no further elaboration), for which I
/// recommend you to run both inputs through the sanitize function provided by
/// this library through the `sanitization` feature.
pub fn highlight_changes(old_html: &str, new_html: &str) -> String {
    let diff = diff_tokens_from_source(old_html, new_html);

    //
}
