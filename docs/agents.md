# AI agents

An AI agent can use open-diagram-rs in two ways: by running the `open-diagram-rs` command, or by connecting to the `open-diagram-rs-mcp` server. Both read the same JSON and Mermaid, and both name each layout problem, so an agent can fix a diagram and check it again until the problems are gone.

## The MCP server

```bash
cargo install open-diagram-rs-mcp --locked
```

The AI app starts `open-diagram-rs-mcp` itself and talks to it over stdio. The server has five tools:

| Tool | What it does |
| --- | --- |
| `format_guide` | Lists the JSON keys, the Mermaid subset, and how to fix each layout problem |
| `template` | Lists the starter diagrams, or returns one by `name` as JSON |
| `check` | Builds a diagram in memory and lists its layout problems |
| `describe` | Says what a diagram contains, in plain words |
| `draw` | Writes the SVG and PNG, and returns the problems, the file paths, a description and the picture |

`check`, `describe` and `draw` take the diagram, JSON or Mermaid, in their `source` argument. The server writes files only inside its output folder: `--out FOLDER` if given, otherwise `$OPEN_DIAGRAM_RS_OUT`, otherwise `./open-diagram-rs`.

The built-in icons are always available. To add your own, pass `--icons FOLDER` (repeat it for more folders) or set `$OPEN_DIAGRAM_RS_ICONS` to a list of folders separated like `PATH`.

## Connect an AI app

Each app below starts the server with output going to `./open-diagram-rs`. Install the server first.

**Claude Code**

```bash
claude mcp add open-diagram-rs -- open-diagram-rs-mcp --out ./open-diagram-rs
```

**Codex**

```bash
codex mcp add open-diagram-rs -- open-diagram-rs-mcp --out ./open-diagram-rs
```

Or add this to `~/.codex/config.toml`, or to `.codex/config.toml` in a project:

```toml
[mcp_servers.open-diagram-rs]
command = "open-diagram-rs-mcp"
args = ["--out", "./open-diagram-rs"]
```

**Grok**

```bash
grok mcp add open-diagram-rs -- open-diagram-rs-mcp --out ./open-diagram-rs
```

Or add the same `[mcp_servers.open-diagram-rs]` table as for Codex to `~/.grok/config.toml` (`%USERPROFILE%\.grok\config.toml` on Windows), or to `.grok/config.toml` in a project.

**VS Code**

Add this to `.vscode/mcp.json`:

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

**Project files for Claude, Cursor and Grok**

Claude and Grok also read `.mcp.json` at the root of a project, and Cursor reads `.cursor/mcp.json`:

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

## The agent skill

[skills/open-diagram-rs/SKILL.md](../skills/open-diagram-rs/SKILL.md) teaches an agent the format and how to fix each kind of layout problem. Install it in one of these ways:

- Copy the folder into the agent's skills folder. For Claude Code: `cp -r skills/open-diagram-rs ~/.claude/skills/`
- Run `npx skills add Orrae-Labs/open-diagram-rs`.
- Let the MCP server provide it. The server offers the skills in `skills/` when it is started from a checkout of this repository, or from the folder that `$OPEN_DIAGRAM_RS_SKILLS` names. It uses the Skills over MCP extension (`io.modelcontextprotocol/skills`): `skills/list`, `skills/get`, and the skill files through `resources/read`.

This repository is also a Claude Code plugin. Add it as a marketplace, then install the plugin:

```bash
claude plugin marketplace add Orrae-Labs/open-diagram-rs
```

```bash
claude plugin install open-diagram-rs@open-diagram-rs
```

The plugin doesn't include the server binary. Install that with `cargo install` as above.

## The command

An agent that can run shell commands can use the command line instead of MCP:

```bash
cargo install open-diagram-rs-cli --locked
open-diagram-rs new
open-diagram-rs check plan.json
open-diagram-rs describe plan.json
open-diagram-rs draw plan.json
```

[cli.md](cli.md) describes every command and option.

## Cloud service icons

The cloud templates embed the icons they use, under `crates/open-diagram/templates/assets/`, and JSON refers to them by name, such as `"icon": "aws/s3"`. Mermaid flowcharts can't use icons.

[cloud-icons/](../cloud-icons) has one list each for AWS, Azure and Google Cloud, giving each service's key and the URL of a pinned SVG. The [cloud-icons skill](../skills/cloud-icons/SKILL.md) shows how to add an icon that isn't built in. A finished SVG contains the icon's bytes, so it doesn't depend on the icon file afterwards.

The icons belong to Amazon, Microsoft and Google and keep their own terms. The MIT license on this code doesn't cover them.

## Changing open-diagram-rs itself

An agent that works on this repository's code should start with [AGENTS.md](../AGENTS.md).
