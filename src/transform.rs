use crate::printer::Printer;
use comrak::nodes::{AstNode, ListType, NodeList, NodeValue};
use comrak::{Arena, Options, parse_document};
use regex::{Captures, Regex};

/// Transforms markdown content by converting unordered lists to ordered lists 
/// within magic comment blocks and replacing current position expressions.
///
/// This function parses the input markdown, processes it to convert bullet lists
/// to numbered lists when they appear between `<!-- ol -->` and `<!-- /ol -->` 
/// comment blocks, and replaces expressions like `(cur±N)` with actual numbers.
///
/// # Arguments
///
/// * `input` - A string slice containing the markdown content to transform
///
/// # Returns
///
/// A `String` containing the transformed markdown content
///
/// # Examples
///
/// ```
/// use md_ol_util::transform;
///
/// let input = r#"<!-- ol -->
/// - First item
/// - Second item with (cur-1) reference
/// <!-- /ol -->"#;
///
/// let result = transform(input);
/// // Result will be:
/// // <!-- ol -->
/// // 1. First item  
/// // 2. Second item with (1) reference
/// // <!-- /ol -->
/// ```
pub fn transform(input: &str) -> String {
    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, input, &options);
    transform_ast(root);
    let mut printer = Printer::new();
    printer.render_node(root);
    printer.finish()
}

fn transform_ast<'a>(node: &'a AstNode<'a>) {
    // Process children first
    for child in node.children() {
        transform_ast(child);
    }

    transform_ul(node);
}

fn transform_ul<'a>(parent: &'a AstNode<'a>) {
    let children: Vec<&AstNode> = parent.children().collect();
    let mut convert_mode = false;
    let mut nodes_to_convert = Vec::new();

    for child in children {
        if let NodeValue::HtmlBlock(html_block) = &child.data.borrow().value {
            let content = html_block.literal.trim();
            if content == "<!-- ol -->" {
                convert_mode = true;
            } else if content == "<!-- /ol -->" {
                convert_mode = false;
            }
        } else if convert_mode {
            if let NodeValue::List(list_data) = &child.data.borrow().value {
                if list_data.list_type == ListType::Bullet {
                    nodes_to_convert.push(child);
                }
            }
        }
    }

    for node in nodes_to_convert {
        let node_list_clone = {
            if let NodeValue::List(list_data) = &node.data.borrow().value {
                Some(list_data.clone())
            } else {
                None
            }
        };

        if let Some(node_list) = node_list_clone {
            let start = 1;
            let new_list = NodeList {
                list_type: ListType::Ordered,
                start,
                delimiter: node_list.delimiter,
                bullet_char: node_list.bullet_char,
                tight: node_list.tight,
                is_task_list: node_list.is_task_list,
                marker_offset: node_list.marker_offset,
                padding: node_list.padding,
            };
            node.data.borrow_mut().value = NodeValue::List(new_list);

            // After converting to ordered list, replace (cur-N) with actual numbers
            replace_cur_expressions_in_list(node, start);
        }
    }
}

fn replace_cur_expressions_in_list<'a>(list_node: &'a AstNode<'a>, start: usize) {
    let mut item_number = start as i32;

    for item in list_node.children() {
        if let NodeValue::Item(_) = &item.data.borrow().value {
            let mut stack = Vec::new();
            stack.push(item);

            while let Some(node) = stack.pop() {
                let new_text_opt = {
                    if let NodeValue::Text(text) = &node.data.borrow().value {
                        let new_text = replace_cur(text, item_number);
                        if new_text != *text {
                            Some(new_text)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(new_text) = new_text_opt {
                    node.data.borrow_mut().value = NodeValue::Text(new_text);
                }

                for child in node.children() {
                    stack.push(child);
                }
            }

            item_number += 1;
        }
    }
}

fn replace_cur(text: &str, current_item_number: i32) -> String {
    let re = Regex::new(r"\(cur([+-]\d+)\)").unwrap();
    re.replace_all(text, |caps: &Captures| {
        let offset_str = &caps[1];
        if let Ok(offset) = offset_str.parse::<i32>() {
            let result = current_item_number + offset;
            format!("({})", result)
        } else {
            caps[0].to_string() // Return original if parsing fails
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unordered_lists_unchanged_by_default() {
        let input = r#"- First item
- Second item
- Third item"#;
        let expected = r#"- First item
- Second item
- Third item
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_magic_comment_conversion() {
        let input = r#"<!-- ol -->
- First item
- Second item
- Third item
<!-- /ol -->"#;
        let expected = r#"<!-- ol -->
1. First item
2. Second item
3. Third item

<!-- /ol -->
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_mixed_content_with_magic_comments() {
        let input = r#"# Header

Normal list (unchanged):
- First item
- Second item

<!-- ol -->
- Third item
- Fourth item
<!-- /ol -->

Some text"#;
        let expected = r#"# Header

Normal list (unchanged):

- First item
- Second item

<!-- ol -->
1. Third item
2. Fourth item

<!-- /ol -->
Some text
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_indented_lists_with_magic_comments() {
        let input = r#"<!-- ol -->
  - Indented item
  - Another indented item
<!-- /ol -->"#;
        let expected = r#"<!-- ol -->
  1. Indented item
  2. Another indented item

<!-- /ol -->
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_no_unordered_lists() {
        let input = r#"Just some text
with no lists"#;
        let expected = r#"Just some text
with no lists
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_cur_minus_one_replacement() {
        let input = r#"<!-- ol -->
- First item
- Second item
- Third item with (cur-1) reference
<!-- /ol -->"#;
        let expected = r#"<!-- ol -->
1. First item
2. Second item
3. Third item with (2) reference

<!-- /ol -->
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_cur_expressions_various_offsets() {
        let input = r#"<!-- ol -->
- First item
- Second item with (cur-1) and (cur+1)
- Third item with (cur-2) and (cur+0)
- Fourth item with (cur-3)
<!-- /ol -->"#;
        let expected = r#"<!-- ol -->
1. First item
2. Second item with (1) and (3)
3. Third item with (1) and (3)
4. Fourth item with (1)

<!-- /ol -->
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_cur_expressions_no_magic_comments() {
        let input = r#"- First item with (cur-1)
- Second item with (cur+1)"#;
        let expected = r#"- First item with (cur-1)
- Second item with (cur+1)
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_cur_expressions_edge_cases() {
        let input = r#"<!-- ol -->
- First item with (cur-1) should be (0)
- Second item with (cur+0) should be (2)
- Third item with (cur-10) should be (-7)
<!-- /ol -->"#;
        let expected = r#"<!-- ol -->
1. First item with (0) should be (0)
2. Second item with (2) should be (2)
3. Third item with (-7) should be (-7)

<!-- /ol -->
"#;
        assert_eq!(transform(input), expected);
    }
}
