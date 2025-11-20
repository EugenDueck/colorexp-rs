use clap::Parser;
use regex::RegexBuilder;
use std::io;
use std::io::BufRead;
use std::process::exit;

/// Command line multicolor regexp highlighter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // // Do not color by changing the background color
    // #[arg(short = 'h', long)]
    // no_highlight: bool,

    // Only color by changing the background color
    #[arg(short = 'H', long)]
    only_highlight: bool,

    /// Perform case-insensitive matching
    #[arg(short, long)]
    ignore_case: bool,

    /// Patterns
    #[arg(num_args = 1..)]
    patterns: Vec<String>,
}

static FOREGROUND_COLORS: &[&str] = &[
    //"\x1b[30m", // Black
    "\x1b[31m", // Red
    "\x1b[32m", // Green
    "\x1b[33m", // Yellow
    "\x1b[34m", // Blue
    "\x1b[35m", // Magenta
    "\x1b[36m", // Cyan
                //"\x1b[37m", // White
];

static BACKGROUND_COLORS: &[&str] = &[
    //"\x1b[40m", // Black
    "\x1b[41m", // Red
    "\x1b[44m", // Blue
    "\x1b[45m", // Magenta
    "\x1b[42m", // Green
    "\x1b[43m", // Yellow
    "\x1b[46m", // Cyan
                //"\x1b[47m", // White
];

const RESET_FOREGROUND: &str = "\x1b[0m";
const RESET_BACKGROUND: &str = "\x1b[49m";

fn main() {
    let args = Args::parse();

    if args.patterns.len() > 1 {
        eprintln!("Only one pattern supported for now. Sorry!");
        exit(1);
    }
    let pattern = args.patterns.first().unwrap();
    let re = RegexBuilder::new(pattern)
        .case_insensitive(args.ignore_case)
        .build()
        .unwrap();
    let stdin = io::stdin();

    let hi = args.only_highlight;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let rep = format!(
            "{}$0{}",
            if hi {
                BACKGROUND_COLORS[0]
            } else {
                FOREGROUND_COLORS[0]
            },
            if hi {
                RESET_BACKGROUND
            } else {
                RESET_FOREGROUND
            },
        );
        let out = re.replace_all(&line, rep);
        println!("{out}");
    }
}
