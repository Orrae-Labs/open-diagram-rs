use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use regex::Regex;

use crate::blocks::Tile;
use crate::diagram::{Diagram, LinkSpec, Placed};
use crate::error::MermaidError;
use crate::font::text_width;
use crate::num::{count, mul, whole};
use crate::theme::Theme;

fn pattern(source: &str) -> Regex {
    match Regex::new(source) {
        Ok(re) => re,
        Err(err) => bad_pattern(&err),
    }
}

#[expect(
    clippy::panic,
    reason = "mermaid patterns are fixed literals; an invalid one is a bug in this file"
)]
fn bad_pattern(err: &regex::Error) -> ! {
    panic!("mermaid pattern is fixed: {err}");
}

fn clean(text: &str) -> String {
    let text = text.trim();
    let text = if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        &text[1..text.len() - 1]
    } else {
        text
    };
    pattern(r"\s*<br\s*/?>\s*")
        .replace_all(text, " ")
        .trim()
        .to_owned()
}

fn dash_labels(input: &str) -> String {
    let re = pattern(r"^--\s+([^-|>][^|]*?)\s+(-->|---)");
    let mut out = String::new();
    let mut rest = input;
    while !rest.is_empty() {
        if let Some(index) = rest.find("--") {
            let prev = rest[..index].chars().next_back();
            let blocked = matches!(prev, Some('-' | '.' | '=' | '<'));
            if !blocked && let Some(caps) = re.captures(&rest[index..]) {
                out.push_str(&rest[..index]);
                out.push_str(&caps[2]);
                out.push('|');
                out.push_str(&caps[1]);
                out.push('|');
                let eaten = caps.get(0).map_or(0, |item| item.end());
                rest = &rest[index + eaten..];
                continue;
            }
            out.push_str(&rest[..index + 2]);
            rest = &rest[index + 2..];
        } else {
            out.push_str(rest);
            break;
        }
    }
    out
}

fn label_subs(stmt: &str) -> String {
    let dotted = pattern(r"-\.\s+([^.|][^|]*?)\s+\.->");
    let equals = pattern(r"==\s+([^=|][^|]*?)\s+==>");
    let once = dash_labels(stmt);
    let twice = dotted.replace_all(&once, "-.->|$1|");
    equals.replace_all(&twice, "==>|$1|").into_owned()
}

struct NodeToken {
    id: String,
    text: Option<String>,
    class_name: Option<String>,
}

fn read_node(token: &str, line_no: usize) -> Result<NodeToken, MermaidError> {
    const SHAPES: &[(&str, &str)] = &[
        ("((", "))"),
        ("([", "])"),
        ("[[", "]]"),
        ("[(", ")]"),
        ("{{", "}}"),
        ("[/", "/]"),
        ("[\\", "\\]"),
        ("[/", "\\]"),
        ("[\\", "/]"),
        ("[", "]"),
        ("(", ")"),
        ("{", "}"),
        (">", "]"),
    ];
    let token = token.trim();
    let class_match = pattern(r":::([\w-]+)\s*$").captures(token);
    let class_name = class_match
        .as_ref()
        .and_then(|caps| caps.get(1))
        .map(|item| item.as_str().to_owned());
    let token = class_match
        .as_ref()
        .and_then(|caps| caps.get(0))
        .map_or(token, |item| token[..item.start()].trim());
    let id_re = pattern(r"^([A-Za-z0-9_][\w.-]*)");
    let Some(caps) = id_re.captures(token) else {
        return Err(MermaidError::new(format!(
            "line {line_no}: cannot read node '{token}'"
        )));
    };
    let node_id = caps[1].to_owned();
    let id_end = caps.get(0).map_or(0, |item| item.end());
    let rest = token[id_end..].trim().to_owned();
    if rest.is_empty() {
        return Ok(NodeToken {
            id: node_id,
            text: None,
            class_name,
        });
    }
    for (open, close) in SHAPES {
        if rest.starts_with(open) && rest.ends_with(close) && rest.len() >= open.len() + close.len()
        {
            return Ok(NodeToken {
                id: node_id,
                text: Some(clean(&rest[open.len()..rest.len() - close.len()])),
                class_name,
            });
        }
    }
    Err(MermaidError::new(format!(
        "line {line_no}: cannot read the shape of node '{token}'"
    )))
}

