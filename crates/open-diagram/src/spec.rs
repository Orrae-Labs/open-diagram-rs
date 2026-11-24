use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde_json::{Map, Value};

use crate::blocks::{Block, Card, Graph, Note, Panel, Resource, Row, Text, Tile, Tiles};
use crate::catalog;
use crate::diagram::{Diagram, Enclosed, EnclosureOpts, GroupId, Placed};
use crate::error::{DiagramError, SpecError};
use crate::num::{mul, py_repr, whole};
use crate::theme::{Status, Theme};

type IconFolders<'a> = Option<&'a [PathBuf]>;

const TOP_KEYS: &[&str] = &[
    "title",
    "subtitle",
    "tag",
    "note",
    "steps",
    "steps_label",
    "legend",
    "theme",
    "columns",
    "panels",
    "ring",
    "center",
    "gap",
    "row_gap",
    "valign",
    "start_angle",
    "groups",
    "links",
    "statuses",
    "resources",
    "enclosures",
];
const LAYOUTS: &[&str] = &["columns", "panels", "ring"];
const PANEL_KEYS: &[&str] = &[
    "id", "title", "sub", "status", "step", "footer", "tone", "width", "tiles", "cols", "label",
    "sections", "x", "y",
];
const TILE_KEYS: &[&str] = &["label", "sub", "status", "icon"];
const RESOURCE_KEYS: &[&str] = &[
    "id",
    "label",
    "sub",
    "icon",
    "icon_size",
    "max_width",
    "border",
    "x",
    "y",
];
const ENCLOSURE_KEYS: &[&str] = &[
    "id", "label", "members", "sub", "pad", "border", "fill", "stroke", "style", "tone",
];
const LINK_KEYS: &[&str] = &[
    "from", "to", "kind", "label", "start", "end", "lane", "via_x", "via_y", "cross_ok", "dash",
];
const GROUP_KEYS: &[&str] = &["label", "members", "sub"];

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_i64() || number.is_u64() => "int",
        Value::Number(_) => "float",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

fn keys(obj: &Value, allowed: &[&str], place: &str, required: &[&str]) -> Result<(), SpecError> {
    let Value::Object(map) = obj else {
        return Err(SpecError::new(format!(
            "{place}: expected an object, got {}",
            type_name(obj)
        )));
    };
    let mut extra: Vec<&str> = map
        .keys()
        .map(String::as_str)
        .filter(|key| !allowed.contains(key))
        .collect();
    extra.sort_unstable();
    if !extra.is_empty() {
        let shown = extra
            .iter()
            .map(|key| py_repr(key))
            .collect::<Vec<_>>()
            .join(", ");
        let mut allowed_names = allowed.to_vec();
        allowed_names.sort_unstable();
        return Err(SpecError::new(format!(
            "{place}: unknown key(s) {shown}; allowed: {}",
            allowed_names.join(", ")
        )));
    }
    for key in required {
        if !map.contains_key(*key) {
            return Err(SpecError::new(format!(
                "{place}: missing required key '{key}'"
            )));
        }
    }
    Ok(())
}

fn object<'a>(value: &'a Value, place: &str) -> Result<&'a Map<String, Value>, SpecError> {
    value.as_object().ok_or_else(|| {
        SpecError::new(format!(
            "{place}: expected an object, got {}",
            type_name(value)
        ))
    })
}

fn as_str<'a>(value: &'a Value, place: &str) -> Result<&'a str, SpecError> {
    value.as_str().ok_or_else(|| {
        SpecError::new(format!(
            "{place}: expected a string, got {}",
            type_name(value)
        ))
    })
}

fn as_f64(value: &Value, place: &str) -> Result<f64, SpecError> {
    value.as_f64().ok_or_else(|| {
        SpecError::new(format!(
            "{place}: expected a number, got {}",
            type_name(value)
        ))
    })
}

fn as_i64(value: &Value, place: &str) -> Result<i64, SpecError> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|n| i64::try_from(n).ok()))
        .ok_or_else(|| {
            SpecError::new(format!(
                "{place}: expected an integer, got {}",
                type_name(value)
            ))
        })
}

fn opt_str(map: &Map<String, Value>, key: &str, place: &str) -> Result<Option<String>, SpecError> {
    match map.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => Ok(Some(as_str(value, &format!("{place}.{key}"))?.to_owned())),
    }
}

fn opt_f64(map: &Map<String, Value>, key: &str, place: &str) -> Result<Option<f64>, SpecError> {
    match map.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => Ok(Some(as_f64(value, &format!("{place}.{key}"))?)),
    }
}

fn tile_from(item: &Value, place: &str, icons: IconFolders<'_>) -> Result<Tile, SpecError> {
    if let Some(label) = item.as_str() {
        return Ok(Tile::new(label));
    }
    keys(item, TILE_KEYS, place, &["label"])?;
    let map = object(item, place)?;
    let mut tile = Tile::new(as_str(&map["label"], &format!("{place}.label"))?);
    if let Some(sub) = opt_str(map, "sub", place)? {
        tile = tile.subtitle(sub);
    }
    if let Some(status) = opt_str(map, "status", place)? {
        tile = tile.status(status);
    }
    if let Some(icon) = map.get("icon") {
        tile = tile
            .icon(resolve_icon(icon, &format!("{place}.icon"), icons)?)
            .map_err(SpecError::from)?;
    }
    Ok(tile)
}

