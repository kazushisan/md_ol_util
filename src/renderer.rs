use comrak::nodes::{AstNode, NodeValue};

pub struct MarkdownRenderer {
    output: String,
    list_stack: Vec<i32>, // Track list item counters for nested lists
}

impl MarkdownRenderer {
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
                if list_data.list_type == comrak::nodes::ListType::Ordered {
                    self.list_stack.push(list_data.start as i32);
                    for child in node.children() {
                        self.render_node(child);
                    }
                    self.list_stack.pop();
                    if !self.is_in_list() && self.should_add_blank_line_after_list(node) {
                        self.output.push('\n');
                    }
                }
            }
            NodeValue::Item(_) => {
                if let Some(counter_val) = self.list_stack.last().copied() {
                    // Get indentation from source position if available
                    let indent = self.get_item_indentation(node);
                    self.output
                        .push_str(&format!("{}{}. ", indent, counter_val));

                    // Update counter after using it
                    if let Some(counter) = self.list_stack.last_mut() {
                        *counter += 1;
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