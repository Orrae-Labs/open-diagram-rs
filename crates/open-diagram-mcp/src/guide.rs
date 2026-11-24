//! Format guide copied from the Python MCP server.

#![expect(
    clippy::redundant_pub_crate,
    reason = "sibling modules use these items; pub would fail unreachable_pub because they are not the external API"
)]

pub(crate) const INSTRUCTIONS: &str = "open-diagram-rs draws architecture diagrams from JSON or Mermaid text. \
Call format_guide once, or template with no name for a starter. \
check lists layout problems and writes nothing; fix every named part and call it again until it reports 0. \
describe says what the picture contains. \
draw writes the SVG and PNG into the output folder and returns the paths and the picture. \
A layout problem is the result of check: change the named parts, and leave text at the size the theme asked for.";

pub(crate) const GUIDE: &str = r#"# open-diagram-rs format guide

Write the diagram as JSON (below) or as a Mermaid flowchart. Call `check` until it reports 0 layout problems, then
`draw`. Every problem names the parts involved; fix each one, never ignore it.

## JSON: a worked example to copy

{
  "title": "How a search request is answered",
  "subtitle": "Optional one-line scope",
  "tag": "DRAFT",
  "steps": [[1, "Cache results"], [2, "Add spelling fixes"]],
  "steps_label": "Rollout",
  "columns": [
    [{"id": "app", "title": "Web app", "tiles": ["Search box", "Results page"], "cols": 1,
      "footer": "Owner: Web team"}],
    [{"id": "api", "title": "Search API", "step": 1,
      "tiles": ["Checks access", {"label": "Caches results", "status": "building"}], "cols": 1},
     {"id": "logs", "title": "Request log", "tiles": ["Daily totals"], "tone": "neutral"}],
    [{"id": "index", "title": "Search index", "status": "planned", "step": 2,
      "tiles": ["Product text", {"label": "Spelling fixes", "status": "planned"}], "cols": 1}]
  ],
  "groups": [{"label": "Platform", "members": ["api", "logs"], "sub": "Run by the platform team"}],
  "links": [
    {"from": "app", "to": "api"},
    {"from": "api", "to": "logs", "start": "bottom", "end": "top"},
    {"from": "api", "to": "index", "kind": "planned", "label": "new"},
    {"from": "app", "to": "index", "lane": "top", "label": "Direct for admins"}
  ]
}

## Other shapes
Use exactly one layout. "columns" (above) reads left to right. "ring" puts panels around a circle, clockwise from
the top, for cycles: {"title": "...", "ring": [panels], "center": [panels in the middle], "gap": 84, "links": [...]}.
"panels" places each panel where you say: give every panel "x" and "y" (its top-left corner, in pixels).
In ring and panels layouts, links leave from the sides that face each other unless you set start and end.

## Keys
- Top level: title (required), one layout (columns, ring with optional center and start_angle, or panels),
  subtitle, tag, note {title, lines}, steps [[number, text]], steps_label, gap, row_gap, valign, groups, links,
  statuses {name: state}, theme, routing ("orthogonal" draws every line as straight runs that meet at right angles;
  the default is "curved").
- Panel: title (required), id (used by links and groups; defaults to the title), sub, status, step, footer
  ("Owner: ..."; text before the colon is bold), tone (default, neutral, accent = soft yellow, dark = charcoal),
  width, x and y (panels layout only), tiles (strings or {label, sub, status}) with cols and label, or sections (each {"tiles": [...]},
  {"text": "..."}, {"card": {"title", "rows": [[name, value]], "intro"}}, {"graph": {"nodes", "links", "at"}},
  or {"row": [sections]}).
- Link: from, to (panel ids), kind (flow = solid, planned = dotted), label, start/end (left, right, top, bottom,
  or auto: the side facing the other panel), lane (top or bottom: route around the picture), via_x / via_y, cross_ok,
  dash (solid, dashed or dotted; optional style override independent of deployment state), routing (curved or
  orthogonal, for this link only).
- Group: label, members (panel ids), sub.
- States: live (default look), building, trial, planned, retiring. Use them only when the user cares about state.

## Cloud resources and templates
For cloud architecture, call `template` with no name to list the templates, then `template` with a name to get one as
JSON; keep its shape, replace the labels, keep each resource's sub (the service name). To write one yourself, use
"resources" instead of a layout (or with "panels"): each {id, label, sub, icon, x, y} is a large icon with its label
under it, placed at x and y in pixels. "enclosures" draw boxes around resources: {id, label, members, sub, border
(solid, dashed, dotted), style (outline, or panel for a filled header), pad, fill, stroke, tone}; list them from the
inside out. An icon is a name such as "aws/s3"; the icons available are listed at the end of this guide. Links join
resource ids as they join panels.

## Mermaid
flowchart or graph, any direction; nodes with any bracket shape; chains and &; edge labels (-->|text| or
-- text -->); dotted edges (-.->) become planned; top-level subgraphs become shaded groups; A:::building or
class A,B planned sets a state; a front-matter title: line sets the title; curve: step (in front matter, or
%%{init: {"flowchart": {"curve": "step"}}}%%) draws right-angle lines.

## Fixing problems
- "overlap" or "px apart": raise gap or row_gap, or move panels to other columns.
- "crosses": route the link with a lane, change start/end, or reorder columns.
- "label ... sits on": shorten the label or raise gap.
- "sits on the title of group": give the link another start/end side or a lane.
- "needs N px": shorten the words or give the panel more width.
- "facing away": the link leaves or enters on the wrong side; set start and end to auto, or use a lane.
Long curved links between distant panels are hard to follow: put the two panels side by side, or use a lane.
When the user asks for straight, square or right-angle lines, set "routing": "orthogonal" at the top level.
Yellow: the accent tone is yellow. When any panel uses it, step numbers turn charcoal so yellow keeps one meaning;
use accent for one kind of panel only and say what it means in a note.
Keep panel titles to two to four words and tiles to four words or fewer. Never invent owners, numbers or states.
"#;

pub(crate) fn guide_text(names: &[String]) -> String {
    let listed = if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    };
    format!("{GUIDE}\nIcons available: {listed}\n")
}