fn tiles_from(
    items: &Value,
    place: &str,
    cols: Option<usize>,
    label: Option<String>,
    icons: IconFolders<'_>,
) -> Result<Tiles, SpecError> {
    let Some(list) = items.as_array().filter(|list| !list.is_empty()) else {
        return Err(SpecError::new(format!(
            "{place}: tiles must be a non-empty list"
        )));
    };
    let mut tiles = Vec::new();
    for (n, item) in list.iter().enumerate() {
        tiles.push(tile_from(item, &format!("{place}[{n}]"), icons)?);
    }
    Tiles::new(tiles, cols, label).map_err(SpecError::from)
}

fn cols_of(map: &Map<String, Value>, place: &str) -> Result<Option<usize>, SpecError> {
    match map.get("cols") {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            let n = as_i64(value, &format!("{place}.cols"))?;
            usize::try_from(n)
                .map(Some)
                .map_err(|_| SpecError::new(format!("{place}.cols: expected an integer")))
        }
    }
}

fn section_from(obj: &Value, place: &str, icons: IconFolders<'_>) -> Result<Block, SpecError> {
    if obj.is_array() {
        return Ok(tiles_from(obj, place, None, None, icons)?.into());
    }
    if let Some(text) = obj.as_str() {
        return Ok(Text::new(text).into());
    }
    let Value::Object(map) = obj else {
        return Err(SpecError::new(format!(
            "{place}: a section is a list of tiles, a string, or an object"
        )));
    };
    if map.contains_key("tiles") {
        keys(obj, &["tiles", "cols", "label"], place, &[])?;
        return Ok(tiles_from(
            &map["tiles"],
            &format!("{place}.tiles"),
            cols_of(map, place)?,
            opt_str(map, "label", place)?,
            icons,
        )?
        .into());
    }
    if map.contains_key("text") {
        keys(obj, &["text"], place, &[])?;
        return Ok(Text::new(as_str(&map["text"], &format!("{place}.text"))?).into());
    }
    if map.contains_key("card") {
        keys(obj, &["card"], place, &[])?;
        return Ok(card_from(&map["card"], &format!("{place}.card"))?.into());
    }
    if map.contains_key("graph") {
        keys(obj, &["graph"], place, &[])?;
        return Ok(graph_from(&map["graph"], &format!("{place}.graph"))?.into());
    }
    if map.contains_key("row") {
        keys(obj, &["row"], place, &[])?;
        let list = map["row"]
            .as_array()
            .ok_or_else(|| SpecError::new(format!("{place}.row: expected a list")))?;
        let mut items = Vec::new();
        for (n, item) in list.iter().enumerate() {
            items.push(section_from(item, &format!("{place}.row[{n}]"), icons)?);
        }
        return Ok(Row::new(items, None).into());
    }
    Err(SpecError::new(format!(
        "{place}: a section needs one of 'tiles', 'text', 'card', 'graph' or 'row'"
    )))
}

fn card_from(value: &Value, place: &str) -> Result<Card, SpecError> {
    keys(
        value,
        &["title", "rows", "intro"],
        place,
        &["title", "rows"],
    )?;
    let map = object(value, place)?;
    let rows_v = map["rows"]
        .as_array()
        .ok_or_else(|| SpecError::new(format!("{place}.rows: expected a list")))?;
    let mut rows = Vec::new();
    for (n, row) in rows_v.iter().enumerate() {
        let pair = row
            .as_array()
            .ok_or_else(|| SpecError::new(format!("{place}.rows[{n}]: expected a pair")))?;
        if pair.len() != 2 {
            return Err(SpecError::new(format!(
                "{place}.rows[{n}]: expected a pair"
            )));
        }
        rows.push((
            as_str(&pair[0], &format!("{place}.rows[{n}]"))?.to_owned(),
            as_str(&pair[1], &format!("{place}.rows[{n}]"))?.to_owned(),
        ));
    }
    Ok(Card::new(
        as_str(&map["title"], &format!("{place}.title"))?,
        rows,
        opt_str(map, "intro", place)?,
    ))
}

