# open-diagram-rs

A command that draws architecture diagrams from JSON or Mermaid and checks their layout.

```bash
cargo install open-diagram-rs-cli --locked
```

```bash
open-diagram-rs new azure_web_app
open-diagram-rs check azure_web_app.json
open-diagram-rs draw azure_web_app.json
```

| Command | What it does |
| --- | --- |
| `draw` | Writes an SVG and a PNG, and exits with status 1 if the layout has problems |
| `check` | Lists the layout problems without writing files |
| `describe` | Prints what the picture says in plain words |
| `new` | Lists the templates, or writes one to start from |
| `png` | Converts an SVG to PNG |

PNG output needs Chrome, Chromium or Edge. If the command can't find yours, set `CHROME` to its path.

The command is built on the `open-diagram-rs` library crate. It embeds that crate's fonts and cloud icons, which keep the licenses listed there.

Documentation: <https://github.com/Orrae-Labs/open-diagram-rs/blob/main/docs/cli.md>
