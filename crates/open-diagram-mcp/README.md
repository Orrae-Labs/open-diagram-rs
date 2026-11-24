# open-diagram-rs-mcp

MCP server for architecture diagrams. An MCP app starts it over stdio.

```bash
cargo install open-diagram-rs-mcp --locked
open-diagram-rs-mcp --out ./open-diagram-rs
```

Tools: `format_guide`, `template`, `check`, `describe`, `draw`. The input is JSON or Mermaid. Files are written only inside the output folder. `--icons` and `$OPEN_DIAGRAM_RS_ICONS` add icon folders. The built-in icons are always included.

When this checkout's `skills/` folder is visible, or `$OPEN_DIAGRAM_RS_SKILLS` points at a skill folder, those skills are served with the SDK's skill catalog. That catalog is Skills over MCP (`io.modelcontextprotocol/skills`, directory read included). `draw` returns the PNG as a tool image.

The library is the `open-diagram-rs` crate. Repository: <https://github.com/Orrae-Labs/open-diagram-rs>

- MCP Registry name: `mcp-name: io.github.Orrae-Labs/open-diagram-rs`

Host setup for Grok, Claude, Codex, Cursor, and VS Code: <https://github.com/Orrae-Labs/open-diagram-rs/blob/main/docs/agents.md>