fn graph_from(value: &Value, place: &str) -> Result<Graph, SpecError> {
    keys(
        value,
        &["nodes", "links", "at"],
        place,
        &["nodes", "links", "at"],
    )?;
    let map = object(value, place)?;
    let nodes_v = object(&map["nodes"], &format!("{place}.nodes"))?;
    let mut nodes = IndexMap::new();
    for (key, label) in nodes_v {
        nodes.insert(
            key.clone(),
            as_str(label, &format!("{place}.nodes.{key}"))?.to_owned(),
        );
    }
    let links_v = map["links"]
        .as_array()
        .ok_or_else(|| SpecError::new(format!("{place}.links: expected a list")))?;
    let mut links = Vec::new();
    for (n, link) in links_v.iter().enumerate() {
        let parts = link
            .as_array()
            .ok_or_else(|| SpecError::new(format!("{place}.links[{n}]: expected a list")))?;
        if parts.len() < 2 {
            return Err(SpecError::new(format!(
                "{place}.links[{n}]: expected from, to, and an optional label"
            )));
        }
        let label = if parts.len() >= 3 && !parts[2].is_null() {
            Some(as_str(&parts[2], &format!("{place}.links[{n}]"))?.to_owned())
        } else {
            None
        };
        links.push((
            as_str(&parts[0], &format!("{place}.links[{n}]"))?.to_owned(),
            as_str(&parts[1], &format!("{place}.links[{n}]"))?.to_owned(),
            label,
        ));
    }
    let at_v = object(&map["at"], &format!("{place}.at"))?;
    let mut at = IndexMap::new();
    for (key, spot) in at_v {
        let pair = spot
            .as_array()
            .filter(|pair| pair.len() == 2)
            .ok_or_else(|| SpecError::new(format!("{place}.at.{key}: expected two numbers")))?;
        at.insert(
            key.clone(),
            (
                as_f64(&pair[0], &format!("{place}.at.{key}"))?,
                as_f64(&pair[1], &format!("{place}.at.{key}"))?,
            ),
        );
    }
    Graph::new(nodes, links, at).map_err(|err| SpecError::new(format!("{place}: {err}")))
}

/// Build one panel from its JSON object.
///
/// # Errors
/// Returns an error naming the place of any mistake in `obj`.
pub fn panel_from_spec(obj: &Value, place: &str) -> Result<Panel, SpecError> {
    panel_with_icons(obj, place, None)
}

fn panel_with_icons(obj: &Value, place: &str, icons: IconFolders<'_>) -> Result<Panel, SpecError> {
    keys(obj, PANEL_KEYS, place, &["title"])?;
    let map = object(obj, place)?;
    let title = as_str(&map["title"], &format!("{place}.title"))?;
    let mut panel = Panel::new(title);
    if map.contains_key("tiles") {
        panel = panel.section(
            tiles_from(
                &map["tiles"],
                &format!("{place}.tiles"),
                cols_of(map, place)?,
                opt_str(map, "label", place)?,
                icons,
            )?
            .into(),
        );
    }
    if let Some(sections) = map.get("sections").and_then(Value::as_array) {
        for (n, section) in sections.iter().enumerate() {
            panel = panel.section(section_from(
                section,
                &format!("{place}.sections[{n}]"),
                icons,
            )?);
        }
    }
    if let Some(sub) = opt_str(map, "sub", place)? {
        panel = panel.subtitle(sub);
    }
    if let Some(status) = opt_str(map, "status", place)? {
        panel = panel.status(status);
    }
    if let Some(step) = map.get("step").filter(|value| !value.is_null()) {
        panel = panel.step(as_i64(step, &format!("{place}.step"))?);
    }
    if let Some(footer) = opt_str(map, "footer", place)? {
        panel = panel.footer(footer);
    }
    if let Some(tone) = opt_str(map, "tone", place)? {
        panel = panel.tone(tone);
    }
    if let Some(width) = opt_f64(map, "width", place)? {
        panel = panel.width(width);
    }
    let name = opt_str(map, "id", place)?.unwrap_or_else(|| title.to_owned());
    Ok(panel.name(name))
}

fn apply_number(theme: &mut Theme, key: &str, value: f64) -> bool {
    let field = match key {
        "title" => &mut theme.title,
        "subtitle" => &mut theme.subtitle,
        "legend" => &mut theme.legend,
        "panel_title" => &mut theme.panel_title,
        "panel_sub" => &mut theme.panel_sub,
        "tile" => &mut theme.tile,
        "tile_sub" => &mut theme.tile_sub,
        "body" => &mut theme.body,
        "small" => &mut theme.small,
        "min_text" => &mut theme.min_text,
        "lid" => &mut theme.lid,
        "depth" => &mut theme.depth,
        "pad" => &mut theme.pad,
        "tile_gap" => &mut theme.tile_gap,
        "section_gap" => &mut theme.section_gap,
        "tile_h" => &mut theme.tile_h,
        "line_h" => &mut theme.line_h,
        "header_h" => &mut theme.header_h,
        "header_h_sub" => &mut theme.header_h_sub,
        "panel_min_w" => &mut theme.panel_min_w,
        "tile_max_w" => &mut theme.tile_max_w,
        "radius" => &mut theme.radius,
        "col_gap" => &mut theme.col_gap,
        "row_gap" => &mut theme.row_gap,
        "min_gap" => &mut theme.min_gap,
        "margin" => &mut theme.margin,
        _ => return false,
    };
    *field = value;
    true
}

