# Command line

Draw, check and describe diagrams from a terminal or a CI job.

```bash
cargo install open-diagram-rs-cli --locked
```

The binary is named `open-diagram-rs`. From this repo, `cargo run -p open-diagram-rs-cli --` runs the same commands.

```bash
open-diagram-rs draw plan.json       # writes plan.svg and plan.png; exits with 1 if the layout has problems
open-diagram-rs check docs/*.json    # checks without writing files, for scripts and CI
open-diagram-rs describe plan.json   # prints what the picture says, in plain words
open-diagram-rs new                  # lists the templates; `open-diagram-rs new azure_web_app` writes one to start from
```

Each command takes a JSON description (`.json`) or a Mermaid flowchart (`.mmd` or `.mermaid`). Add `--statuses states.json` to set states by name before drawing, and `--icons FOLDER` to use your own icons in JSON.

## MCP

`open-diagram-rs-mcp` is the same drawings over stdio, for an MCP app. Tools: `format_guide`, `template`, `check`, `describe`, `draw`. Files stay inside `--out` (default `$OPEN_DIAGRAM_RS_OUT` or `./open-diagram-rs`).

```bash
cargo install open-diagram-rs-mcp --locked
open-diagram-rs-mcp --out ./open-diagram-rs
```

Connecting Claude, Codex, Cursor, or Grok is in [agents.md](agents.md).

## Keep diagrams from going stale

Add this GitHub Action and every push redraws your diagrams and fails the check if a layout problem appears:

```yaml
# .github/workflows/diagrams.yml
name: diagrams
on: [push, pull_request]
jobs:
  diagrams:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v7
      - uses: Orrae-Labs/open-diagram-rs@main
        with:
          files: docs/diagrams/*.json docs/diagrams/*.mmd
          statuses: docs/diagrams/states.json   # optional
          mode: check                           # or draw, to write SVG and PNG next to each file
```

Keep states in a small file, such as `{"Fraud checks": "live", "Parcel lockers": "building"}`. When work ships, change one word; the pictures follow. Names match panel titles, panel ids and tile labels, and `live` returns a part to the default look. From the library, call `Diagram::set_statuses`.