struct Graph {
    order: Vec<String>,
    text: BTreeMap<String, String>,
    class_name: BTreeMap<String, String>,
    edges: Vec<(String, String, String, Option<String>)>,
    groups: Vec<String>,
    group_of: BTreeMap<String, usize>,
    direction: String,
    title: Option<String>,
}

impl Graph {
    fn touch(
        &mut self,
        node_id: &str,
        text: Option<String>,
        class_name: Option<String>,
        group: Option<usize>,
    ) {
        if !self.text.contains_key(node_id) {
            self.order.push(node_id.to_owned());
            self.text.insert(node_id.to_owned(), node_id.to_owned());
        }
        if let Some(text) = text {
            self.text.insert(node_id.to_owned(), text);
        }
        if let Some(class_name) = class_name {
            self.class_name.insert(node_id.to_owned(), class_name);
        }
        if let Some(group) = group {
            self.group_of.entry(node_id.to_owned()).or_insert(group);
        }
    }
}

struct EdgeSplit {
    nodes: Vec<String>,
    edges: Vec<(String, Option<String>)>,
}

fn split_edges(stmt: &str) -> EdgeSplit {
    let edge =
        pattern(r"\s*(<?(?:-\.+->|-\.+-|={2,}>|={3,}|-{2,}>|-{3,}|--[ox]))\s*(?:\|([^|]*)\|)?\s*");
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut rest = stmt;
    while let Some(found) = edge.captures(rest) {
        let Some(whole) = found.get(0) else {
            break;
        };
        nodes.push(rest[..whole.start()].to_owned());
        let label = found.get(2).map(|item| item.as_str().to_owned());
        edges.push((found[1].to_owned(), label));
        rest = &rest[whole.end()..];
    }
    nodes.push(rest.to_owned());
    EdgeSplit { nodes, edges }
}

fn read_front(lines: &[&str], graph: &mut Graph) -> usize {
    if lines.first().is_some_and(|line| line.trim() == "---")
        && let Some(end) = lines.iter().skip(1).position(|line| line.trim() == "---")
        && let Some(meta_lines) = lines.get(1..=end)
    {
        for meta in meta_lines {
            let (key, value) = meta.split_once(':').unwrap_or((*meta, ""));
            if key.trim().eq_ignore_ascii_case("title") {
                graph.title = Some(clean(value));
            }
        }
        end + 2
    } else {
        0
    }
}

fn record_chain(
    graph: &mut Graph,
    stmt: &str,
    line_no: usize,
    group: Option<usize>,
) -> Result<(), MermaidError> {
    let split = split_edges(&label_subs(stmt));
    let mut groups_of_nodes = Vec::new();
    for piece in &split.nodes {
        let mut ids = Vec::new();
        for token in piece.split('&') {
            let node = read_node(token, line_no)?;
            graph.touch(&node.id, node.text, node.class_name, group);
            ids.push(node.id);
        }
        groups_of_nodes.push(ids);
    }
    for (index, (op, label)) in split.edges.iter().enumerate() {
        let kind = if op.contains('.') { "planned" } else { "flow" };
        let (Some(left), Some(right)) =
            (groups_of_nodes.get(index), groups_of_nodes.get(index + 1))
        else {
            continue;
        };
        for left_id in left {
            for right_id in right {
                if left_id != right_id {
                    let label = label
                        .as_ref()
                        .map(|text| clean(text))
                        .filter(|text| !text.is_empty());
                    graph
                        .edges
                        .push((left_id.clone(), right_id.clone(), kind.to_owned(), label));
                }
            }
        }
    }
    Ok(())
}

