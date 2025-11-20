# Colorexp
**Colorexp** is a command line tool that allows coloring of text matches from standard input with multiple colors,
something that is not easy to achieve with tools like `grep` and `less`.

**Colorexp**
- uses Rust's regex format, as documented [here](https://docs.rs/regex/latest/regex/#syntax).
- supports overlapping matches (the color for the last pattern that matches will be used)

# Usage
```
Usage: colorexp-rs [OPTIONS] [PATTERNS]...

Arguments:
  [PATTERNS]...  Patterns

Options:
  -H, --only-highlight  Only color by changing the background color
  -i, --ignore-case     Perform case-insensitive matching
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
