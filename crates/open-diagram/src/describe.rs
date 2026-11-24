#![expect(
    clippy::redundant_pub_crate,
    reason = "sibling modules use these items; pub would fail unreachable_pub because they are not the external API"
)]

use std::fmt::Write as _;

use crate::blocks::Block;
use crate::diagram::{Diagram, Node};
use crate::num::py_round;

pub(crate) fn describe(diagram: &Diagram) -> String {
    let mut out = vec![diagram.title.clone()];
    if let Some(subtitle) = &diagram.subtitle {
        out.push(subtitle.clone());
    }
    if let Some(line) = legend_line(diagram) {
        out.push(line);
    }
    if let Some(line) = steps_line(diagram) {
        out.push(line);
    }
    out.push(String::new());
    out.push("Blocks, left to right:".to_owned());
    let mut order: Vec<_> = diagram.nodes.iter().collect();
    order.sort_by(|left, right| {
        (py_round(left.placed.x), py_round(left.placed.y))
            .cmp(&(py_round(right.placed.x), py_round(right.placed.y)))
    });
    for node in order {
        out.push(format!("- {}", placed(diagram, node)));
    }
    if !diagram.groups.is_empty() {
        out.push("Groups:".to_owned());
        for group in &diagram.groups {
            let sub = group.sub.as_ref().map_or_default(|s| format!(" ({s})"));
            let names = group
                .members
                .iter()
                .map(|id| {
                    diagram
                        .nodes
                        .get(*id)
                        .map_or_else(String::new, visible_name)
                })
                .collect::<Vec<_>>()
                .join(", ");
            out.push(format!("- {}{sub}: {names}", group.label));
        }
    }
    if !diagram.links.is_empty() {
        out.push("Lines:".to_owned());
        for line in &diagram.links {
            let from = diagram
                .nodes
                .get(line.a)
                .map_or_else(String::new, visible_name);
            let to = diagram
                .nodes
                .get(line.b)
                .map_or_else(String::new, visible_name);
            let mut text = format!("- {from} -> {to}");
            if line.kind == "planned"
                && let Some(label) = diagram.theme.link_labels.get("planned")
            {
                write!(text, " ({})", label.to_lowercase()).unwrap_or(());
            }
            if let Some(label) = &line.label {
                write!(text, ", labeled \"{label}\"").unwrap_or(());
            }
            if let Some(dash) = &line.dash {
                write!(text, " ({dash} line)").unwrap_or(());
            }
            out.push(text);
        }
    }
    if let Some(note) = &diagram.note {
        out.push(format!(
            "Note: {}",
            block_text(diagram, &Block::Note(note.clone()))
        ));
    }
    out.join("\n")
}

fn legend_line(diagram: &Diagram) -> Option<String> {
    let theme = &diagram.theme;
    let mut used = std::collections::BTreeSet::new();
    for node in &diagram.nodes {
        used.extend(node.block.statuses());
    }
    let words: Vec<_> = theme
        .statuses
        .iter()
        .filter(|(k, _)| used.contains(k.as_str()) || (*k == "live" && used.contains("")))
        .map(|(_, st)| st.label.clone())
        .collect();
    let kinds: std::collections::BTreeSet<_> =
        diagram.links.iter().map(|l| l.kind.clone()).collect();
    let mut key = Vec::new();
    if !words.is_empty() {
        key.push(format!("States shown: {}.", words.join(", ")));
    }
    for (kind, label) in &theme.link_labels {
        if !kinds.contains(kind) {
            continue;
        }
        let mut styles = Vec::new();
        for line in &diagram.links {
            if &line.kind == kind {
                let style = line.dash.clone().unwrap_or_else(|| {
                    if kind == "planned" {
                        "dotted".to_owned()
                    } else {
                        "solid".to_owned()
                    }
                });
                let style = capitalize(&style);
                if !styles.contains(&style) {
                    styles.push(style);
                }
            }
        }
        key.push(format!(
            "{} lines: {}.",
            styles.join("/"),
            label.to_lowercase()
        ));
    }
    if key.is_empty() {
        None
    } else {
        Some(key.join(" "))
    }
}

fn steps_line(diagram: &Diagram) -> Option<String> {
    if diagram.steps.is_empty() {
        return None;
    }
    let mut owners = std::collections::BTreeMap::new();
    for node in &diagram.nodes {
        if let Block::Panel(panel) = &node.block
            && let Some(step) = panel.step
        {
            owners.entry(step).or_insert_with(|| visible_name(node));
        }
    }
    let parts: Vec<_> = diagram
        .steps
        .iter()
        .map(|(n, text)| {
            let owned = owners.get(n).map_or_default(|name| format!(" ({name})"));
            format!("{n}. {text}{owned}")
        })
        .collect();
    Some(format!("{}: {}.", diagram.steps_label, parts.join("; ")))
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or_default(|c| c.to_uppercase().collect::<String>() + chars.as_str())
}