fn read_statement(
    graph: &mut Graph,
    stmt: &str,
    n: usize,
    header: &Regex,
    header_seen: &mut bool,
    stack: &mut Vec<usize>,
) -> Result<(), MermaidError> {
    if !*header_seen {
        let Some(found) = header.captures(stmt) else {
            return Err(MermaidError::new(format!(
                "line {n}: expected 'flowchart LR' or 'graph TD' first, got '{stmt}'"
            )));
        };
        graph.direction = found
            .get(1)
            .map_or("TD", |item| item.as_str())
            .to_ascii_uppercase();
        *header_seen = true;
        return Ok(());
    }
    let word = stmt
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let has_edge =
        pattern(r"\s*(<?(?:-\.+->|-\.+-|={2,}>|={3,}|-{2,}>|-{3,}|--[ox]))").is_match(stmt);
    if matches!(
        word.as_str(),
        "classdef" | "style" | "linkstyle" | "click" | "direction"
    ) && !has_edge
    {
        return Ok(());
    }
    if word == "subgraph" {
        let rest = stmt[word.len()..].trim();
        let titled = pattern(r"^([\w.-]+)\s*\[\s*(.*?)\s*\]\s*$");
        let title = if let Some(found) = titled.captures(rest) {
            clean(&found[2])
        } else {
            let cleaned = clean(rest);
            if cleaned.is_empty() {
                format!("Group {}", graph.groups.len() + 1)
            } else {
                cleaned
            }
        };
        if stack.is_empty() {
            graph.groups.push(title);
            stack.push(graph.groups.len() - 1);
        } else if let Some(&group) = stack.first() {
            stack.push(group);
        }
        return Ok(());
    }
    if word == "end" {
        if stack.is_empty() {
            return Err(MermaidError::new(format!(
                "line {n}: 'end' without a matching 'subgraph'"
            )));
        }
        stack.pop();
        return Ok(());
    }
    if word == "class" && !has_edge {
        let parts: Vec<&str> = stmt.split_whitespace().collect();
        if let (Some(ids), Some(class_name)) = (parts.get(1), parts.get(2)) {
            for node_id in ids.split(',') {
                graph
                    .class_name
                    .insert(node_id.trim().to_owned(), (*class_name).to_owned());
            }
        }
        return Ok(());
    }
    record_chain(graph, stmt, n, stack.first().copied())
}

/// Read a Mermaid flowchart into its nodes, edges and groups.
///
/// # Errors
/// Returns an error naming the line that could not be read.
pub fn parse(source: &str) -> Result<Parsed, MermaidError> {
    let mut graph = Graph {
        order: Vec::new(),
        text: BTreeMap::new(),
        class_name: BTreeMap::new(),
        edges: Vec::new(),
        groups: Vec::new(),
        group_of: BTreeMap::new(),
        direction: "LR".to_owned(),
        title: None,
    };
    let lines: Vec<&str> = source.lines().collect();
    let index = read_front(&lines, &mut graph);
    let header = pattern(r"(?i)^(?:flowchart|graph)\s*(LR|RL|TD|TB|BT)?\s*;?\s*$");
    let mut header_seen = false;
    let mut stack: Vec<usize> = Vec::new();
    for (line_no, line) in lines.iter().enumerate().skip(index) {
        let raw = line.split("%%").next().unwrap_or("").trim();
        for stmt in raw
            .split(';')
            .map(str::trim)
            .filter(|stmt| !stmt.is_empty())
        {
            read_statement(
                &mut graph,
                stmt,
                line_no + 1,
                &header,
                &mut header_seen,
                &mut stack,
            )?;
        }
    }
    if !header_seen {
        return Err(MermaidError::new("no 'flowchart' or 'graph' line found"));
    }
    if !stack.is_empty() {
        return Err(MermaidError::new("a 'subgraph' is missing its 'end'"));
    }
    Ok(Parsed { graph })
}

pub struct Parsed {
    graph: Graph,
}

