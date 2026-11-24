# Command line

```bash
cargo install open-diagram-rs-cli --locked
```

This installs the `open-diagram-rs` command. Inside a checkout of this repository, `cargo run -p open-diagram-rs-cli --` runs the same command without installing it.

Every command reads a JSON description (`.json`) or a Mermaid flowchart (`.mmd` or `.mermaid`).

| Command | What it does |
| --- | --- |
| `open-diagram-rs draw FILE` | Writes `FILE.svg` and `FILE.png` next to the input, prints any layout problems, and exits with status 1 if there are any |
| `open-diagram-rs check FILE...` | Builds each diagram in memory and prints its layout problems. Writes nothing. Exits with status 1 if any file has a problem |
| `open-diagram-rs describe FILE` | Prints what the picture says in plain words: its blocks, groups and lines |
| `open-diagram-rs new [TEMPLATE]` | Lists the templates, or writes `TEMPLATE.json` into the current folder |
| `open-diagram-rs png SVG` | Converts an SVG to PNG with Chrome, Chromium or Edge |

Useful options:

- `draw -o PATH` writes `PATH.svg` and `PATH.png` instead. Leave out the extension.
- `draw --no-png` writes the SVG only, so no browser is needed.
- `draw --scale N` and `png --scale N` set the PNG's pixel scale. The default is 2.
- `--title TEXT` on `draw` and `describe` replaces the title. A Mermaid file otherwise uses the `title:` in its front matter, or "Flowchart".
- `--statuses FILE` on `draw`, `check` and `describe` sets states by name before building. See below.
- `--icons FOLDER` on `draw`, `check` and `describe` adds a folder of icons that JSON icon names can use.
- `new --force` replaces an existing file. `new -o FILE` writes somewhere else.

PNG output needs Chrome, Chromium or Edge. If the command can't find yours, set `CHROME` to its path.

## Keep diagrams up to date with GitHub Actions

This repository is also a GitHub Action. Add the workflow below and every push and pull request checks your diagrams, failing if one has a layout problem:

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

`files` takes globs separated by spaces or newlines, including `**`. A pattern that matches nothing fails the job, so a moved folder doesn't pass silently.

## Track state in one file

The parts of a system move from planned to being built to live. Instead of editing each diagram as that happens, keep the states in one JSON file and pass it with `--statuses`:

```json
{"Fraud checks": "live", "Parcel lockers": "building"}
```

Each name matches a panel title, a panel id or a tile label in any diagram. The states are `live`, `building`, `trial`, `planned` and `retiring`. Setting a part to `live` returns it to the default look. When work ships, change one word in this file and every diagram that mentions the part follows.

From Rust, call `Diagram::set_statuses`.

## MCP server

The MCP server, `open-diagram-rs-mcp`, offers the same drawing and checking to AI apps. Setup is in [agents.md](agents.md).
