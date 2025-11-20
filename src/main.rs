use clap::Parser;

/// Command line multicolor regexp highlighter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Perform case-insensitive matching
    #[arg(short, long)]
    ignore_case: bool,
}

fn main() {
    let args = Args::parse();

    println!("Hello! {}", args.ignore_case);
}