/// Lay a Mermaid flowchart out as a diagram.
///
/// # Errors
/// Returns an error naming a line that could not be read, or a layout that cannot be measured.
pub fn from_mermaid(
    source: &str,
    title: Option<&str>,
    theme: Option<Theme>,
) -> Result<Diagram, MermaidError> {
    let Parsed { graph } = parse(source)?;
    let theme = theme.unwrap_or_default();
    let mut diagram =
        Diagram::new(title.or(graph.title.as_deref()).unwrap_or("Flowchart")).theme(theme.clone());
    if graph.order.is_empty() {
        return Err(MermaidError::new("the flowchart has no nodes"));
    }
    let across = graph.direction == "LR" || graph.direction == "RL";
    let flip = graph.direction == "RL" || graph.direction == "BT";
    let back = back_edges(&graph);
    let forward: Vec<usize> = (0..graph.edges.len())
        .filter(|index| !back.contains(index))
        .collect();
    let layer = layers(&graph, &forward);
    let slots = Slots {
        n_layers: layer.values().copied().max().unwrap_or(0) + 1,
        flip,
    };
    let mut band_keys: Vec<Option<usize>> = Vec::new();
    for node in &graph.order {
        let key = graph.group_of.get(node).copied();
        if !band_keys.contains(&key) {
            band_keys.push(key);
        }
    }
    let cells = order_cells(&graph, &layer, &forward, &band_keys, slots.n_layers);
    let NodeMeasure {
        tiles,
        size,
        label_w,
    } = measure_nodes(&graph, &theme, &layer, &forward, slots);
    let grouped: Vec<bool> = band_keys.iter().map(Option::is_some).collect();
    let group_head = theme.pad + 8.0 + theme.line_h + 6.0;
    let step_gap = step_gaps(&graph, &theme, &layer, &band_keys, &label_w, across, slots);
    let item_gap = if across { 24.0 } else { 40.0 };
    let grid = Grid {
        graph: &graph,
        tiles: &tiles,
        size: &size,
        layer: &layer,
        slots,
        band_keys: &band_keys,
        cells: &cells,
        step_gap: &step_gap,
        item_gap,
        grouped: &grouped,
    };
    let placed = if across {
        place_across(&mut diagram, &grid, group_head)?
    } else {
        place_down(&mut diagram, &grid)?
    };
    for key in band_keys.iter().flatten() {
        let members: Vec<Placed> = graph
            .order
            .iter()
            .filter(|node| graph.group_of.get(node.as_str()) == Some(key))
            .filter_map(|node| placed.get(node).cloned())
            .collect();
        let Some(label) = graph.groups.get(*key) else {
            continue;
        };
        diagram.group(label, &members, None, None);
    }
    let any_group = grouped.iter().any(|flag| *flag);
    route_edges(
        &mut diagram,
        &graph,
        &placed,
        &layer,
        &back,
        Route {
            across,
            flip,
            group_head,
            any_group,
        },
    )?;
    Ok(diagram)
}

fn order_cells(
    graph: &Graph,
    layer: &BTreeMap<String, usize>,
    forward: &[usize],
    band_keys: &[Option<usize>],
    n_layers: usize,
) -> BTreeMap<(usize, usize), Vec<String>> {
    let mut pos: BTreeMap<String, i64> = graph
        .order
        .iter()
        .enumerate()
        .map(|(k, node)| (node.clone(), i64::try_from(k).unwrap_or(0)))
        .collect();
    let mut cells: BTreeMap<(usize, usize), Vec<String>> = BTreeMap::new();
    for node in &graph.order {
        let band = band_keys
            .iter()
            .position(|key| key == &graph.group_of.get(node).copied())
            .unwrap_or(0);
        let Some(&level) = layer.get(node) else {
            continue;
        };
        cells.entry((level, band)).or_default().push(node.clone());
    }
    let (preds, succs) = neighbours(graph, forward);
    for sweep in 0..6 {
        let neighbours = if sweep % 2 == 0 { &preds } else { &succs };
        let levels: Vec<usize> = if sweep % 2 == 0 {
            (0..n_layers).collect()
        } else {
            (0..n_layers).rev().collect()
        };
        for level in levels {
            for band in 0..band_keys.len() {
                let Some(cell) = cells.get_mut(&(level, band)) else {
                    continue;
                };
                if cell.is_empty() {
                    continue;
                }
                let mut keyed = Vec::new();
                for node in cell.iter() {
                    let Some(near) = neighbours.get(node) else {
                        continue;
                    };
                    let Some(&at) = pos.get(node) else {
                        continue;
                    };
                    let average = if near.is_empty() {
                        whole(at)
                    } else {
                        near.iter()
                            .map(|item| pos.get(item).copied().map_or(0.0, whole))
                            .sum::<f64>()
                            / count(near.len())
                    };
                    keyed.push((average, at, node.clone()));
                }
                keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                *cell = keyed.iter().map(|item| item.2.clone()).collect();
                for (k, node) in cell.iter().enumerate() {
                    pos.insert(
                        node.clone(),
                        i64::try_from(band).unwrap_or(0) * 1000 + i64::try_from(k).unwrap_or(0),
                    );
                }
            }
        }
    }
    cells
}

