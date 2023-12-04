use similar::{Algorithm, DiffOp};

use crate::tokenize::{tokenize, HtmlTokenWithDiff, SimplifiedDiffTag};

pub(crate) fn diff_tokens_from_source(old_html: &str, new_html: &str) -> Vec<HtmlTokenWithDiff> {
    let old_tokens = tokenize(old_html);
    let new_tokens = tokenize(new_html);
    let diff = similar::capture_diff_slices(Algorithm::Patience, &old_tokens, &new_tokens);

    let mut unified = Vec::with_capacity(old_tokens.len() + new_tokens.len());
    for op in diff {
        match op {
            DiffOp::Equal {
                old_index: _,
                new_index,
                len,
            } => unified.extend(
                // prefer new_tokens over old_tokens because the final HTML will
                // be built from the new source code
                new_tokens[new_index..new_index + len]
                    .iter()
                    .map(|t| t.clone_with_diff_tag(SimplifiedDiffTag::Equal)),
            ),
            DiffOp::Delete {
                old_index,
                old_len,
                new_index: _,
            } => unified.extend(
                old_tokens[old_index..old_index + old_len]
                    .iter()
                    .map(|t| t.clone_with_diff_tag(SimplifiedDiffTag::Delete)),
            ),
            DiffOp::Insert {
                old_index: _,
                new_index,
                new_len,
            } => unified.extend(
                new_tokens[new_index..new_index + new_len]
                    .iter()
                    .map(|t| t.clone_with_diff_tag(SimplifiedDiffTag::Insert)),
            ),
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                unified.extend(
                    old_tokens[old_index..old_index + old_len]
                        .iter()
                        .map(|t| t.clone_with_diff_tag(SimplifiedDiffTag::Delete)),
                );
                unified.extend(
                    new_tokens[new_index..new_index + new_len]
                        .iter()
                        .map(|t| t.clone_with_diff_tag(SimplifiedDiffTag::Insert)),
                );
            }
        }
    }
    unified
}
