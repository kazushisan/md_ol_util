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
    for child in node.children() {
        transform_ast(child);
    }

    convert_bullet_to_ordered(node);
}

fn convert_bullet_to_ordered<'a>(node: &'a AstNode<'a>) {
    if let NodeValue::List(ref mut list_data) = node.data.borrow_mut().value {
        if list_data.list_type == ListType::Bullet {
            list_data.list_type = ListType::Ordered;
            list_data.start = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_unordered_to_ordered() {
        let input = r#"- First item
- Second item
- Third item"#;
        let expected = r#"1. First item
2. Second item
3. Third item
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_mixed_content() {
        let input = r#"# Header

- First item
- Second item

Some text"#;
        let expected = r#"# Header

1. First item
2. Second item

Some text
"#;
        assert_eq!(transform(input), expected);
    }

    #[test]
    fn test_indented_lists() {
        let input = r#"  - Indented item
  - Another indented item"#;
        let expected = r#"  1. Indented item
  2. Another indented item
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