fn apply_string(theme: &mut Theme, key: &str, value: String) -> bool {
    let field = match key {
        "ink" => &mut theme.ink,
        "muted" => &mut theme.muted,
        "faint" => &mut theme.faint,
        "paper" => &mut theme.paper,
        "line" => &mut theme.line,
        "panel_fill" => &mut theme.panel_fill,
        "panel_stroke" => &mut theme.panel_stroke,
        "tile_fill" => &mut theme.tile_fill,
        "tile_stroke" => &mut theme.tile_stroke,
        "header_text" => &mut theme.header_text,
        "header_sub" => &mut theme.header_sub,
        "header_text_on_dark" => &mut theme.header_text_on_dark,
        "header_sub_on_dark" => &mut theme.header_sub_on_dark,
        "group_fill" => &mut theme.group_fill,
        "group_stroke" => &mut theme.group_stroke,
        "note_fill" => &mut theme.note_fill,
        "note_stroke" => &mut theme.note_stroke,
        "tag_color" => &mut theme.tag_color,
        "graph_fill" => &mut theme.graph_fill,
        "graph_grid" => &mut theme.graph_grid,
        "graph_line" => &mut theme.graph_line,
        _ => return false,
    };
    *field = value;
    true
}

/// Build a theme from the JSON `theme` object.
///
/// # Errors
/// Returns an error naming any unknown setting or malformed status.
pub fn theme_from_spec(obj: Option<&Value>) -> Result<Theme, SpecError> {
    let mut theme = Theme::default();
    let Some(obj) = obj else {
        return Ok(theme);
    };
    let map = object(obj, "theme")?;
    for (key, value) in map {
        if key == "tones" {
            let tones = object(value, "theme.tones")?;
            for (name, color) in tones {
                theme.tones.insert(
                    name.clone(),
                    as_str(color, &format!("theme.tones.{name}"))?.to_owned(),
                );
            }
        } else if key == "statuses" {
            let statuses = object(value, "theme.statuses")?;
            for (name, status) in statuses {
                keys(
                    status,
                    &["label", "stroke", "fill", "dash"],
                    &format!("theme.statuses.{name}"),
                    &["label", "stroke", "fill"],
                )?;
                let fields = object(status, &format!("theme.statuses.{name}"))?;
                theme.statuses.insert(
                    name.clone(),
                    Status::new(
                        as_str(&fields["label"], &format!("theme.statuses.{name}.label"))?,
                        as_str(&fields["stroke"], &format!("theme.statuses.{name}.stroke"))?,
                        as_str(&fields["fill"], &format!("theme.statuses.{name}.fill"))?,
                        opt_str(fields, "dash", &format!("theme.statuses.{name}"))?,
                    ),
                );
            }
        } else if key == "link_labels" {
            let labels = object(value, "theme.link_labels")?;
            for (name, label) in labels {
                theme.link_labels.insert(
                    name.clone(),
                    as_str(label, &format!("theme.link_labels.{name}"))?.to_owned(),
                );
            }
        } else if key == "badge_fill" || key == "badge_text" {
            let text = if value.is_null() {
                None
            } else {
                Some(as_str(value, &format!("theme.{key}"))?.to_owned())
            };
            if key == "badge_fill" {
                theme.badge_fill = text;
            } else {
                theme.badge_text = text;
            }
        } else if let Some(number) = value.as_f64() {
            if !apply_number(&mut theme, key, number) {
                return Err(SpecError::new(format!("theme: unknown setting '{key}'")));
            }
        } else if let Some(text) = value.as_str() {
            if !apply_string(&mut theme, key, text.to_owned()) {
                return Err(SpecError::new(format!("theme: unknown setting '{key}'")));
            }
        } else {
            return Err(SpecError::new(format!("theme: unknown setting '{key}'")));
        }
    }
    Ok(theme)
}

fn plain_panel(obj: &Value, place: &str, icons: IconFolders<'_>) -> Result<Panel, SpecError> {
    if let Some(map) = obj.as_object()
        && (map.contains_key("x") || map.contains_key("y"))
    {
        return Err(SpecError::new(format!(
            "{place}: x and y only work in the 'panels' layout"
        )));
    }
    panel_with_icons(obj, place, icons)
}

fn steps_from(value: &Value) -> Result<Vec<(i64, String)>, SpecError> {
    let list = value.as_array().ok_or_else(|| {
        SpecError::new(format!("steps: expected a list, got {}", type_name(value)))
    })?;
    let mut steps = Vec::new();
    for (n, step) in list.iter().enumerate() {
        let pair = step
            .as_array()
            .filter(|pair| pair.len() == 2)
            .ok_or_else(|| SpecError::new(format!("steps[{n}]: expected a number and a label")))?;
        steps.push((
            as_i64(&pair[0], &format!("steps[{n}]"))?,
            as_str(&pair[1], &format!("steps[{n}]"))?.to_owned(),
        ));
    }
    Ok(steps)
}

fn note_from(value: &Value) -> Result<Option<Note>, SpecError> {
    if value.is_null() || value.as_object().is_some_and(Map::is_empty) {
        return Ok(None);
    }
    keys(value, &["title", "lines"], "note", &["title"])?;
    let map = object(value, "note")?;
    let mut lines = Vec::new();
    if let Some(list) = map.get("lines").and_then(Value::as_array) {
        for (n, line) in list.iter().enumerate() {
            lines.push(as_str(line, &format!("note.lines[{n}]"))?.to_owned());
        }
    }
    Ok(Some(Note::new(as_str(&map["title"], "note.title")?, lines)))
}

