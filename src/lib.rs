pub mod highlight;
mod tokendiff;
pub mod tokenize;

#[cfg(feature = "sanitization")]
pub mod sanitize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_attr_order_changed() {
        let old = r#"<div id="a-div" class="container"></div>"#;
        let new = r#"<div class="container" id="a-div"></div>"#;

        assert_eq!(
            similar::capture_diff_slices(
                similar::Algorithm::Patience,
                &tokenize::tokenize(old),
                &tokenize::tokenize(new)
            ),
            [similar::DiffOp::Equal {
                old_index: 0,
                new_index: 0,
                len: 2
            }]
        );
    }

    #[test]
    fn diff_class_order_changed() {
        let old = r#"<div class="cls1  cls2"></div>"#;
        let new = r#"<div class=" cls2 cls1 "></div>"#;

        assert_eq!(
            similar::capture_diff_slices(
                similar::Algorithm::Patience,
                &tokenize::tokenize(old),
                &tokenize::tokenize(new)
            ),
            [similar::DiffOp::Equal {
                old_index: 0,
                new_index: 0,
                len: 2
            }]
        );
    }

    #[test]
    fn diff_style_order_changed() {
        let old = r#"<span style="font-size: 16px; color: #fff"></span>"#;
        let new = r#"<span style="color:#fff;  font-size: 16px "></span>"#;

        println!("{:#?}", tokenize::tokenize(new));

        assert_eq!(
            similar::capture_diff_slices(
                similar::Algorithm::Patience,
                &tokenize::tokenize(old),
                &tokenize::tokenize(new)
            ),
            [similar::DiffOp::Equal {
                old_index: 0,
                new_index: 0,
                len: 2
            }]
        );
    }

    #[test]
    fn highlight_test_a() {
        let old = r##"<h2>some text &with; &lt; <a href="#">a link</a></h2>"##;
        let new = r##"<h3>some</h3><h2>more text &without; &lt; <a href="#">any link</a></h2>"##;

        assert_eq!(
            highlight::highlight_changes(old, new),
            r##"<ins><h3>some</h3></ins><h2><ins>more</ins> text <ins>&without;</ins> &lt; <a href="#"><ins>any</ins> link</a></h2>"##
        );
    }
}
