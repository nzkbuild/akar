# Design DNA

<!-- Canonical visual and UX rules for this project.
     Update when the design system changes; note the date of each change. -->

## Typography

<!-- Font families, weights, size scale. -->
- CLI output: monospace system font (terminal default)
- Heading level: bold UPPERCASE label followed by colon
- Body: sentence case, no trailing punctuation on labels

## Colors

<!-- Terminal color usage (ANSI codes or named). -->
- Success / healthy: green (ANSI 32)
- Warning / degraded: yellow (ANSI 33)
- Error / failure: red (ANSI 31)
- Neutral / info: default (no color)

## Spacing

<!-- Indentation and whitespace conventions. -->
- Top-level label: no indent
- Sub-item: 2-space indent
- Blank line between logical sections in multi-section output

## Components

<!-- Reusable output patterns. -->
- Status line: `label: VALUE`
- Sub-item: `  key: value`
- Issue bullet: `    - description`

## Anti-patterns

<!-- Things to never do in output or UX. -->
- Do not use emoji in CLI output
- Do not print color codes when stdout is not a TTY
- Do not truncate error messages — show the full path and reason

## Tone

<!-- Voice for user-facing text. -->
- Direct and factual
- No filler words ("successfully", "please", "just")
- Imperative for actionable output ("Run 'akar doctor' to investigate")
