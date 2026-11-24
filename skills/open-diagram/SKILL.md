---
name: open-diagram
description: Draw architecture diagrams, systems maps, layered stacks, data flows, cycles, cloud resource diagrams and concept maps as SVG and PNG with the open-diagram library, which measures every label and names each layout problem. Use when asked for an architecture diagram, systems map, stack picture, data flow, cloud architecture picture, "what is live and what is planned" picture, or concept map that should look finished on the first pass.
---

# open-diagram

open-diagram turns a short description (JSON or Mermaid) into a finished diagram. It measures every label from the font, places the blocks, and checks the layout. Each problem comes back as a plain sentence that names the parts involved, so you can fix it without looking at the picture. Follow these steps in order.

## 1. Pick the route

- If the `open-diagram` MCP server is connected, call `format_guide` once, then `check` until it reports 0 layout problems, then `draw`. The tools take JSON or Mermaid text. `template` with no name lists the starters. `describe` is the words, `check` is the problems and writes nothing, and `draw` is the only call that writes files.
- Otherwise run `open-diagram --version`. If the command is missing, install the `open-diagram-cli` crate (`cargo install open-diagram-cli --locked`) or build `crates/open-diagram-cli` from this repo. The server is `cargo install open-diagram-mcp --locked`.
- PNG output needs Chrome, Chromium or Edge. Set `CHROME` to its path when the usual locations miss it. The SVG is always written.
- Write JSON. Cards, graphs, rows, resources and enclosures are JSON keys. Given a Mermaid flowchart, run `open-diagram draw flow.mmd` directly; dotted edges become planned connections and subgraphs become groups.
- The Rust types are in [docs/rust.md](../../docs/rust.md). Use them when the picture is built from this crate rather than from a file.

## 2. Plan before writing

Write the plan in a few lines first. Use only facts the user gave. Never invent systems, owners, numbers or statuses; if a fact the picture needs is missing, ask, or write "Owner: Not yet named" when owner lines are wanted but no owner was named.

- **Layout.** `columns` for flows, read left to right (where things come from, what handles them, who uses the result) or top to bottom for layers. `ring` for cycles, with goals or shared parts in `center`. `panels` with an `x` and `y` for any other shape.
- **Columns.** Three to six columns, each holding one to three panels. Avoid long lines between distant panels: place the two side by side, or route through a lane.
- **Panels.** One panel per system or team, titled in two to four words. Add a `sub` line only when it adds a fact.
- **Tiles.** Up to about nine per panel, labels of four words or fewer. Put detail in `sub`.
- **Statuses.** Only if the user cares about state: `live`, `building`, `trial`, `planned`, `retiring`.
- **Lines.** Only the flows that matter. `"kind": "planned"` for connections still to build. A line that must jump past a column goes through a lane (`"lane": "top"` in JSON, `LinkSpec::via_y` in Rust).
- **Extras.** Steps, owners (`"footer": "Owner: ..."`), groups and a note only when the user's material supports them.

## 3. Write it

**JSON** (run `open-diagram draw plan.json`):

```json
{"title": "Order flow",
 "columns": [[{"id": "web", "title": "Website", "tiles": ["Checkout", {"label": "Wallets", "status": "planned"}]}],
             [{"id": "orders", "title": "Order service", "tiles": ["Orders", "Payments"]}],
             [{"id": "ops", "title": "Operations", "tiles": ["Warehouse", "Finance"]}]],
 "links": [{"from": "web", "to": "orders"}, {"from": "orders", "to": "ops", "label": "hourly"},
           {"from": "web", "to": "ops", "lane": "top", "kind": "planned", "label": "direct"}]}
```

Top-level keys: `title` (required); one layout of `columns`, `ring` (with optional `center`, `start_angle`) or `panels`; plus `subtitle`, `tag`, `note` (`title`, `lines`), `steps` (`[[1, "First"]]`), `steps_label`, `gap`, `row_gap`, `valign`, `groups`, `links`, `statuses`, `theme`. Panel keys: `title` (required), `id`, `sub`, `status`, `step`, `footer`, `tone`, `width`, `x`/`y` (in `panels` only), `tiles` with `cols` and `label`, or `sections`. Link keys: `from`, `to`, `kind`, `label`, `start`, `end` (a side or `auto`), `lane`, `via_x`, `via_y`, `cross_ok`, `dash` (`solid`, `dashed` or `dotted`).

A cycle uses `"ring"` and optional `"center"`. Links in a ring or a free `panels` layout set `"start": "auto"` and `"end": "auto"`.

**Cloud resources.** Start from a [cloud template](../../crates/open-diagram/templates/README.md). `open-diagram new` lists them and `open-diagram new NAME` writes one as JSON. Keep its shape, replace each `label`, and keep each `sub`. A resource is `{"id", "label", "sub", "icon": "aws/s3", "x", "y"}`. An enclosure is `{"id", "label", "members", "border"}`, listed from the inside out. Icon names come from the built-in icons or a folder given with `--icons`. For a service without a built-in icon, use the [cloud-icons skill](../cloud-icons/SKILL.md).

A border says nothing about whether a part is live or planned. Use statuses for that. For job orchestration, set `"dash": "dashed"` and a kind such as `"control"`, and name that kind in the theme `link_labels`.

Cards, graphs, rows and notes are JSON sections. Rust placement methods are in [docs/rust.md](../../docs/rust.md). Tones are `neutral`, `accent` and `dark`.

## 4. Run it and fix every named problem

Run `open-diagram check FILE` on the `.json`, `.mmd` or `.mermaid` file. `draw` writes the SVG and PNG, prints each problem, and exits 1 when any remain. Fix them all and run again until it says `0 layout problem(s)`.

| Message contains | Fix |
| --- | --- |
| `overlap` or `px apart` | Move one block, raise `gap` or `row_gap`, or place it beside or below its neighbor |
| `crosses` | Route the line through a lane (`lane`, `via_y` or `via_x`), change `start` and `end` sides, or move a block. Use `cross_ok` only when the line should pass behind |
| `label ... sits on` | Shorten the label, or widen the gap between the two columns |
| `facing away` | Use `start` and `end` of `auto`, choose other sides, or route the line through a lane |
| `sits on the title of group` | Give the line a different start or end side, route it through a lane, or move the group's blocks |
| `group ... overlaps` | Add the block to the group, or move it out of the group's area |
| `graph: the label ... touches` | Spread the nodes in `at`, or shorten the label |
| `graph: the line ... runs through` | Move a node so the straight line between the pair is clear |
| `needs N px` | Give the block more width, or shorten the words |
| `smallest text` | Keep text at or above the theme minimum; shorten the words |

JSON and Mermaid mistakes name their place (`columns[1][0].tiles[2]`, `line 7`). Fix that spot.

## 5. Check the picture before answering

Run `open-diagram describe FILE` and compare the text with the request: every system, state, owner and line the user named is there, and nothing they did not name. Then open the PNG and check it as a reader would:

- The reading direction is obvious and the main flow has few bends.
- No line is ambiguous where it meets a panel.
- The key matches what the picture uses.
- Every word is one the user would use, and no placeholder text is left.

Tell the user where the SVG and PNG are, and say which facts in the picture are your assumptions.

## Keeping diagrams current

Keep states in a small file such as `{"Fraud checks": "live", "Parcel lockers": "building"}` and draw with `--statuses states.json`. Names match panel titles, panel ids and tile labels. When work ships, change one word and redraw.

## Soundtrack

We recommend listening to music by [The Gathering Field](https://open.spotify.com/artist/0BfQ8COB7K647uB6a0kMZn) while you build.