fn place_columns(
    diagram: &mut Diagram,
    items: &[Value],
    spec: &Map<String, Value>,
    icons: IconFolders<'_>,
) -> Result<Vec<Placed>, SpecError> {
    let mut columns = Vec::new();
    for (i, col) in items.iter().enumerate() {
        let cells = col
            .as_array()
            .map_or_else(|| std::slice::from_ref(col), Vec::as_slice);
        let mut column = Vec::new();
        for (j, panel) in cells.iter().enumerate() {
            column.push(plain_panel(panel, &format!("columns[{i}][{j}]"), icons)?.into());
        }
        columns.push(column);
    }
    let placed = diagram
        .columns(
            columns,
            (0.0, 0.0),
            opt_f64(spec, "gap", "diagram")?,
            opt_f64(spec, "row_gap", "diagram")?,
            spec.get("valign")
                .and_then(Value::as_str)
                .unwrap_or("center"),
            true,
        )
        .map_err(SpecError::from)?;
    Ok(placed.into_iter().flatten().collect())
}

fn place_panels(
    diagram: &mut Diagram,
    items: &[Value],
    icons: IconFolders<'_>,
) -> Result<Vec<Placed>, SpecError> {
    let mut placed = Vec::new();
    for (n, obj) in items.iter().enumerate() {
        let spot = format!("panels[{n}]");
        let map = obj.as_object().ok_or_else(|| {
            SpecError::new(format!(
                "{spot}: each panel needs numbers for x and y (its top-left corner, in pixels)"
            ))
        })?;
        let (Some(x), Some(y)) = (map.get("x"), map.get("y")) else {
            return Err(SpecError::new(format!(
                "{spot}: each panel needs numbers for x and y (its top-left corner, in pixels)"
            )));
        };
        if !x.is_number() || !y.is_number() {
            return Err(SpecError::new(format!(
                "{spot}: each panel needs numbers for x and y (its top-left corner, in pixels)"
            )));
        }
        placed.push(
            diagram
                .add(
                    panel_with_icons(obj, &spot, icons)?,
                    as_f64(x, &format!("{spot}.x"))?,
                    as_f64(y, &format!("{spot}.y"))?,
                )
                .map_err(SpecError::from)?,
        );
    }
    Ok(placed)
}

fn place_ring(
    diagram: &mut Diagram,
    items: &[Value],
    spec: &Map<String, Value>,
    icons: IconFolders<'_>,
) -> Result<Vec<Placed>, SpecError> {
    let middle = match spec.get("center") {
        None => Vec::new(),
        Some(value) => value
            .as_array()
            .ok_or_else(|| SpecError::new("center: expected a list of panels"))?
            .clone(),
    };
    let mut around = Vec::new();
    for (n, panel) in items.iter().enumerate() {
        around.push(plain_panel(panel, &format!("ring[{n}]"), icons)?.into());
    }
    let mut inside = Vec::new();
    for (n, panel) in middle.iter().enumerate() {
        inside.push(plain_panel(panel, &format!("center[{n}]"), icons)?.into());
    }
    let start = spec
        .get("start_angle")
        .map(|value| as_f64(value, "start_angle"))
        .transpose()?
        .unwrap_or(-90.0);
    let (ring, center) = diagram
        .ring(
            around,
            0.0,
            0.0,
            opt_f64(spec, "gap", "diagram")?,
            start,
            inside,
        )
        .map_err(SpecError::from)?;
    let mut placed = ring;
    placed.extend(center);
    Ok(placed)
}

