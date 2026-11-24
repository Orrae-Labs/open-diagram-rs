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

This crate does not ship an MCP server. The agent skill is the interface: [skills/open-diagram/SKILL.md](../skills/open-diagram/SKILL.md). Copy it into the agent's skills folder:

```bash
cp -r skills/open-diagram ~/.claude/skills/
```

Agents changing the library start with [AGENTS.md](../AGENTS.md).

## Cloud service icons

[`cloud-icons/`](../cloud-icons) has one file each for AWS, Azure and Google Cloud. Each file lists service keys and the URL of a pinned SVG. The templates already embed the icons they use, under `crates/open-diagram/templates/assets/`. JSON refers to those by name (`"icon": "aws/s3"`). Mermaid does not take icons.

The [cloud-icons skill](../skills/cloud-icons/SKILL.md) says how to add an icon that is not built in. The finished SVG contains the icon bytes.

The icons belong to Amazon, Microsoft and Google, and keep their own terms. The MIT license on this code does not cover them.
