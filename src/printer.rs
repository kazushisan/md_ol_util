use comrak::nodes::{AstNode, NodeValue};

pub struct Printer {
    output: String,
    list_stack: Vec<i32>, // Track list item counters for nested lists
}

impl Printer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            list_stack: Vec::new(),
        }
    }

    pub fn finish(self) -> String {
        self.output.trim_end().to_string() + "\n"
    }

    pub fn render_node<'a>(&mut self, node: &'a AstNode<'a>) {
        match &node.data.borrow().value {
            NodeValue::Document => {
                for child in node.children() {
                    self.render_node(child);
                }
            }
            NodeValue::Heading(heading_data) => {
                self.output.push_str(&"#".repeat(heading_data.level.into()));
                self.output.push(' ');
                for child in node.children() {
                    self.render_node(child);
                }
                self.output.push('\n');
                if self.should_add_blank_line_after_heading(node) {
                    self.output.push('\n');
                }
            }
            NodeValue::Paragraph => {
                for child in node.children() {
                    self.render_node(child);
                }
                if !self.is_in_list() {
                    self.output.push('\n');
                    if self.should_add_blank_line_after_paragraph(node) {
                        self.output.push('\n');
                    }
                }
            }
            NodeValue::List(list_data) => {
                match list_data.list_type {
                    comrak::nodes::ListType::Ordered => {
                        self.list_stack.push(list_data.start as i32);
                        for child in node.children() {
                            self.render_node(child);
                        }
                        self.list_stack.pop();
                    }
                    comrak::nodes::ListType::Bullet => {
                        self.list_stack.push(-1); // Use -1 to indicate bullet list
                        for child in node.children() {
                            self.render_node(child);
                        }
                        self.list_stack.pop();
                    }
                }
                if !self.is_in_list() && self.should_add_blank_line_after_list(node) {
                    self.output.push('\n');
                }
            }
            NodeValue::Item(_) => {
                if let Some(counter_val) = self.list_stack.last().copied() {
                    // Get indentation from source position if available
                    let indent = self.get_item_indentation(node);
                    
                    if counter_val == -1 {
                        // Bullet list item
                        self.output.push_str(&format!("{}- ", indent));
                    } else {
                        // Ordered list item
                        self.output
                            .push_str(&format!("{}{}. ", indent, counter_val));
                        
                        // Update counter after using it
                        if let Some(counter) = self.list_stack.last_mut() {
                            *counter += 1;
                        }
                    }

                    for child in node.children() {
                        self.render_node(child);
                    }
                    self.output.push('\n');
                }
            }
            NodeValue::Text(text) => {
                self.output.push_str(text);
            }
            NodeValue::SoftBreak => {
                if self.is_in_list() {
                    self.output.push(' ');
                } else {
                    self.output.push('\n');
                }
            }
            NodeValue::LineBreak => {
                self.output.push_str("  \n");
            }
            NodeValue::HtmlBlock(html_block) => {
                self.output.push_str(&html_block.literal);
                if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            NodeValue::HtmlInline(html) => {
                self.output.push_str(html);
            }
            _ => {
                // Handle other node types as needed
                for child in node.children() {
                    self.render_node(child);
                }
            }
        }
    }

    fn is_in_list(&self) -> bool {
        !self.list_stack.is_empty()
    }

    fn should_add_blank_line_after_heading<'a>(&self, node: &'a AstNode<'a>) -> bool {
        node.next_sibling().is_some()
    }

    fn should_add_blank_line_after_paragraph<'a>(&self, node: &'a AstNode<'a>) -> bool {
        if let Some(next) = node.next_sibling() {
            matches!(
                next.data.borrow().value,
                NodeValue::List(_) | NodeValue::Heading(_)
            )
        } else {
            false
        }
    }

    fn should_add_blank_line_after_list<'a>(&self, node: &'a AstNode<'a>) -> bool {
        if let Some(next) = node.next_sibling() {
            !matches!(next.data.borrow().value, NodeValue::List(_))
        } else {
            false
        }
    }

    fn get_item_indentation<'a>(&self, node: &'a AstNode<'a>) -> String {
        // Check if this item has source position info that indicates indentation
        let start_column = node.data.borrow().sourcepos.start.column;
        if start_column > 1 {
            // Calculate indentation based on column position
            // Assuming each indentation level is 2 spaces and list markers start at column 1, 3, 5, etc.
            let indent_chars = if start_column > 1 {
                start_column - 1
            } else {
                0
            };
            " ".repeat(indent_chars)
        } else {
            // Default to no indentation
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{Arena, Options, parse_document};

    fn test_printer_output(input: &str, expected: &str) {
        let arena = Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, input, &options);
        
        let mut printer = Printer::new();
        printer.render_node(root);
        let result = printer.finish();
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_heading_node() {
        let input = r#"# Main Heading"#;
        let expected = r#"# Main Heading
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_heading_with_blank_line() {
        let input = r#"# Header

Some content"#;
        let expected = r#"# Header

Some content
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_paragraph_node() {
        let input = r#"This is a simple paragraph."#;
        let expected = r#"This is a simple paragraph.
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_text_node() {
        let input = r#"Plain text"#;
        let expected = r#"Plain text
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_ordered_list_node() {
        let input = r#"1. First item
2. Second item"#;
        let expected = r#"1. First item
2. Second item
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_unordered_list_node() {
        let input = r#"- First item
- Second item
- Third item"#;
        let expected = r#"- First item
- Second item
- Third item
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_mixed_list_types() {
        let input = r#"# Lists Example

Ordered list:
1. First ordered
2. Second ordered

Unordered list:
- First bullet
- Second bullet"#;
        let expected = r#"# Lists Example

Ordered list:

1. First ordered
2. Second ordered

Unordered list:

- First bullet
- Second bullet
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_soft_break_in_paragraph() {
        let input = r#"Line one
Line two"#;
        let expected = r#"Line one
Line two
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_multiple_headings() {
        let input = r#"# First

## Second

### Third"#;
        let expected = r#"# First

## Second

### Third
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_paragraph_before_list() {
        let input = r#"Introduction text

1. First item
2. Second item"#;
        let expected = r#"Introduction text

1. First item
2. Second item
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_empty_document() {
        let input = r#""#;
        let expected = r#"
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_nested_structure() {
        let input = r#"# Main

Paragraph text

1. Item one
2. Item two

End text"#;
        let expected = r#"# Main

Paragraph text

1. Item one
2. Item two

End text
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_html_comment() {
        let input = r#"<!-- This is an HTML comment -->"#;
        let expected = r#"<!-- This is an HTML comment -->
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_html_block() {
        let input = r#"<div class="example">
    <p>This is HTML content</p>
</div>"#;
        let expected = r#"<div class="example">
    <p>This is HTML content</p>
</div>
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_html_inline() {
        let input = r#"This is text with <em>inline HTML</em> content."#;
        let expected = r#"This is text with <em>inline HTML</em> content.
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_html_elements_passthrough() {
        let input = r#"Text with <a href="https://example.com">anchor</a> and <img src="image.jpg" alt="image"> tags."#;
        let expected = r#"Text with <a href="https://example.com">anchor</a> and <img src="image.jpg" alt="image"> tags.
"#;
        test_printer_output(input, expected);
    }

    #[test]
    fn test_multiline_html_comment() {
        let input = r#"<!--
This is a multi-line
HTML comment
-->"#;
        let expected = r#"<!--
This is a multi-line
HTML comment
-->
"#;
        test_printer_output(input, expected);
    }
}
