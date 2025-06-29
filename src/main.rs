use clap::Parser;
use md_ol_util::transform;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "md_ol_util")]
#[command(
    about = "Transform markdown unordered lists to ordered lists with current position expressions"
)]
#[command(version)]
struct Args {
    #[arg(help = "Input markdown file. If not provided, reads from stdin")]
    file: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let input = match args.file {
        Some(file_path) => fs::read_to_string(file_path)?,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let transformed = transform(&input);
    print!("{}", transformed);

    Ok(())
}
