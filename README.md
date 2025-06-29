# md_ol_util

## Install

```bash
cargo install md_ol_util
```

## CLI

```
Transform markdown unordered lists to ordered lists with current position expressions

Usage: md_ol_util [FILE]

Arguments:
  [FILE]  Input markdown file. If not provided, reads from stdin

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Usage

```rust
use md_ol_util::transform;

let input = r#"<!-- ol -->
- First item
- Second item with (cur-1) reference
<!-- /ol -->"#;

let result = transform(input);
// Result will be:
// <!-- ol -->
// 1. First item  
// 2. Second item with (1) reference
// <!-- /ol -->
```
