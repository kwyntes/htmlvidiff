use htmlvidiff::tokenize::{self, HtmlViDiffToken};
use similar::DiffOp;

#[allow(unused)]
fn debug_diff(
    diff: Vec<DiffOp>,
    old_tokens: Vec<HtmlViDiffToken>,
    new_tokens: Vec<HtmlViDiffToken>,
) {
    println!("=== TOKEN DIFF ===");
    for op in diff {
        match op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                // println!("Equal ({} L{} -> {} L{})", old_index, len, new_index, len);
                for offset in 0..len {
                    println!("  {:#?}", new_tokens[new_index + offset]);
                }
            }
            DiffOp::Delete {
                old_index,
                old_len,
                new_index,
            } => {
                // println!("Delete ({} L{} -> {} L0)", old_index, old_len, new_index);
                for offset in 0..old_len {
                    println!("- {:#?}", old_tokens[old_index + offset]);
                }
            }
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                // println!("Insert ({} L0 -> {} L{})", old_index, new_index, new_len);
                for offset in 0..new_len {
                    println!("+ {:#?}", new_tokens[new_index + offset]);
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                // println!(
                //     "Replace ({} L{} -> {} L{})",
                //     old_index, old_len, new_index, new_len
                // );
                for offset in 0..old_len {
                    println!("- {:#?}", old_tokens[old_index + offset]);
                }
                println!("with");
                for offset in 0..new_len {
                    println!("+ {:#?}", new_tokens[new_index + offset]);
                }
            }
        }
    }
}

fn diff_and_debug(old_html: &str, new_html: &str) {
    println!("=== OLD HTML ===\n{}\n", old_html);
    println!("=== NEW HTML ===\n{}\n", new_html);
    let old_tokens = tokenize::tokenize(old_html);
    let new_tokens = tokenize::tokenize(new_html);
    let diff = similar::capture_diff_slices(similar::Algorithm::Patience, &old_tokens, &new_tokens);
    debug_diff(diff, old_tokens, new_tokens);
}

fn main() {
    // let html = r##"<h2>some text &with; &lt; <a href="#">a link</a></h2>"##;

    // let tokens = tokenize::tokenize(html);

    // let more_html = r##"<h3>some</h3><h2>more text &without; &lt; <a href="#">any link</a></h2>"##;

    // let more_tokens = tokenize::tokenize(more_html);

    // let diff = similar::capture_diff_slices(similar::Algorithm::Patience, &tokens, &more_tokens);

    // debug_diff(diff, tokens, more_tokens);

    // println!("{:#?}", highlight::highlight_changes(html, more_html));

    // README.md|test case #2
    let old = "some <p>more text</p>";
    let new = "<p>some more</p> text";
    diff_and_debug(old, new);
}