fn add_links(
    diagram: &mut Diagram,
    spec: &Map<String, Value>,
    placed: &[Placed],
    layout: &str,
    has_resources: bool,
) -> Result<(), SpecError> {
    let mut by_id = IndexMap::new();
    for panel in placed {
        by_id.insert(panel.name.clone(), panel.clone());
    }
    let kind = if has_resources {
        "panel or resource"
    } else {
        "panel"
    };
    let find =
        |name: &str, place: &str, ids: &IndexMap<String, Placed>| -> Result<Placed, SpecError> {
            ids.get(name).cloned().ok_or_else(|| {
                SpecError::new(format!(
                    "{place}: no {kind} with id '{name}'; ids: {}",
                    ids.keys().cloned().collect::<Vec<_>>().join(", ")
                ))
            })
        };
    if let Some(groups) = spec.get("groups").and_then(Value::as_array) {
        for (n, group) in groups.iter().enumerate() {
            let spot = format!("groups[{n}]");
            keys(group, GROUP_KEYS, &spot, &["label", "members"])?;
            let map = object(group, &spot)?;
            let members_v = map["members"]
                .as_array()
                .ok_or_else(|| SpecError::new(format!("{spot}.members: expected a list")))?;
            let mut members = Vec::new();
            for member in members_v {
                members.push(find(
                    as_str(member, &format!("{spot}.members"))?,
                    &format!("{spot}.members"),
                    &by_id,
                )?);
            }
            diagram.group(
                as_str(&map["label"], &format!("{spot}.label"))?,
                &members,
                opt_str(map, "sub", &spot)?,
                None,
            );
        }
    }
    place_enclosures(diagram, spec, &by_id)?;
    let top = placed
        .iter()
        .map(|panel| panel.y)
        .fold(f64::INFINITY, f64::min);
    let bottom = placed
        .iter()
        .map(crate::diagram::Placed::bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    let default_sides = if layout == "columns" {
        ("right", "left")
    } else {
        ("auto", "auto")
    };
    let mut lanes = IndexMap::from([("top".to_owned(), 0_i64), ("bottom".to_owned(), 0_i64)]);
    let Some(links) = spec.get("links").and_then(Value::as_array) else {
        return Ok(());
    };
    for (n, link) in links.iter().enumerate() {
        let spot = format!("links[{n}]");
        keys(link, LINK_KEYS, &spot, &["from", "to"])?;
        let map = object(link, &spot)?;
        let a = find(
            as_str(&map["from"], &format!("{spot}.from"))?,
            &format!("{spot}.from"),
            &by_id,
        )?;
        let b = find(
            as_str(&map["to"], &format!("{spot}.to"))?,
            &format!("{spot}.to"),
            &by_id,
        )?;
        let mut start = opt_str(map, "start", &spot)?.unwrap_or_else(|| default_sides.0.to_owned());
        let mut end = opt_str(map, "end", &spot)?.unwrap_or_else(|| default_sides.1.to_owned());
        let mut via_y = opt_f64(map, "via_y", &spot)?;
        if let Some(lane) = opt_str(map, "lane", &spot)? {
            let Some(k) = lanes.get(&lane).copied() else {
                return Err(SpecError::new(format!(
                    "{spot}.lane: use 'top' or 'bottom'"
                )));
            };
            lanes.insert(lane.clone(), k + 1);
            via_y = Some(if lane == "top" {
                top - 70.0 - mul(30.0, whole(k))
            } else {
                bottom + 44.0 + mul(30.0, whole(k))
            });
            if !map.contains_key("start") {
                start.clone_from(&lane);
            }
            if !map.contains_key("end") {
                end = lane;
            }
        }
        let mut spec_link = crate::diagram::LinkSpec::default()
            .kind(opt_str(map, "kind", &spot)?.unwrap_or_else(|| "flow".to_owned()))
            .start(start)
            .end(end);
        if let Some(label) = opt_str(map, "label", &spot)? {
            spec_link = spec_link.label(label);
        }
        if let Some(via_x) = opt_f64(map, "via_x", &spot)? {
            spec_link = spec_link.via_x(via_x);
        }
        if let Some(via_y) = via_y {
            spec_link = spec_link.via_y(via_y);
        }
        if map
            .get("cross_ok")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            spec_link = spec_link.cross_ok(true);
        }
        if let Some(dash) = opt_str(map, "dash", &spot)? {
            spec_link = spec_link.dash(dash);
        }
        diagram
            .link(&a, &b, spec_link)
            .map_err(|err: DiagramError| SpecError::new(format!("{spot}: {err}")))?;
    }
    Ok(())
}

/// Build a diagram from parsed JSON.
///
/// # Errors
/// Returns an error naming the place of any mistake in `spec`.
pub fn from_spec(spec: &Value) -> Result<Diagram, SpecError> {
    from_spec_with(spec, None)
}