fn step_gaps(
    graph: &Graph,
    theme: &Theme,
    layer: &BTreeMap<String, usize>,
    band_keys: &[Option<usize>],
    label_w: &BTreeMap<usize, f64>,
    across: bool,
    slots: Slots,
) -> Vec<f64> {
    let group_head = theme.pad + 8.0 + theme.line_h + 6.0;
    if across {
        (0..slots.n_layers)
            .map(|level| {
                theme
                    .col_gap
                    .max(label_w.get(&level).copied().unwrap_or(0.0) + 56.0)
            })
            .collect()
    } else {
        let starts: BTreeSet<usize> = band_keys
            .iter()
            .filter_map(|key| *key)
            .map(|key| {
                graph
                    .order
                    .iter()
                    .filter(|node| graph.group_of.get(node.as_str()) == Some(&key))
                    .filter_map(|node| layer.get(node).copied())
                    .map(|level| slots.at(level))
                    .min()
                    .unwrap_or(0)
            })
            .collect();
        (0..slots.n_layers)
            .map(|level| {
                64.0 + if starts.contains(&(level + 1)) {
                    group_head + 24.0
                } else {
                    0.0
                }
            })
            .collect()
    }
}

fn back_edges(graph: &Graph) -> BTreeSet<usize> {
    let mut out_edges: BTreeMap<&str, Vec<usize>> = graph
        .order
        .iter()
        .map(|node| (node.as_str(), Vec::new()))
        .collect();
    for (index, (a, _, _, _)) in graph.edges.iter().enumerate() {
        if let Some(list) = out_edges.get_mut(a.as_str()) {
            list.push(index);
        }
    }
    let mut state: BTreeMap<&str, u8> = BTreeMap::new();
    let mut back = BTreeSet::new();
    for root in &graph.order {
        if state.contains_key(root.as_str()) {
            continue;
        }
        state.insert(root.as_str(), 1);
        let mut walk = vec![(root.as_str(), 0_usize)];
        while let Some((node, next)) = walk.last().copied() {
            let Some(edges) = out_edges.get(node) else {
                state.insert(node, 2);
                walk.pop();
                continue;
            };
            if next >= edges.len() {
                state.insert(node, 2);
                walk.pop();
                continue;
            }
            let Some(frame) = walk.last_mut() else {
                break;
            };
            frame.1 = next + 1;
            let Some(&edge) = edges.get(next) else {
                continue;
            };
            let Some(child) = graph.edges.get(edge).map(|item| item.1.as_str()) else {
                continue;
            };
            match state.get(child).copied() {
                Some(1) => {
                    back.insert(edge);
                }
                None => {
                    state.insert(child, 1);
                    walk.push((child, 0));
                }
                Some(_) => {}
            }
        }
    }
    back
}

