use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};

mod printer;
use printer::Printer;

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
            let new_list = comrak::nodes::NodeList {
                list_type: ListType::Ordered,
                start: 1,
                delimiter: node_list.delimiter,
                bullet_char: node_list.bullet_char,
                tight: node_list.tight,
                is_task_list: node_list.is_task_list,
                marker_offset: node_list.marker_offset,
                padding: node_list.padding,
            };
            node.data.borrow_mut().value = NodeValue::List(new_list);
        }
    }
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
}
