use itertools::Itertools;

use crate::tokenize::{self, HtmlViDiffToken};

// oh god oh fuck oh god oh fuck this code is so fucking horrible oh god oh no

// we're not even dealing with tags being moved and it's horrible
// <h2>some text</h2> -> some <h2>text</h2> will act like the whole <h2>text</h2> is new which it isnt aaaaaaaaa

// okay so our main issue is that we need to somehow track tags to  check if
// they are moved somewhere else or actually changed.
// i want a special <ins> class (that in css maybe only draws a border around or
// a lighter background colour or something like that) for when only an element's
// attributes have changed.
// if we can manage to track elements and see that a start or end tag has been
// moved over to somewhere we can highlight only the extra elements that are now
// added.
// the real difficult part is probably deciding when a start tag has been moved
// because it can have attributes and we kind of want to ignore those i guess?
// like <p>some</p> text -> <p class="red">some text</p>   we ultimately want to
// highlight as <ins class="-attr-change"><p class="red">some <ins
// class="-move-into">text</ins></p></ins>

// sidenote i'm not too sure on whether or not wrapping things in <ins> tags
// will lead to layout issues so we might want to (optionally perhaps) add a
// class or data- attribute to changed start/end tags instead of wrapping them
// in <ins>s.

// this will require a lot of rethinking..............