fn layers(graph: &Graph, forward: &[usize]) -> BTreeMap<String, usize> {
    let mut layer: BTreeMap<String, usize> =
        graph.order.iter().map(|node| (node.clone(), 0)).collect();
    for _ in 0..graph.order.len() {
        let mut changed = false;
        for index in forward {
            let Some((a, b, _, _)) = graph.edges.get(*index) else {
                continue;
            };
            let (Some(&layer_b), Some(&layer_a)) = (layer.get(b), layer.get(a)) else {
                continue;
            };
            if layer_b < layer_a + 1 {
                layer.insert(b.clone(), layer_a + 1);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    layer
}

fn neighbours(
    graph: &Graph,
    forward: &[usize],
) -> (BTreeMap<String, Vec<String>>, BTreeMap<String, Vec<String>>) {
    let mut preds: BTreeMap<String, Vec<String>> = graph
        .order
        .iter()
        .map(|node| (node.clone(), Vec::new()))
        .collect();
    let mut succs: BTreeMap<String, Vec<String>> = graph
        .order
        .iter()
        .map(|node| (node.clone(), Vec::new()))
        .collect();
    for index in forward {
        let Some((a, b, _, _)) = graph.edges.get(*index) else {
            continue;
        };
        if let Some(list) = succs.get_mut(a) {
            list.push(b.clone());
        }
        if let Some(list) = preds.get_mut(b) {
            list.push(a.clone());
        }
    }
    (preds, succs)
}

#[derive(Clone, Copy)]
struct Grid<'a> {
    graph: &'a Graph,
    tiles: &'a BTreeMap<String, Tile>,
    size: &'a BTreeMap<String, (f64, f64)>,
    layer: &'a BTreeMap<String, usize>,
    slots: Slots,
    band_keys: &'a [Option<usize>],
    cells: &'a BTreeMap<(usize, usize), Vec<String>>,
    step_gap: &'a [f64],
    item_gap: f64,
    grouped: &'a [bool],
}

#[derive(Clone, Copy)]
struct Route {
    across: bool,
    flip: bool,
    group_head: f64,
    any_group: bool,
}

#[derive(Clone, Copy)]
struct Slots {
    n_layers: usize,
    flip: bool,
}

impl Slots {
    // lukendeasy/open-diagram 49982c0b93766aec4c2483da24afbda18e60c239 draws RL and BT
    // by slot, not by flipping coordinates afterwards. The group-head gap is reserved
    // in front of the first drawn step, so it stays on the leading side of the group.
    const fn at(self, level: usize) -> usize {
        if self.flip {
            self.n_layers - 1 - level
        } else {
            level
        }
    }
}

struct NodeMeasure {
    tiles: BTreeMap<String, Tile>,
    size: BTreeMap<String, (f64, f64)>,
    label_w: BTreeMap<usize, f64>,
}

fn measure_nodes(
    graph: &Graph,
    theme: &Theme,
    layer: &BTreeMap<String, usize>,
    forward: &[usize],
    slots: Slots,
) -> NodeMeasure {
    let mut tiles = BTreeMap::new();
    for node in &graph.order {
        let class_status = graph
            .class_name
            .get(node)
            .filter(|status| {
                theme.statuses.contains_key(status.as_str()) && status.as_str() != "live"
            })
            .cloned();
        let Some(label) = graph.text.get(node) else {
            continue;
        };
        let mut node_tile = Tile::new(label);
        if let Some(class_status) = class_status {
            node_tile = node_tile.status(class_status);
        }
        node_tile.name = Some(node.clone());
        tiles.insert(node.clone(), node_tile);
    }
    let mut size = BTreeMap::new();
    for node in &graph.order {
        if let Some(tile) = tiles.get(node) {
            size.insert(node.clone(), tile.measure(theme));
        }
    }
    let mut label_w = BTreeMap::new();
    for index in forward {
        let Some((from_id, to_id, _, label)) = graph.edges.get(*index) else {
            continue;
        };
        if let Some(label) = label
            && layer[to_id] == layer[from_id] + 1
        {
            let width = text_width(label, theme.small, "medium") + 14.0;
            let gap = slots.at(layer[from_id]).min(slots.at(layer[to_id]));
            let gap_w = label_w.entry(gap).or_insert(0.0);
            *gap_w = f64::max(*gap_w, width);
        }
    }
    NodeMeasure {
        tiles,
        size,
        label_w,
    }
}

fn cell_at(cells: &BTreeMap<(usize, usize), Vec<String>>, level: usize, band: usize) -> &[String] {
    cells.get(&(level, band)).map_or(&[], Vec::as_slice)
}

fn place_across(
    diagram: &mut Diagram,
    grid: &Grid<'_>,
    group_head: f64,
) -> Result<BTreeMap<String, Placed>, MermaidError> {
    let Grid {
        graph,
        tiles,
        size,
        layer,
        slots,
        band_keys,
        cells,
        step_gap,
        item_gap,
        grouped,
    } = *grid;
    let theme = diagram.theme.clone();
    let n_layers = slots.n_layers;
    let widths: Vec<f64> = (0..n_layers)
        .map(|level| {
            graph
                .order
                .iter()
                .filter(|node| slots.at(layer[*node]) == level)
                .map(|node| size[node].0)
                .fold(0.0, f64::max)
        })
        .collect();
    let mut band_h = Vec::new();
    for band in 0..band_keys.len() {
        let height = (0..n_layers)
            .map(|level| {
                let cell = cell_at(cells, level, band);
                cell.iter().map(|node| size[node].1).sum::<f64>()
                    + mul(item_gap, count(cell.len().saturating_sub(1)))
            })
            .fold(0.0, f64::max);
        band_h.push(height);
    }
    let mut xs = Vec::new();
    let mut x = 0.0;
    for level in 0..n_layers {
        xs.push(x);
        let width = widths.get(level).copied().unwrap_or(0.0);
        let gap = step_gap.get(level).copied().unwrap_or(0.0);
        x += width + gap;
    }
    let mut placed = BTreeMap::new();
    let mut y = 0.0;
    for band in 0..band_keys.len() {
        if band > 0 {
            let here = grouped.get(band).copied().unwrap_or(false);
            let prev = grouped.get(band - 1).copied().unwrap_or(false);
            y += if here || prev {
                mul(2.0, theme.pad + 8.0) + group_head + 8.0
            } else {
                theme.row_gap + 16.0
            };
        }
        let band_height = band_h.get(band).copied().unwrap_or(0.0);
        for level in 0..n_layers {
            let cell = cell_at(cells, level, band);
            let stack_h = cell.iter().map(|node| size[node].1).sum::<f64>()
                + mul(item_gap, count(cell.len().saturating_sub(1)));
            let mut cy = y + (band_height - stack_h) / 2.0;
            for node in cell {
                let tile = tiles[node].clone();
                let name = graph
                    .text
                    .get(node)
                    .cloned()
                    .unwrap_or_else(|| node.clone());
                let drawn = slots.at(level);
                let level_x = xs.get(drawn).copied().unwrap_or(0.0);
                let width = widths.get(drawn).copied().unwrap_or(0.0);
                placed.insert(
                    node.clone(),
                    diagram
                        .add_named(tile, level_x, cy, Some(width), Some(name))
                        .map_err(|err| MermaidError::new(err.to_string()))?,
                );
                cy += size[node].1 + item_gap;
            }
        }
        y += band_height;
    }
    Ok(placed)
}

fn place_down(
    diagram: &mut Diagram,
    grid: &Grid<'_>,
) -> Result<BTreeMap<String, Placed>, MermaidError> {
    let Grid {
        graph,
        tiles,
        size,
        layer,
        slots,
        band_keys,
        cells,
        step_gap,
        item_gap,
        grouped,
    } = *grid;
    let theme = diagram.theme.clone();
    let n_layers = slots.n_layers;
    let heights: Vec<f64> = (0..n_layers)
        .map(|level| {
            graph
                .order
                .iter()
                .filter(|node| slots.at(layer[*node]) == level)
                .map(|node| size[node].1)
                .fold(0.0, f64::max)
        })
        .collect();
    let mut band_w = Vec::new();
    for band in 0..band_keys.len() {
        let band_width = (0..n_layers)
            .map(|level| {
                let cell = cell_at(cells, level, band);
                cell.iter().map(|node| size[node].0).sum::<f64>()
                    + mul(item_gap, count(cell.len().saturating_sub(1)))
            })
            .fold(0.0, f64::max);
        band_w.push(band_width);
    }
    let mut ys = Vec::new();
    let mut y = 0.0;
    for level in 0..n_layers {
        ys.push(y);
        let height = heights.get(level).copied().unwrap_or(0.0);
        let gap = step_gap.get(level).copied().unwrap_or(0.0);
        y += height + gap;
    }
    let mut placed = BTreeMap::new();
    let mut x = 0.0;
    for band in 0..band_keys.len() {
        if band > 0 {
            let here = grouped.get(band).copied().unwrap_or(false);
            let prev = grouped.get(band - 1).copied().unwrap_or(false);
            x += if here || prev {
                mul(2.0, theme.pad + 8.0) + 16.0
            } else {
                theme.col_gap
            };
        }
        let band_width = band_w.get(band).copied().unwrap_or(0.0);
        for level in 0..n_layers {
            let cell = cell_at(cells, level, band);
            let row_w = cell.iter().map(|node| size[node].0).sum::<f64>()
                + mul(item_gap, count(cell.len().saturating_sub(1)));
            let mut cx = x + (band_width - row_w) / 2.0;
            for node in cell {
                let tile = tiles[node].clone();
                let name = graph
                    .text
                    .get(node)
                    .cloned()
                    .unwrap_or_else(|| node.clone());
                let drawn = slots.at(level);
                let level_y = ys.get(drawn).copied().unwrap_or(0.0);
                let level_h = heights.get(drawn).copied().unwrap_or(0.0);
                let cy = level_y + (level_h - size[node].1) / 2.0;
                placed.insert(
                    node.clone(),
                    diagram
                        .add_named(tile, cx, cy, None, Some(name))
                        .map_err(|err| MermaidError::new(err.to_string()))?,
                );
                cx += size[node].0 + item_gap;
            }
        }
        x += band_width;
    }
    Ok(placed)
}

fn route_edges(
    diagram: &mut Diagram,
    graph: &Graph,
    placed: &BTreeMap<String, Placed>,
    layer: &BTreeMap<String, usize>,
    back: &BTreeSet<usize>,
    route: Route,
) -> Result<(), MermaidError> {
    let Route {
        across,
        flip,
        group_head,
        any_group,
    } = route;
    let theme = &diagram.theme;
    let margin = theme.pad + 8.0 + if any_group { group_head } else { 0.0 };
    let lo = placed
        .values()
        .map(|panel| if across { panel.y } else { panel.x })
        .fold(f64::INFINITY, f64::min)
        - margin;
    let hi = placed
        .values()
        .map(|panel| {
            if across {
                panel.bottom()
            } else {
                panel.right()
            }
        })
        .fold(f64::NEG_INFINITY, f64::max)
        + theme.pad
        + 8.0;
    let span = lo + hi;
    let mid = span / 2.0;
    let mut low = 0_i64;
    let mut high = 0_i64;
    for (index, (a, b, kind, label)) in graph.edges.iter().enumerate() {
        let pa = &placed[a];
        let pb = &placed[b];
        let mut spec = LinkSpec::default().kind(kind);
        if let Some(label) = label {
            spec = spec.label(label);
        }
        if !back.contains(&index) && layer[b] == layer[a] + 1 {
            spec = if across {
                if flip {
                    spec.start("left").end("right")
                } else {
                    spec
                }
            } else if flip {
                spec.start("top").end("bottom")
            } else {
                spec.start("bottom").end("top")
            };
            diagram
                .link(pa, pb, spec)
                .map_err(|err| MermaidError::new(err.to_string()))?;
            continue;
        }
        let centre = if across {
            let sum = pa.cy() + pb.cy();
            sum / 2.0
        } else {
            let sum = pa.cx() + pb.cx();
            sum / 2.0
        };
        let side_high = centre > mid;
        let k = if side_high { high } else { low };
        if side_high {
            high += 1;
        } else {
            low += 1;
        }
        let at = if side_high {
            hi + 30.0 + mul(28.0, whole(k))
        } else {
            lo - 30.0 - mul(28.0, whole(k))
        };
        spec = match (across, flip) {
            (true, false) => spec.start("right").end("left").via_y(at),
            (true, true) => spec.start("left").end("right").via_y(at),
            (false, false) => spec.start("bottom").end("top").via_x(at),
            (false, true) => spec.start("top").end("bottom").via_x(at),
        };
        diagram
            .link(pa, pb, spec)
            .map_err(|err| MermaidError::new(err.to_string()))?;
    }
    Ok(())
}

/// Read a Mermaid file.
///
/// # Errors
/// Returns an error when the file cannot be read or a line cannot be parsed.
pub fn load_mermaid(path: impl AsRef<Path>, title: Option<&str>) -> Result<Diagram, MermaidError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)
        .map_err(|err| MermaidError::new(format!("{}: {err}", path.display())))?;
    from_mermaid(&text, title, None)
}
