# Appendix G: Rules Text Preview Pipeline

## Overview

The rules text preview feature renders card ability text with variable
substitution and rich text styling. This appendix describes the current
tabula_cli implementation and how to adapt it for TV with Univer rendering.

## Current tabula_cli Architecture

The FluentRulesTextListener in tabula_cli implements rules text preview as
part of the server listener pipeline. It processes the Cards table, reading
from a Rules Text column and an optional Variables column, then writes
formatted output to an adjacent preview column.

Source file: rules_engine/src/tabula_cli/src/server/listeners/fluent_rules_text.rs

The listener loads Fluent templates from rules_engine/tabula/strings.ftl,
which contains approximately 450 lines of message definitions with terms
and variables for card ability formatting.

## Input Format

### Rules Text Column
Contains Fluent expressions with message references and variable placeholders.
Examples from cards:
- `{-keyword(k: "Foresee")} {$n}.`
- `{-trigger(trigger: "Materialized")} Gain {$e} energy.`
- `When you play {-n-cards(n: $cards)}, draw {-n-cards(n: $draw)}.`

### Variables Column
Contains key-value pairs, one per line, defining substitution values:
```
n: 3
e: 2
cards: 1
draw: 1
```

Numeric values are parsed as Fluent numbers. String values can be quoted.
Missing variables result in Fluent resolver errors displayed in the output.

## Fluent Template Processing

### Resource Loading
The strings.ftl file is embedded into the binary via include_str! macro.
It contains reusable terms (prefixed with -) and message definitions that
provide consistent formatting across all card text.

### Term Examples from strings.ftl
```
-trigger = ▸ <b>{$trigger}:</b>
-keyword = <color=#AA00FF>{$k}</color>
-figment = <color=#F57F17><b><u>{$f} Figment</u></color></b>
-type = <color=#2E7D32><b>{$value}</b></color>
-n-cards = { $n ->
    [1] a card
   *[other] {$n} cards
}
```

### Variable Expansion
Before Fluent processing, a preprocessing step expands shorthand variable
syntax. References like `$variable_name` are converted to Fluent syntax
`{ $variable_name }`. This tracking respects brace nesting depth to avoid
expanding variables inside existing Fluent expressions.

### Temporary Message Wrapping
The input expression is wrapped into a complete Fluent message definition:
```
temp-message = <expression>
```

Multi-line expressions have continuation lines indented to maintain valid
Fluent syntax. This temporary message is parsed and formatted against the
main resource bundle containing all term definitions.

### Format Pipeline
1. Create isolated FluentBundle for this formatting operation
2. Add the main resource (strings.ftl) to the bundle
3. Parse the temporary message definition
4. Add the temporary message to the bundle
5. Look up and format temp-message with the provided FluentArgs
6. Return formatted string with all references resolved

## HTML-like Style Tags

### Supported Tags
The formatted output contains HTML-like tags for styling:
- `<b>...</b>` - Bold text
- `<i>...</i>` - Italic text
- `<u>...</u>` - Underline text
- `<color=#RRGGBB>...</color>` - Colored text with 6-digit hex RGB

Tag matching is case-insensitive. Color values accept both #RRGGBB and
RRGGBB formats. Invalid tags are passed through as literal text.

### Nested Tags
Tags can be nested to combine styles:
```
<color=#F57F17><b><u>Fire Figment</u></color></b>
```

The parser tracks depth for each style type using stacks. Style state
changes when entering or exiting a tag. The implementation handles
improper nesting gracefully by maintaining independent counters.

## Style Run Extraction

### Parsing Algorithm
The parser iterates through the formatted text character by character:
1. Accumulate plain text, stripping tag markers
2. Track output character position (not byte position)
3. When style state changes, record a StyledRun with position and length
4. Maintain style stacks for bold, italic, underline
5. Maintain color stack for nested color tags

### StyledRun Structure
Each run captures a contiguous span with consistent styling:
- start: Character position in output text (0-based)
- length: Number of characters in run
- bold: Boolean flag
- italic: Boolean flag
- underline: Boolean flag
- color: Optional RGB hex string

### Span Conversion
Runs are converted to formatting spans for application:
- Character positions convert to 1-based indices (Excel compatibility)
- Separate span lists for each attribute type
- Both active spans (bold_spans) and inactive spans (unbold_spans)
- Colors grouped by RGB value with span lists

