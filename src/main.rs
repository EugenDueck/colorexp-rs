use anyhow::Result;
use clap::{ArgAction, ArgGroup, Parser};
use regex::{Regex, RegexBuilder};
use std::cmp::min;
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
    group(
        ArgGroup::new("vary_group_colors")
            .args(&["vary_group_colors_off", "vary_group_colors_on"])
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

    /// Highlight the entire match, even if pattern contains capturing groups
    #[arg(short, long)]
    full_match_highlight: bool,

    /// Perform case-insensitive matching
    #[arg(short, long)]
    ignore_case: bool,

    /// Do not color by changing the background color
    #[arg(short = 'h', long)]
    no_highlight: bool,

    /// Only color by changing the background color
    #[arg(short = 'H', long)]
    only_highlight: bool,

    /// Patterns
    #[arg(required = true, num_args = 1..)]
    patterns: Vec<String>,

    /// Turn off changing of colors for every capturing group. Defaults to on if exactly one pattern is given.
    #[arg(short = 'g', long)]
    vary_group_colors_off: bool,

    /// Turn on changing of colors for every capturing group. Defaults to on if exactly one pattern is given.
    #[arg(short = 'G', long)]
    vary_group_colors_on: bool,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct RangeWithId {
    start_idx: usize,
    end_idx: usize,
    id: usize,
}

/// add_range adds a new range to the ordered list of non-overlapping ranges.
/// It ensures that the list stays ordered and any existing ranges are subtracted
/// from the new range, potentially splitting it into multiple pieces.
fn add_range(ranges: &mut Vec<RangeWithId>, mut new_range: RangeWithId) {
    let mut inserted = false;

    let mut i = 0;
    while i < ranges.len() {
        let existing_range = *unsafe { ranges.get_unchecked(i) };

        if new_range.end_idx <= existing_range.start_idx {
            // The new range is entirely before the existing range.
            if !inserted {
                ranges.insert(i, new_range);
                i += 1;
                inserted = true;
            }
        } else if new_range.start_idx >= existing_range.end_idx {
            // The new range is entirely after the existing range.
        } else {
            // There is an overlap; we may need to split the new range.
            if !inserted && new_range.start_idx < existing_range.start_idx {
                // Add the non-overlapping piece before the existing range.
                ranges.insert(
                    i,
                    RangeWithId {
                        start_idx: new_range.start_idx,
                        end_idx: existing_range.start_idx,
                        id: new_range.id,
                    },
                );
                i += 1;
            }
            if new_range.end_idx > existing_range.end_idx {
                // Update the new range to start from the end of the existing range.
                new_range.start_idx = existing_range.end_idx;
            } else {
                // The new range is fully covered by the existing range; nothing left to add.
                inserted = true;
                new_range.start_idx = new_range.end_idx;
            }
        }
        i += 1;
    }

    // If the new range was not inserted because it is after all existing ranges,
    // or if it still has a remaining piece after processing overlaps, add it now.
    if !inserted {
        ranges.push(new_range);
    }
}

fn match_line(
    line: &str,
    regexps: &Vec<Regex>,
    vary_group_colors: bool,
    full_match_highlight: bool,
) -> Vec<RangeWithId> {
    let mut ranges = Vec::default();
    let mut color_idx = 0;
    for re in regexps {
        let num_groups = re.captures_len() - 1; // subtract implicit group
        let first_group_to_colorize = if full_match_highlight {
            0
        } else {
            min(1, num_groups)
        };
        let groups_to_colorize = if full_match_highlight {
            1
        } else {
            num_groups + 1 - first_group_to_colorize
        };
        for match_ in re.captures_iter(line) {
            // if there is no capturing group, the full match will be colorized (group 0)
            // if there are capturing groups, all groups but group 0 (the full match) will be colorized, unless
            // full_match_highlight == true
            for i in 0..groups_to_colorize {
                let mut cur_color_idx = color_idx;
                if vary_group_colors {
                    cur_color_idx += groups_to_colorize - 1 - i;
                }
                let g_idx = i + first_group_to_colorize;
                if let Some(g) = match_.get(g_idx) {
                    add_range(
                        &mut ranges,
                        RangeWithId {
                            start_idx: g.start(),
                            end_idx: g.end(),
                            id: cur_color_idx,
                        },
                    );
                }
            }
        }
        if vary_group_colors {
            color_idx += groups_to_colorize;
        } else {
            color_idx += 1;
        }
    }
    ranges
}

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
    let vary_group_colors = {
        if args.vary_group_colors_on {
            true
        } else if args.vary_group_colors_off {
            false
        } else {
            args.patterns.len() == 1
        }
    };

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
        let line = line?;
        let _ranges = match_line(
            &line,
            &regexps,
            vary_group_colors,
            args.full_match_highlight,
        );
        let rep = format!("{}$0{}", colors[0].0, colors[0].1);
        let out = regexps[0].replace_all(&line, rep);
        println!("{out}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn r(start_idx: usize, end_idx: usize, id: usize) -> RangeWithId {
        RangeWithId {
            start_idx,
            end_idx,
            id,
        }
    }

    #[rstest]
    #[case::before_1(
        vec![r(5, 8, 1)],
        r(3, 5, 2),
        vec![r(3, 5, 2), r(5, 8, 1)],
    )]
    #[case::before_2(
        vec![r(5, 8, 1)],
        r(3, 4, 2),
        vec![r(3, 4, 2), r(5, 8, 1)],
    )]
    #[case::after_1(
        vec![r(1, 3, 0)],
        r(3, 5, 2),
        vec![r(1, 3, 0), r(3, 5, 2)],
    )]
    #[case::after_2(
        vec![r(1, 3, 0)],
        r(4, 5, 2),
        vec![r(1, 3, 0), r(4, 5, 2)],
    )]
    #[case::in_between_1(
        vec![r(1, 3, 0), r(5, 8, 1)],
        r(3, 5, 2),
        vec![r(1, 3, 0), r(3, 5, 2), r(5, 8, 1)],
    )]
    #[case::in_between_2(
        vec![r(1, 3, 0), r(5, 8, 1)],
        r(3, 4, 2),
        vec![r(1, 3, 0), r(3, 4, 2), r(5, 8, 1)],
    )]
    #[case::in_between_3(
        vec![r(1, 3, 0), r(5, 8, 1)],
        r(4, 5, 2),
        vec![r(1, 3, 0), r(4, 5, 2), r(5, 8, 1)],
    )]
    #[case::partial_overlap(
        vec![r(1, 3, 0), r(5, 8, 1)],
        r(2, 6, 2),
        vec![r(1, 3, 0), r(3, 5, 2), r(5, 8, 1)],
    )]
    #[case::full_overlap(
        vec![r(1, 3, 0), r(5, 8, 1)],
        r(6, 7, 2),
        vec![r(1, 3, 0), r(5, 8, 1)],
    )]
    #[case::overlap_and_extend(
        vec![r(1, 5, 0), r(10, 15, 1)],
        r(3, 12, 2),
        vec![r(1, 5, 0), r(5, 10, 2), r(10, 15, 1)],
    )]
    fn test_add_range(
        #[case] existing: Vec<RangeWithId>,
        #[case] new_range: RangeWithId,
        #[case] expected: Vec<RangeWithId>,
    ) {
        let mut actual = existing.clone();
        add_range(&mut actual, new_range);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_match_line() {
        let regexps = vec![
            RegexBuilder::new("t")
                .case_insensitive(false)
                .build()
                .unwrap(),
        ];
        let ranges = match_line("test", &regexps, false, false);
        assert_eq!(
            ranges,
            vec![
                RangeWithId {
                    start_idx: 0,
                    end_idx: 1,
                    id: 0
                },
                RangeWithId {
                    start_idx: 3,
                    end_idx: 4,
                    id: 0
                },
            ]
        );
    }
}