/// Build a diagram from parsed JSON, looking up icon names in `icons`.
///
/// `icons` replaces the built-in icon folder. Pass `None` to use that folder.
///
/// # Errors
/// Returns an error naming the place of any mistake in `spec`.
pub fn from_spec_with(spec: &Value, icons: Option<&[PathBuf]>) -> Result<Diagram, SpecError> {
    keys(spec, TOP_KEYS, "diagram", &["title"])?;
    let map = object(spec, "diagram")?;
    let chosen: Vec<&str> = LAYOUTS
        .iter()
        .copied()
        .filter(|key| map.contains_key(*key))
        .collect();
    let has_resources = map.contains_key("resources");
    if has_resources && !chosen.is_empty() && chosen.as_slice() != ["panels"] {
        return Err(SpecError::new(format!(
            "resources: each resource is placed with x and y, so they work alone or with the 'panels' layout, not with '{}'",
            chosen[0]
        )));
    }
    if chosen.len() != 1 && !(chosen.is_empty() && has_resources) {
        let mut message = "diagram: give exactly one layout: 'columns' (left to right), 'panels' (each with x and y) or 'ring' (around a circle); cloud resources use 'resources', each with x and y".to_owned();
        if !chosen.is_empty() {
            message.push_str("; got ");
            message.push_str(&chosen.join(", "));
        }
        return Err(SpecError::new(message));
    }
    let layout = chosen.first().copied().unwrap_or("resources");
    if map.contains_key("center") && layout != "ring" {
        return Err(SpecError::new("center: only works with the 'ring' layout"));
    }
    let note = match map.get("note") {
        None | Some(Value::Null) => None,
        Some(value) => note_from(value)?,
    };
    let mut diagram = Diagram::new(as_str(&map["title"], "diagram.title")?)
        .theme(theme_from_spec(map.get("theme"))?);
    if let Some(subtitle) = opt_str(map, "subtitle", "diagram")? {
        diagram = diagram.subtitle(subtitle);
    }
    if let Some(tag) = opt_str(map, "tag", "diagram")? {
        diagram = diagram.tag(tag);
    }
    if let Some(note) = note {
        diagram = diagram.note(note);
    }
    if let Some(legend) = map.get("legend").and_then(Value::as_bool) {
        diagram = diagram.legend(legend);
    }
    if let Some(steps) = map.get("steps") {
        let label = opt_str(map, "steps_label", "diagram")?.unwrap_or_else(|| "Steps".to_owned());
        diagram = diagram.steps(steps_from(steps)?, label);
    }
    let mut placed = if layout == "resources" {
        Vec::new()
    } else {
        let items = map[layout]
            .as_array()
            .filter(|items| !items.is_empty())
            .ok_or_else(|| SpecError::new(format!("{layout}: expected a non-empty list")))?;
        match layout {
            "columns" => place_columns(&mut diagram, items, map, icons)?,
            "panels" => place_panels(&mut diagram, items, icons)?,
            _ => place_ring(&mut diagram, items, map, icons)?,
        }
    };
    if has_resources {
        place_resources(&mut diagram, map, &mut placed, icons)?;
    }
    add_links(&mut diagram, map, &placed, layout, has_resources)?;
    if let Some(statuses) = map
        .get("statuses")
        .filter(|value| value.as_object().is_some_and(|obj| !obj.is_empty()))
    {
        let mut named = IndexMap::new();
        for (name, state) in object(statuses, "statuses")? {
            named.insert(
                name.clone(),
                as_str(state, &format!("statuses.{name}"))?.to_owned(),
            );
        }
        diagram.set_statuses(&named)?;
    }
    Ok(diagram)
}

/// Read a JSON diagram description.
///
/// # Errors
/// Returns an error when the file cannot be read or the description is not valid.
pub fn load_spec(path: impl AsRef<Path>) -> Result<Diagram, SpecError> {
    load_spec_with(path, &[])
}

/// Read a JSON diagram description, searching `icons` after the file's folder and before the built-in icons.
///
/// # Errors
/// Returns an error when the file cannot be read or the description is not valid.
pub fn load_spec_with(path: impl AsRef<Path>, icons: &[PathBuf]) -> Result<Diagram, SpecError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)
        .map_err(|err| SpecError::new(format!("{}: {err}", path.display())))?;
    let data = serde_json::from_str(&text)
        .map_err(|err| SpecError::new(format!("{}: not valid JSON ({err})", path.display())))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = std::path::absolute(parent).unwrap_or_else(|_| parent.to_path_buf());
    let mut folders = vec![parent];
    folders.extend(icons.iter().cloned());
    folders.push(catalog::icon_dir());
    from_spec_with(&data, Some(&folders))
}

fn icon_folders(icons: IconFolders<'_>) -> Vec<PathBuf> {
    icons.map_or_else(|| vec![catalog::icon_dir()], <[PathBuf]>::to_vec)
}

fn resolve_icon(value: &Value, place: &str, icons: IconFolders<'_>) -> Result<String, SpecError> {
    let Some(name) = value.as_str().filter(|name| !name.trim().is_empty()) else {
        return Err(SpecError::new(format!(
            "{place}: an icon is a name inside an icon folder, such as 'aws/s3'"
        )));
    };
    let folders = icon_folders(icons);
    if let Some(found) = catalog::find_icon(name, &folders) {
        return Ok(found.to_string_lossy().into_owned());
    }
    let normalized = name.replace('\\', "/");
    if name.starts_with('/')
        || name.starts_with('\\')
        || name.contains(':')
        || normalized.split('/').any(|part| part == "..")
    {
        return Err(SpecError::new(format!(
            "{place}: icon {} must be a name inside an icon folder, such as 'aws/s3'; full paths and '..' are not allowed",
            py_repr(name)
        )));
    }
    let mut known = Vec::new();
    for folder in &folders {
        for icon in catalog::icon_names(folder) {
            if known.len() == 40 {
                break;
            }
            known.push(icon);
        }
    }
    let hint = if known.is_empty() {
        "; no icon folder holds any icons".to_owned()
    } else {
        format!("; icons available: {}", known.join(", "))
    };
    Err(SpecError::new(format!(
        "{place}: no icon named {}{hint}",
        py_repr(name)
    )))
}

