# AI agents

Let an agent draw diagrams and fix their layout problems with the `open-diagram-rs` command.

## Command

```bash
cargo install open-diagram-rs-cli --locked
open-diagram-rs new
open-diagram-rs check plan.json
open-diagram-rs describe plan.json
open-diagram-rs draw plan.json
```

The command reads JSON and Mermaid. It writes SVG and PNG next to the file, or to `-o NAME` without an extension. PNG needs Chrome, Chromium or Edge (`CHROME` when the usual locations miss it).

Icons come from the built-in folder and from folders named with `--icons`. The command refuses absolute icon paths and `..`.

The MCP server is `open-diagram-rs-mcp`. An MCP app starts it over stdio. It draws JSON and Mermaid, writes files only inside its output folder, and returns the PNG from `draw`.

```bash
cargo install open-diagram-rs-mcp --locked
open-diagram-rs-mcp --out ./open-diagram-rs
```

`--icons FOLDER` may be repeated. `$OPEN_DIAGRAM_RS_ICONS` is the default, split on the system path separator. The built-in icons are always included. `$OPEN_DIAGRAM_RS_SKILLS` points at a skill folder. Without it, the server serves `skills/` when that folder is visible from the working directory. That catalog is Skills over MCP (`io.modelcontextprotocol/skills`, directory read included): `skills/list`, `skills/get`, and the skill files through `resources/read`.

The agent skill is [skills/open-diagram-rs/SKILL.md](../skills/open-diagram-rs/SKILL.md). Copy it into the agent's skills folder, install it with `npx skills add Orrae-Labs/open-diagram-rs`, or let the server serve it.

This checkout is also a Claude Code plugin (`.claude-plugin/`). Add the marketplace with `claude plugin marketplace add Orrae-Labs/open-diagram-rs`, then install `open-diagram-rs`. The server binary still comes from `cargo install`.

```bash
cp -r skills/open-diagram-rs ~/.claude/skills/
```

Agents changing the library start with [AGENTS.md](../AGENTS.md).

## MCP hosts

Install the server first: `cargo install open-diagram-rs-mcp --locked`. The process is `open-diagram-rs-mcp`. `--out` is the folder that receives SVG and PNG files.

Grok reads `~/.grok/config.toml` (`%USERPROFILE%\.grok\config.toml` on Windows) and a project `.grok/config.toml`:

```toml
[mcp_servers.open-diagram-rs]
command = "open-diagram-rs-mcp"
args = ["--out", "./open-diagram-rs"]
```

From the command line: `grok mcp add open-diagram-rs -- open-diagram-rs-mcp --out ./open-diagram-rs`.

Claude Code: `claude mcp add open-diagram-rs -- open-diagram-rs-mcp --out ./open-diagram-rs`.

Codex reads `~/.codex/config.toml` and a project `.codex/config.toml`:

```toml
[mcp_servers.open-diagram-rs]
command = "open-diagram-rs-mcp"
args = ["--out", "./open-diagram-rs"]
```

From the command line: `codex mcp add open-diagram-rs -- open-diagram-rs-mcp --out ./open-diagram-rs`.

VS Code, `.vscode/mcp.json` (`servers`, with a transport type):

```json
{
  "servers": {
    "open-diagram-rs": {
      "type": "stdio",
      "command": "open-diagram-rs-mcp",
      "args": ["--out", "./open-diagram-rs"]
    }
  }
}
```

Or: `code --add-mcp "{\"name\":\"open-diagram-rs\",\"type\":\"stdio\",\"command\":\"open-diagram-rs-mcp\",\"args\":[\"--out\",\"./open-diagram-rs\"]}"`.

Claude, Cursor, and Grok also read a project `.mcp.json` (Cursor uses `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "open-diagram-rs": {
      "command": "open-diagram-rs-mcp",
      "args": ["--out", "./open-diagram-rs"]
    }
  }
}
```

## Cloud service icons

[`cloud-icons/`](../cloud-icons) has one file each for AWS, Azure and Google Cloud. Each file lists service keys and the URL of a pinned SVG. The templates already embed the icons they use, under `crates/open-diagram/templates/assets/`. JSON refers to those by name (`"icon": "aws/s3"`). Mermaid does not take icons.

The [cloud-icons skill](../skills/cloud-icons/SKILL.md) says how to add an icon that is not built in. The finished SVG contains the icon bytes.

The icons belong to Amazon, Microsoft and Google, and keep their own terms. The MIT license on this code does not cover them.
