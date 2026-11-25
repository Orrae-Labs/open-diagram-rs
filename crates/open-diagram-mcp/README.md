# open-diagram-rs-mcp

An MCP server that lets AI apps draw architecture diagrams and check their layout. The app starts the server and talks to it over stdio.

```bash
cargo install open-diagram-rs-mcp --locked
```

The tools are `format_guide`, `template`, `check`, `describe` and `draw`. They read JSON or Mermaid. `draw` returns the PNG as an image.

The server writes files only inside its output folder: `--out FOLDER`, otherwise `$OPEN_DIAGRAM_RS_OUT`, otherwise `./open-diagram-rs`. The built-in icons are always available, and `--icons` or `$OPEN_DIAGRAM_RS_ICONS` add more icon folders.

When the server can see this repository's `skills/` folder, or `$OPEN_DIAGRAM_RS_SKILLS` names a skill folder, it also offers those skills through the Skills over MCP extension (`io.modelcontextprotocol/skills`).

Setup for Claude Code, Codex, Cursor, Grok and VS Code: <https://github.com/Orrae-Labs/open-diagram-rs/blob/main/docs/agents.md>

The server is built on the `open-diagram-rs` library crate. Repository: <https://github.com/Orrae-Labs/open-diagram-rs>

MCP Registry name: `mcp-name: io.github.Orrae-Labs/open-diagram-rs`