fn place_resources(
    diagram: &mut Diagram,
    spec: &Map<String, Value>,
    placed: &mut Vec<Placed>,
    icons: IconFolders<'_>,
) -> Result<(), SpecError> {
    let Some(list) = spec
        .get("resources")
        .and_then(Value::as_array)
        .filter(|list| !list.is_empty())
    else {
        return Err(SpecError::new("resources: expected a non-empty list"));
    };
    for (n, obj) in list.iter().enumerate() {
        let place = format!("resources[{n}]");
        keys(obj, RESOURCE_KEYS, &place, &["label", "x", "y"])?;
        let map = object(obj, &place)?;
        if !map["x"].is_number() || !map["y"].is_number() {
            return Err(SpecError::new(format!(
                "{place}: each resource needs numbers for x and y (its top-left corner, in pixels)"
            )));
        }
        let label = as_str(&map["label"], &format!("{place}.label"))?;
        let icon = match map.get("icon") {
            Some(icon) => Some(resolve_icon(icon, &format!("{place}.icon"), icons)?),
            None => None,
        };
        let icon_size = opt_f64(map, "icon_size", &place)?.unwrap_or(64.0);
        let max_width = opt_f64(map, "max_width", &place)?.unwrap_or(180.0);
        let name = opt_str(map, "id", &place)?.unwrap_or_else(|| label.to_owned());
        let resource = Resource::new(
            label,
            icon,
            opt_str(map, "sub", &place)?,
            icon_size,
            max_width,
            opt_str(map, "border", &place)?,
            Some(name),
        )
        .map_err(|err| SpecError::new(format!("{place}: {err}")))?;
        if placed.iter().any(|item| item.name == resource.name) {
            return Err(SpecError::new(format!(
                "{place}.id: '{}' is already used; give each resource its own id",
                resource.name
            )));
        }
        let item = diagram
            .add(
                resource,
                as_f64(&map["x"], &format!("{place}.x"))?,
                as_f64(&map["y"], &format!("{place}.y"))?,
            )
            .map_err(SpecError::from)?;
        placed.push(item);
    }
    Ok(())
}

fn place_enclosures(
    diagram: &mut Diagram,
    spec: &Map<String, Value>,
    by_id: &IndexMap<String, Placed>,
) -> Result<(), SpecError> {
    let Some(list) = spec.get("enclosures").and_then(Value::as_array) else {
        return Ok(());
    };
    let mut later = Vec::new();
    for item in list {
        if let Some(map) = item.as_object() {
            let key = opt_str(map, "id", "enclosures")?
                .or_else(|| map.get("label").and_then(Value::as_str).map(str::to_owned));
            if let Some(key) = key {
                later.push(key);
            }
        }
    }
    let mut boxes: IndexMap<String, GroupId> = IndexMap::new();
    for (n, item) in list.iter().enumerate() {
        let place = format!("enclosures[{n}]");
        keys(item, ENCLOSURE_KEYS, &place, &["label", "members"])?;
        let map = object(item, &place)?;
        let members_v = map["members"]
            .as_array()
            .filter(|members| !members.is_empty())
            .ok_or_else(|| {
                SpecError::new(format!(
                    "{place}.members: expected a non-empty list of resource, panel or enclosure ids"
                ))
            })?;
        let mut members = Vec::new();
        for member in members_v {
            let name = as_str(member, &format!("{place}.members"))?;
            if let Some(group) = boxes.get(name) {
                members.push(Enclosed::Group(*group));
            } else if let Some(placed) = by_id.get(name) {
                members.push(Enclosed::Placed(placed.clone()));
            } else if later.iter().any(|key| key == name) {
                return Err(SpecError::new(format!(
                    "{place}.members: enclosure '{name}' is listed after this one; list enclosures from the inside out, so each one comes before the enclosures that hold it"
                )));
            } else {
                let mut ids: Vec<&str> = by_id.keys().map(String::as_str).collect();
                ids.extend(boxes.keys().map(String::as_str));
                return Err(SpecError::new(format!(
                    "{place}.members: no resource, panel or enclosure with id '{name}'; ids: {}",
                    ids.join(", ")
                )));
            }
        }
        let key = opt_str(map, "id", &place)?
            .unwrap_or_else(|| map["label"].as_str().unwrap_or("").to_owned());
        if boxes.contains_key(&key) || by_id.contains_key(&key) {
            return Err(SpecError::new(format!(
                "{place}.id: '{key}' is already used; give each enclosure its own id"
            )));
        }
        let pad = match map.get("pad") {
            None | Some(Value::Null) => 24.0,
            Some(value) => as_f64(value, &format!("{place}.pad"))?,
        };
        let opts = EnclosureOpts {
            sub: opt_str(map, "sub", &place)?,
            pad,
            border: map
                .get("border")
                .and_then(Value::as_str)
                .unwrap_or("dashed"),
            fill: opt_str(map, "fill", &place)?,
            stroke: opt_str(map, "stroke", &place)?,
            style: map
                .get("style")
                .and_then(Value::as_str)
                .unwrap_or("outline"),
            tone: map.get("tone").and_then(Value::as_str).unwrap_or("default"),
        };
        let group = diagram
            .enclosure(
                as_str(&map["label"], &format!("{place}.label"))?,
                &members,
                opts,
            )
            .map_err(|err| SpecError::new(format!("{place}: {err}")))?;
        boxes.insert(key, group);
    }
    Ok(())
}
