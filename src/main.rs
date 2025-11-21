use anyhow::Result;
use clap::{ArgAction, ArgGroup, Parser};
use regex::RegexBuilder;
use std::io;
use std::io::BufRead;
use std::process::exit;

/// Command line multicolor regexp highlighter
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None,
    disable_help_flag = true,
    group(
        ArgGroup::new("highlight")
            .args(&["no_highlight", "only_highlight"])
            .multiple(false)
    ),
)]
struct Args {
    // using custom help arg to be able to turn off -h, which is used by the no_highlight arg
    /// Show help
    #[arg(long = "help", action = ArgAction::Help)]
    help: Option<bool>,

    /// More verbose output on errors
    #[arg(long)]
    debug: bool,

    /// Do not color by changing the background color
    #[arg(short = 'h', long)]
    no_highlight: bool,

    /// Only color by changing the background color
    #[arg(short = 'H', long)]
    only_highlight: bool,

    /// Perform case-insensitive matching
    #[arg(short, long)]
    ignore_case: bool,

    /// Patterns
    #[arg(required = true, num_args = 1..)]
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

    if args.debug {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "full");
        }
    }

    if let Err(err) = run(&args) {
        if args.debug {
            eprintln!("{err:?}");
        } else {
            eprintln!("{err}");

            let mut source = err.source();
            while let Some(cause) = source {
                eprintln!("  Caused by: {cause}");
                source = cause.source();
            }
        }

        exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    let regexps = args
        .patterns
        .iter()
        // reverse order, so that the last given regex that matches takes precedence
        .rev()
        .map(|p| {
            RegexBuilder::new(p)
                .case_insensitive(args.ignore_case)
                .build()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let stdin = io::stdin();

    let colors = {
        let mut colors = Vec::new();
        if !args.only_highlight {
            for c in FOREGROUND_COLORS {
                colors.push((c, RESET_FOREGROUND));
            }
        }
        if !args.no_highlight {
            for c in BACKGROUND_COLORS {
                colors.push((c, RESET_BACKGROUND));
            }
        }
        colors
    };

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let rep = format!("{}$0{}", colors[0].0, colors[0].1);
        let out = regexps[0].replace_all(&line, rep);
        println!("{out}");
    }
    Ok(())
}
