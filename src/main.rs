use md_ol_util::convert_unordered_to_ordered;

fn main() {
    let sample_markdown = r#"# My Document

Here are some items:

- First item
- Second item  
- Third item

Some more text here.

* Another list
+ With different bullets
- Mixed together"#;

    println!("Original markdown:");
    println!("{}", sample_markdown);
    println!("\nConverted markdown:");
    println!("{}", convert_unordered_to_ordered(sample_markdown));
}
