# AI agents

Let an agent draw diagrams and fix their layout problems with the `open-diagram` command.

## Command

```bash
cargo install open-diagram-cli --locked
open-diagram new
open-diagram check plan.json
open-diagram describe plan.json
open-diagram draw plan.json
```

The command reads JSON and Mermaid. It writes SVG and PNG next to the file, or to `-o NAME` without an extension. PNG needs Chrome, Chromium or Edge (`CHROME` when the usual locations miss it).

Icons come from the built-in folder and from folders named with `--icons`. The command refuses absolute icon paths and `..`.

The MCP server is `open-diagram-mcp`. An MCP app starts it over stdio. It draws JSON and Mermaid, writes files only inside its output folder, and returns the PNG from `draw`.

```bash
cargo install open-diagram-mcp --locked
open-diagram-mcp --out ./open-diagram
```

`--icons FOLDER` may be repeated. `$OPEN_DIAGRAM_ICONS` is the default, split on the system path separator. The built-in icons are always included. `$OPEN_DIAGRAM_SKILLS` points at a skill folder. Without it, the server serves `skills/` when that folder is visible from the working directory.

The agent skill is [skills/open-diagram/SKILL.md](../skills/open-diagram/SKILL.md). Copy it into the agent's skills folder, or let the server serve it:

```bash
cp -r skills/open-diagram ~/.claude/skills/
```

Agents changing the library start with [AGENTS.md](../AGENTS.md).

## Cloud service icons

[`cloud-icons/`](../cloud-icons) has one file each for AWS, Azure and Google Cloud. Each file lists service keys and the URL of a pinned SVG. The templates already embed the icons they use, under `crates/open-diagram/templates/assets/`. JSON refers to those by name (`"icon": "aws/s3"`). Mermaid does not take icons.

The [cloud-icons skill](../skills/cloud-icons/SKILL.md) says how to add an icon that is not built in. The finished SVG contains the icon bytes.

The icons belong to Amazon, Microsoft and Google, and keep their own terms. The MIT license on this code does not cover them.