// TODO: Add doc comment for this function
pub fn highlight_changes(old_html: &str, new_html: &str) -> String {
    let old_tokens = tokenize::tokenize(old_html);
    let mut new_tokens = tokenize::tokenize(new_html);
    let diff = similar::capture_diff_slices(similar::Algorithm::Patience, &old_tokens, &new_tokens);

    let mut out = String::new();

    let highlight_insert = |out: &mut String,
                            new_tokens: &mut Vec<HtmlViDiffToken>,
                            new_index,
                            new_len| {
        for offset in 0..new_len {
            let new_token: &HtmlViDiffToken = &new_tokens[new_index + offset];
            match &new_token.kind {
                tokenize::HtmlViDiffTokenKind::StringSegment(s) => {
                    // FIXME: is this wrong? like ther'es no nesting level now
                    // anymoroe ebut i dont eeven think we can implement tahat
                    // is impossile it is it impossiible?? idk idk what to do god
                    if !new_token.has_been_inserted() {
                    // TODO: Wrap consecutive segments in a single <ins>
                    *out += &format!("<ins>{}</ins>", s);
                    let new_token_mut: &mut HtmlViDiffToken = &mut new_tokens[new_index + offset];
                    new_token_mut.mark_inserted();
                    }
                }
                tokenize::HtmlViDiffTokenKind::StartTag {
                    name, self_closing, ..
                } => {
                    // TODO: Do a has_been_inserted() check here but I guess use
                    // it  to w  wijf  ieijedj iwdn work on sos  eie   ieieieiei
                    // do the sos the SOS shte the dow use it dot tod ot dto
                    // check if the level neseting lseenesting level increase
                    // aaa i dont knww knw nkown know www sktis this is never
                    // going to workkk aaaaaaaa  dmmaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa

                    // Find the matching closing tag if it's not
                    // self-closing or a void tag.
                    // REFACTOR: Move this into it's own function.
                    if *self_closing || new_token.kind.start_tag_is_void() {
                        // TODO: Add nesting level class to <ins>
                        *out += &format!("<ins>{}</ins>", &new_html[new_token.span.clone()]);
                        let new_token_mut: &mut HtmlViDiffToken =
                            &mut new_tokens[new_index + offset];
                        new_token_mut.mark_inserted();
                    } else {
                        let mut n_open: u32 = 1;
                        if let Some((end_tag_token_offset, end_tag_token)) = new_tokens
                            [new_index + offset + 1..]
                            .into_iter()
                            .find_position(|other_token| match &other_token.kind {
                                tokenize::HtmlViDiffTokenKind::StartTag {
                                    name: other_name,
                                    self_closing,
                                    ..
                                } if !self_closing && other_name == name => {
                                    n_open += 1;
                                    false
                                }
                                tokenize::HtmlViDiffTokenKind::EndTag(other_name)
                                    if other_name == name =>
                                {
                                    n_open -= 1;
                                    n_open == 0
                                }
                                _ => false,
                            })
                        {
                            // TODO: Add nesting level class to <ins>
                            *out += &format!(
                                "<ins>{}</ins>",
                                &new_html[new_token.span.start..end_tag_token.span.end]
                            );

                            // Can we find a way to not have to iterate over all
                            // tokens between the start and end tag again?
                            for token in &mut new_tokens
                                // somebody please kill me
                                [new_index + offset..=new_index + offset + 1 + end_tag_token_offset]
                            {
                                token.mark_inserted();
                            }
                        } else {
                            // No matching end tag token was found.

                            // FIXME: Remove this println!
                            println!(
                                "WARN: No matching end tag for <{}> at {:?}",
                                name, new_token.span
                            );

                            // REVIEW: Not sure what to do here. We might want
                            // to return something indicating that the new HTML
                            // is invalid, but we do still want to at least
                            // highlight the changes we can highlight instead of
                            // failing altogether I think.

                            // Since browsers will act as if the tag was closed
                            // at the end of the input I suppose we could simply
                            // insert the </ins> tag at the end of the input,
                            // but that'll likely lead to huge portions of the
                            // document to be highlighted which is not what we
                            // want.

                            // Instead of failing completely we could also stop
                            // highlight changes after this and return all
                            // current highlights + the rest of the HTML input
                            // as part of the error part of a Result<String,
                            // String> or a custom enum like
                            // HighlightResult::Incomplete(String).

                            // For now I'll leave it at skip this and silently
                            // continue.
                        }
                    }
                }
                tokenize::HtmlViDiffTokenKind::EndTag(name) => {
                    if !new_token.has_been_inserted() {
                        // REFACTOR: Code duplication, this is almost
                        // exactly the same as the code above for finding
                        // the matching end tag, but in reverse.
                        let mut n_closed: u32 = 1;
                        if let Some((start_tag_token_pos, start_tag_token)) = new_tokens
                            [..new_index + offset]
                            .into_iter()
                            .rev()
                            .find_position(|other_token| match &other_token.kind {
                                tokenize::HtmlViDiffTokenKind::StartTag {
                                    name: other_name,
                                    self_closing,
                                    ..
                                } if !self_closing && other_name == name => {
                                    n_closed -= 1;
                                    n_closed == 0
                                }
                                tokenize::HtmlViDiffTokenKind::EndTag(other_name)
                                    if other_name == name =>
                                {
                                    n_closed += 1;
                                    false
                                }
                                _ => false,
                            })
                        {
                            // TODO: Add nesting level class to <ins>
                            *out += &format!(
                                "<ins>{}</ins>",
                                &new_html[start_tag_token.span.start..new_token.span.end]
                            );

                            // Can we find a way to not have to iterate over all
                            // tokens between the start and end tag again?
                            for token in &mut new_tokens[start_tag_token_pos..=new_index + offset] {
                                token.mark_inserted();
                            }
                        } else {
                            // No matching start tag found.

                            // FIXME: Remove this println!
                            println!(
                                "WARN: No matching start tag for </{}> at {:?}",
                                name, new_token.span
                            );

                            // REVIEW: Same as with no matching end tag.
                        }
                    }
                }
            }
        }
    };

    for diff_op in diff {
        match diff_op {
            similar::DiffOp::Equal {
                old_index: _,
                new_index,
                len,
            } => {
                for offset in 0..len {
                    let new_token = &new_tokens[new_index + offset];
                    if !new_token.has_been_inserted() {
                        out += &new_html[new_token.span.clone()];
                    }
                }
            }
            similar::DiffOp::Delete {
                old_index: _,
                old_len: _,
                new_index: _,
            } => {
                // Might want to add code later to *optionally* highlight
                // deletions.
            }
            similar::DiffOp::Insert {
                old_index: _,
                new_index,
                new_len,
            } => highlight_insert(&mut out, &mut new_tokens, new_index, new_len),
            similar::DiffOp::Replace {
                old_index: _,
                old_len: _,
                new_index,
                new_len,
            } => highlight_insert(&mut out, &mut new_tokens, new_index, new_len),
        }
    }

    out
}
