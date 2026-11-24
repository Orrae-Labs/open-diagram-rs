# open-diagram-mcp

MCP server for architecture diagrams. An MCP app starts it over stdio.

```bash
cargo install open-diagram-mcp --locked
open-diagram-mcp --out ./open-diagram
```

Tools: `format_guide`, `template`, `check`, `describe`, `draw`. The input is JSON or Mermaid. Files are written only inside the output folder. `--icons` and `$OPEN_DIAGRAM_ICONS` add icon folders. The built-in icons are always included.

When this checkout's `skills/` folder is visible, or `$OPEN_DIAGRAM_SKILLS` points at a skill folder, those skills are served with the SDK's skill catalog. `draw` returns the PNG as a tool image.

The library is the `open-diagram` crate. Repository: <https://github.com/Orrae-Labs/open-diagram-rs>

- MCP Registry name: `mcp-name: io.github.Orrae-Labs/open-diagram`

Host setup for Grok, Claude, Codex, Cursor, and VS Code: <https://github.com/Orrae-Labs/open-diagram-rs/blob/main/docs/agents.md>