## TV Implementation Strategy

### Derived Column Integration
Rules text preview becomes a derived column function in TV. The function
receives rules_text and variables fields from the row data. It processes
through Fluent and returns rich text suitable for Univer rendering.

### Fluent Resource Management
Load strings.ftl once at application startup. Store the parsed FluentResource
in the derived function registry or application state. Each preview
computation creates an isolated FluentBundle from the shared resource.

### Input Processing
1. Parse variables field into FluentArgs dictionary
2. Expand shorthand variable syntax in rules text
3. Wrap expression in temporary message
4. Format using FluentBundle with main resource
5. Parse HTML tags from output
6. Build Univer rich text structure

### Univer Rich Text Format
Univer cells support rich text via the paragraph/run model. Each cell's
content is an array of paragraphs, where each paragraph contains text runs
with individual styling.

ICellData rich text structure:
```
{
  p: [{
    ts: [
      { t: "Bold text", s: { bl: 1 } },
      { t: " normal ", s: {} },
      { t: "colored", s: { cl: { rgb: "FF0000" } } }
    ]
  }]
}
```

### Style Run to Univer Conversion
Convert parsed style runs to Univer text runs:
- Create one text run per style change
- Set bl: 1 for bold runs
- Set it: 1 for italic runs
- Set ul: { s: 1 } for underline runs
- Set cl: { rgb: "RRGGBB" } for colored runs

Multiple styles combine in the same run:
```
{ t: "Bold and Red", s: { bl: 1, cl: { rgb: "FF0000" } } }
```

### Simplified Span Approach
Instead of tracking span start/length positions, generate sequential runs:
1. Parse input and extract styled character ranges
2. Create one Univer text run for each range with consistent styling
3. Concatenate runs into a single paragraph
4. Return the paragraph structure for cell insertion

### Error Display
Fluent parsing or formatting errors appear as the cell content with
distinctive styling (red text) to indicate the problem. The error message
describes the issue: missing variable, syntax error, or unknown reference.

## Implementation Modules

### Fluent Integration (derived/functions/rules_preview_function.rs)
- Load and cache FluentResource from strings.ftl
- Parse variables from string into FluentArgs
- Format expressions through FluentBundle
- Handle errors gracefully with descriptive messages

### Tag Parser (derived/functions/style_tag_parser.rs)
- Parse HTML-like tags from formatted text
- Track nested style state
- Generate sequential styled runs
- Validate color hex values

### Univer Converter (derived/functions/rich_text_builder.rs)
- Convert styled runs to Univer paragraph structure
- Build ICellData-compatible rich text objects
- Serialize to JSON for frontend transport

## Migration Considerations

### Reusing Existing Code
The ability parser in rules_engine/src/parser provides additional
validation but is separate from the Fluent-based preview rendering.
Consider whether to integrate parser validation for syntax highlighting
of invalid ability text.

### Localized Strings Module
The tabula_data crate contains LocalizedStrings which wraps Fluent with
additional context handling. This could be adapted for TV or the simpler
direct FluentBundle approach can be used.

### Performance
Fluent parsing is relatively fast. Cache the FluentResource but create
fresh FluentBundle instances for each format operation to ensure isolation.
The preview function runs asynchronously so blocking is acceptable.

### Testing
Test the preview function with fixtures covering:
- Simple variable substitution
- Nested term references
- All style tag combinations
- Malformed tags (should pass through as text)
- Missing variables (error display)
- Empty input (empty output)
- Unicode content in variables and output

## Example End-to-End Flow

Input rules_text: `{-keyword(k: "Foresee")} {$n}. Draw a card.`
Input variables: `n: 3`

After Fluent expansion: `<color=#AA00FF>Foresee</color> 3. Draw a card.`

Parsed runs:
1. "Foresee" with color #AA00FF
2. " 3. Draw a card." with no styling

Univer rich text:
```json
{
  "p": [{
    "ts": [
      { "t": "Foresee", "s": { "cl": { "rgb": "AA00FF" } } },
      { "t": " 3. Draw a card.", "s": {} }
    ]
  }]
}
```

This structure is set as the cell's content, and Univer renders the styled
text inline with the appropriate colors and formatting.