fn status_word(diagram: &Diagram, key: Option<&str>) -> Option<String> {
    let key = key?;
    if key.is_empty() || key == "live" {
        return None;
    }
    diagram
        .theme
        .statuses
        .get(key)
        .map(|st| st.label.to_lowercase())
        .or_else(|| Some(key.to_owned()))
}

fn tile_text(diagram: &Diagram, tile: &crate::blocks::Tile) -> String {
    let mut extra = Vec::new();
    if let Some(sub) = &tile.sub {
        extra.push(sub.clone());
    }
    if let Some(word) = status_word(diagram, tile.status.as_deref()) {
        extra.push(word);
    }
    if extra.is_empty() {
        tile.label.clone()
    } else {
        format!("{} ({})", tile.label, extra.join("; "))
    }
}

fn block_text(diagram: &Diagram, block: &Block) -> String {
    match block {
        Block::Resource(v) => v
            .sub
            .as_ref()
            .map_or_else(|| v.label.clone(), |s| format!("{} ({s})", v.label)),
        Block::Tiles(v) => {
            let items = v
                .items
                .iter()
                .map(|t| tile_text(diagram, t))
                .collect::<Vec<_>>()
                .join("; ");
            v.label
                .as_ref()
                .map(|l| format!("{l}: {items}"))
                .unwrap_or(items)
        }
        Block::Tile(v) => tile_text(diagram, v),
        Block::Text(v) => v.text.clone(),
        Block::Card(v) => {
            let rows = v
                .rows
                .iter()
                .map(|(key, value)| format!("{key} {value}"))
                .collect::<Vec<_>>()
                .join("; ");
            format!("example '{}': {rows}", v.title)
        }
        Block::Graph(v) => {
            let links = v
                .links
                .iter()
                .map(|(a, b, label)| {
                    format!(
                        "{} {} {}",
                        v.nodes.get(a).map_or("", String::as_str),
                        label.as_deref().unwrap_or("to"),
                        v.nodes.get(b).map_or("", String::as_str),
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            format!("map: {links}")
        }
        Block::Row(v) => v
            .items
            .iter()
            .map(|b| block_text(diagram, b))
            .collect::<Vec<_>>()
            .join(". "),
        Block::Note(v) => std::iter::once(format!("{}.", v.title))
            .chain(v.lines.iter().cloned())
            .collect::<Vec<_>>()
            .join(" "),
        Block::Panel(_) => String::new(),
    }
}

fn visible_name(node: &Node) -> String {
    match &node.block {
        Block::Panel(panel) => panel.title.clone(),
        Block::Tile(tile) => tile.label.clone(),
        Block::Resource(resource) => resource.label.clone(),
        Block::Card(_)
        | Block::Graph(_)
        | Block::Note(_)
        | Block::Row(_)
        | Block::Text(_)
        | Block::Tiles(_) => node.placed.name.clone(),
    }
}

fn placed(diagram: &Diagram, node: &Node) -> String {
    match &node.block {
        Block::Panel(panel) => {
            let mut tags = Vec::new();
            if let Some(step) = panel.step {
                tags.push(format!("step {step}"));
            }
            if let Some(word) = status_word(diagram, panel.status.as_deref()) {
                tags.push(word);
            }
            let mut head = visible_name(node);
            if !tags.is_empty() {
                write!(head, " [{}]", tags.join(", ")).unwrap_or(());
            }
            if let Some(sub) = &panel.sub {
                write!(head, " ({sub})").unwrap_or(());
            }
            let body = panel
                .body
                .iter()
                .map(|b| block_text(diagram, b))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(". ");
            let mut text = if body.is_empty() {
                head
            } else {
                format!("{head}: {body}")
            };
            if let Some(footer) = &panel.footer {
                write!(text, ". {}", footer.replace('\n', " ")).unwrap_or(());
            }
            text
        }
        Block::Resource(_) => block_text(diagram, &node.block),
        Block::Tile(tile) => tile_text(diagram, tile),
        Block::Card(_)
        | Block::Graph(_)
        | Block::Note(_)
        | Block::Row(_)
        | Block::Text(_)
        | Block::Tiles(_) => {
            let inner = block_text(diagram, &node.block);
            if inner.is_empty() {
                node.placed.name.clone()
            } else {
                format!("{}: {inner}", node.placed.name)
            }
        }
    }
}
