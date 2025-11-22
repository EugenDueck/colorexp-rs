# Colorexp
**Colorexp** is a command line tool that allows coloring of text matches from standard input with multiple colors,
something that is not easy to achieve with tools like `grep` and `less`.

**Colorexp**
- uses Rust's regex format, as documented [here](https://docs.rs/regex/latest/regex/#syntax).
- supports overlapping matches (the color for the last pattern that matches will be used)

# Usage
```
Usage: colorexp [OPTIONS] [PATTERNS]...

Arguments:
  [PATTERNS]...  Patterns

Options:
  -F, --fixed-strings          Interpret PATTERNS as fixed strings, not regular expressions
  -f, --full-match-highlight   Highlight the entire match, even if pattern contains capturing groups
  -i, --ignore-case            Perform case-insensitive matching
  -h, --no-highlight           Do not color by changing the background color
  -H, --only-highlight         Only color by changing the background color
  -o, --only-matching-lines    Only print lines with matches (suppress lines without matches)
  -g, --vary-group-colors-off  Turn off changing of colors for every capturing group. Defaults to on if exactly one pattern is given
  -G, --vary-group-colors-on   Turn on changing of colors for every capturing group. Defaults to on if exactly one pattern is given
```
## Examples

### Basic Usage
- use the `-h`/`-H` options to only colorize the text, or only the background

![Example](example-basic.png)

### Overlapping matches - last match wins
- all matches are colorized, and the color of the last match will be used

![Example](example-overlaps.png)

### Capturing groups
- when using capturing groups, only the matched group contents will be colorized
#### Vary colors of groups in patterns
- when exactly one pattern is given, the default is to use different colors for each capturing group
    - in case of multiple patterns, the `-G` option can be used to enforce varying of the colors for each group

![Example](example-group-varying-colors.png)

#### Use the same color for all groups of a pattern
- when multiple patterns are given, the default is to use the same colors for all capturing groups of a pattern
    - in case of a single pattern, the `-g` option can be used to enforce use of a single color

![Example](example-group-same-color.png)
