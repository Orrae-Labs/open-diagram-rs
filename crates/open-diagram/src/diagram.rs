#![expect(
    clippy::redundant_pub_crate,
    reason = "sibling modules use these items; pub would fail unreachable_pub because they are not the external API"
)]

use std::collections::BTreeSet;

use indexmap::IndexMap;

use crate::blocks::{self, Block, Note, Panel, apply_statuses};
use crate::describe;
use crate::error::{DiagramError, SaveError};
use crate::font::{cap_height, text_width};
use crate::num::{count, fmt, mul, py_fixed0, py_g, py_repr, trunc, whole};
use crate::svg::{Canvas, Letter, Paint};
use crate::theme::Theme;

const ARROW: f64 = 7.0;
const STUB: f64 = 16.0;
const CORNER: f64 = 10.0;
const TRACK: f64 = 12.0;

pub(crate) type Point = (f64, f64);

#[derive(Clone, Copy)]
struct LinkShape<'a> {
    index: usize,
    a_id: usize,
    b_id: usize,
    start: &'a str,
    end: &'a str,
    p0: Point,
    p3: Point,
}

fn strip_picture_suffix(path: &str) -> &str {
    let Some(tail) = path.get(path.len().saturating_sub(4)..) else {
        return path;
    };
    if tail.eq_ignore_ascii_case(".svg") || tail.eq_ignore_ascii_case(".png") {
        path.get(..path.len() - 4).unwrap_or(path)
    } else {
        path
    }
}

#[derive(Clone, Debug)]
pub struct Placed {
    id: usize,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Placed {
    #[must_use]
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    #[must_use]
    pub fn cx(&self) -> f64 {
        self.x + self.w / 2.0
    }
    #[must_use]
    pub fn cy(&self) -> f64 {
        self.y + self.h / 2.0
    }
    #[must_use]
    pub const fn rect(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.w, self.h)
    }
    #[must_use]
    pub const fn id(&self) -> usize {
        self.id
    }
}

#[derive(Clone, Debug)]
pub struct LinkSpec {
    pub kind: String,
    pub label: Option<String>,
    pub start: String,
    pub end: String,
    pub via_x: Option<f64>,
    pub via_y: Option<f64>,
    pub cross_ok: bool,
    pub dash: Option<String>,
    pub routing: Option<String>,
}

impl Default for LinkSpec {
    fn default() -> Self {
        Self {
            kind: "flow".to_owned(),
            label: None,
            start: "right".to_owned(),
            end: "left".to_owned(),
            via_x: None,
            via_y: None,
            cross_ok: false,
            dash: None,
            routing: None,
        }
    }
}

impl LinkSpec {
    #[must_use]
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    #[must_use]
    pub fn start(mut self, start: impl Into<String>) -> Self {
        self.start = start.into();
        self
    }
    #[must_use]
    pub fn end(mut self, end: impl Into<String>) -> Self {
        self.end = end.into();
        self
    }
    #[must_use]
    pub const fn via_x(mut self, x: f64) -> Self {
        self.via_x = Some(x);
        self
    }
    #[must_use]
    pub const fn via_y(mut self, y: f64) -> Self {
        self.via_y = Some(y);
        self
    }
    #[must_use]
    pub const fn cross_ok(mut self, ok: bool) -> Self {
        self.cross_ok = ok;
        self
    }
    #[must_use]
    pub fn dash(mut self, dash: impl Into<String>) -> Self {
        self.dash = Some(dash.into());
        self
    }
    /// `curved` or `orthogonal` for this line. `None` uses the diagram setting.
    #[must_use]
    pub fn routing(mut self, routing: impl Into<String>) -> Self {
        self.routing = Some(routing.into());
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GroupId(pub usize);

#[derive(Clone, Debug)]
pub enum Enclosed {
    Placed(Placed),
    Group(GroupId),
}

pub struct EnclosureOpts<'a> {
    pub sub: Option<String>,
    pub pad: f64,
    pub border: &'a str,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub style: &'a str,
    pub tone: &'a str,
}

pub(crate) struct Node {
    pub(crate) placed: Placed,
    pub(crate) block: Block,
}

pub(crate) struct Link {
    pub(crate) a: usize,
    pub(crate) b: usize,
    pub(crate) kind: String,
    pub(crate) label: Option<String>,
    pub(crate) start: String,
    pub(crate) end: String,
    pub(crate) via_x: Option<f64>,
    pub(crate) via_y: Option<f64>,
    pub(crate) cross_ok: bool,
    pub(crate) offsets: [f64; 2],
    pub(crate) d: String,
    pub(crate) tip: Point,
    pub(crate) direction: Point,
    pub(crate) samples: Vec<Point>,
    pub(crate) label_spots: Vec<Point>,
    pub(crate) label_at: Point,
    pub(crate) want_start: String,
    pub(crate) want_end: String,
    pub(crate) start_pt: Point,
    pub(crate) dash: Option<String>,
    pub(crate) routing: Option<String>,
    pub(crate) points: Vec<Point>,
    pub(crate) track: Option<(usize, f64, f64)>,
    pub(crate) alone: [bool; 2],
}

pub(crate) struct Group {
    pub(crate) label: String,
    pub(crate) members: Vec<usize>,
    pub(crate) sub: Option<String>,
    pub(crate) pad: f64,
    pub(crate) rect: (f64, f64, f64, f64),
    pub(crate) border: String,
    pub(crate) fill: Option<String>,
    pub(crate) stroke: Option<String>,
    pub(crate) children: Vec<usize>,
    pub(crate) enclosure: bool,
    pub(crate) style: String,
    pub(crate) tone: String,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub svg: String,
    pub width: f64,
    pub height: f64,
    pub min_text: Option<f64>,
    pub problems: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct LineView {
    pub a: String,
    pub b: String,
    pub kind: String,
    pub label: Option<String>,
    pub start: String,
    pub end: String,
}

struct LegendItem {
    kind: String,
    key: String,
    label: String,
}

struct HeaderPlan {
    width: f64,
    height: f64,
    rows: Vec<Vec<LegendItem>>,
    steps: bool,
    tag_w: f64,
    note_w: f64,
    centre: bool,
}

pub struct Diagram {
    pub title: String,
    pub subtitle: Option<String>,
    pub tag: Option<String>,
    pub note: Option<Note>,
    pub theme: Theme,
    pub legend: bool,
    pub steps: Vec<(i64, String)>,
    pub steps_label: String,
    pub(crate) routing: String,
    pub(crate) nodes: Vec<Node>,
    pub(crate) links: Vec<Link>,
    pub(crate) groups: Vec<Group>,
    early: Vec<String>,
}

impl Diagram {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            tag: None,
            note: None,
            theme: Theme::default(),
            legend: true,
            steps: Vec::new(),
            steps_label: "Steps".to_owned(),
            routing: "curved".to_owned(),
            nodes: Vec::new(),
            links: Vec::new(),
            groups: Vec::new(),
            early: Vec::new(),
        }
    }

    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }
    #[must_use]
    pub fn note(mut self, note: Note) -> Self {
        self.note = Some(note);
        self
    }
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
    #[must_use]
    pub const fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
        self
    }
    #[must_use]
    pub fn steps(mut self, steps: Vec<(i64, String)>, label: impl Into<String>) -> Self {
        self.steps = steps;
        self.steps_label = label.into();
        self
    }

    /// Draw every self-routed line as horizontal and vertical runs when `routing` is `orthogonal`.
    ///
    /// # Errors
    /// Returns an error when `routing` is not `curved` or `orthogonal`.
    pub fn routing(mut self, routing: impl Into<String>) -> Result<Self, DiagramError> {
        let routing = routing.into();
        if !matches!(routing.as_str(), "curved" | "orthogonal") {
            return Err(DiagramError::new(format!(
                "unknown routing {}; use curved or orthogonal",
                py_repr(&routing)
            )));
        }
        self.routing = routing;
        Ok(self)
    }

    /// The routing set with [`Diagram::routing`]: `curved` or `orthogonal`.
    #[must_use]
    pub fn routing_style(&self) -> &str {
        &self.routing
    }

    pub fn placed(&self) -> impl Iterator<Item = &Placed> {
        self.nodes.iter().map(|n| &n.placed)
    }

    #[must_use]
    pub fn lines(&self) -> Vec<LineView> {
        self.links
            .iter()
            .filter_map(|line| {
                let a = self.nodes.get(line.a)?.placed.name.clone();
                let b = self.nodes.get(line.b)?.placed.name.clone();
                Some(LineView {
                    a,
                    b,
                    kind: line.kind.clone(),
                    label: line.label.clone(),
                    start: line.start.clone(),
                    end: line.end.clone(),
                })
            })
            .collect()
    }

    /// # Errors
    /// Returns an error when the block's text or status cannot be measured.
    pub fn add(&mut self, block: impl Into<Block>, x: f64, y: f64) -> Result<Placed, DiagramError> {
        self.add_width(block.into(), x, y, None, None)
    }

    /// # Errors
    /// Returns an error when the block's text or status cannot be measured.
    pub fn add_named(
        &mut self,
        block: impl Into<Block>,
        x: f64,
        y: f64,
        width: Option<f64>,
        name: Option<String>,
    ) -> Result<Placed, DiagramError> {
        self.add_width(block.into(), x, y, width, name)
    }

    fn add_width(
        &mut self,
        block: Block,
        x: f64,
        y: f64,
        width: Option<f64>,
        name: Option<String>,
    ) -> Result<Placed, DiagramError> {
        let (mut measured_w, measured_h) = block.measure(&self.theme)?;
        let mut label = name
            .or_else(|| block.explicit_name().map(str::to_owned))
            .or_else(|| block.title().map(str::to_owned))
            .unwrap_or_else(|| format!("block {}", self.nodes.len() + 1));
        if self.nodes.iter().any(|n| n.placed.name == label) {
            let copies = self
                .nodes
                .iter()
                .filter(|node| node.placed.name.starts_with(&label))
                .count()
                + 1;
            label = format!("{label} ({copies})");
        }
        if let Some(width) = width {
            if width + 0.5 < measured_w {
                self.early.push(format!(
                    "'{label}' needs {} px wide but was given {}; using {}",
                    py_fixed0(measured_w),
                    py_fixed0(width),
                    py_fixed0(measured_w)
                ));
            }
            measured_w = measured_w.max(width);
        }
        let placed = Placed {
            id: self.nodes.len(),
            name: label,
            x,
            y,
            w: measured_w,
            h: measured_h,
        };
        let out = placed.clone();
        self.nodes.push(Node { placed, block });
        Ok(out)
    }

    /// # Errors
    /// Returns an error when `align` is not top, center, or bottom, or the block cannot be measured.
    pub fn right_of(
        &mut self,
        refer: &Placed,
        block: impl Into<Block>,
        gap: Option<f64>,
        align: &str,
        width: Option<f64>,
        name: Option<String>,
    ) -> Result<Placed, DiagramError> {
        let block = block.into();
        let (_, h) = block.measure(&self.theme)?;
        let gap = gap.unwrap_or(self.theme.col_gap);
        let y = match align {
            "top" => refer.y,
            "center" => refer.cy() - h / 2.0,
            "bottom" => refer.bottom() - h,
            other => return Err(DiagramError::new(format!("unknown align {other}"))),
        };
        self.add_width(block, refer.right() + gap, y, width, name)
    }

    /// # Errors
    /// Returns an error when `align` is not left, center, or right, or the block cannot be measured.
    pub fn below(
        &mut self,
        refer: &Placed,
        block: impl Into<Block>,
        gap: Option<f64>,
        align: &str,
        width: Option<f64>,
        name: Option<String>,
    ) -> Result<Placed, DiagramError> {
        let block = block.into();
        let (mw, _) = block.measure(&self.theme)?;
        let w = mw.max(width.unwrap_or(0.0));
        let gap = gap.unwrap_or(self.theme.row_gap);
        let x = match align {
            "left" => refer.x,
            "center" => refer.cx() - w / 2.0,
            "right" => refer.right() - w,
            other => return Err(DiagramError::new(format!("unknown align {other}"))),
        };
        self.add_width(block, x, refer.bottom() + gap, width, name)
    }

    /// # Errors
    /// Returns an error when a block cannot be measured.
    pub fn stack(
        &mut self,
        blocks: Vec<Block>,
        x: f64,
        y: f64,
        gap: Option<f64>,
    ) -> Result<Vec<Placed>, DiagramError> {
        let cols = self.columns(vec![blocks], (x, y), None, gap, "top", true)?;
        Ok(cols.into_iter().next().unwrap_or_default())
    }

    /// # Errors
    /// Returns an error when `valign` is unknown or a block cannot be measured.
    pub fn columns(
        &mut self,
        cols: Vec<Vec<Block>>,
        origin: (f64, f64),
        gap: Option<f64>,
        row_gap: Option<f64>,
        valign: &str,
        same_width: bool,
    ) -> Result<Vec<Vec<Placed>>, DiagramError> {
        let (origin_x, origin_y) = origin;
        let gap = gap.unwrap_or(self.theme.col_gap);
        let row_gap = row_gap.unwrap_or(self.theme.row_gap);
        let mut measured = Vec::new();
        for col in cols {
            if col.is_empty() {
                return Err(DiagramError::new("a column needs at least one block"));
            }
            let mut items = Vec::new();
            for block in col {
                let size = block.measure(&self.theme)?;
                items.push((block, size));
            }
            measured.push(items);
        }
        let widths: Vec<f64> = measured
            .iter()
            .map(|col| col.iter().map(|(_, s)| s.0).fold(0.0, f64::max))
            .collect();
        let heights: Vec<f64> = measured
            .iter()
            .map(|col| {
                col.iter().map(|(_, s)| s.1).sum::<f64>()
                    + mul(row_gap, count(col.len().saturating_sub(1)))
            })
            .collect();
        let tallest = heights.iter().copied().fold(0.0, f64::max);
        let mut out = Vec::new();
        let mut cx = origin_x;
        for (col, (column_width, column_height)) in measured
            .into_iter()
            .zip(widths.iter().copied().zip(heights.iter().copied()))
        {
            let mut cy = origin_y
                + match valign {
                    "top" => 0.0,
                    "center" => (tallest - column_height) / 2.0,
                    "bottom" => tallest - column_height,
                    other => return Err(DiagramError::new(format!("unknown valign {other}"))),
                };
            let mut placed = Vec::new();
            for (block, (width, height)) in col {
                let px = if same_width {
                    cx
                } else {
                    cx + (column_width - width) / 2.0
                };
                let given_width = if same_width { Some(column_width) } else { None };
                placed.push(self.add_width(block, px, cy, given_width, None)?);
                cy += height + row_gap;
            }
            out.push(placed);
            cx += column_width + gap;
        }
        Ok(out)
    }

    /// # Errors
    /// Returns an error when the ring has fewer than two blocks or a block cannot be measured.
    pub fn ring(
        &mut self,
        blocks: Vec<Block>,
        x: f64,
        y: f64,
        gap: Option<f64>,
        start_angle: f64,
        center: Vec<Block>,
    ) -> Result<(Vec<Placed>, Vec<Placed>), DiagramError> {
        let gap = gap.unwrap_or(self.theme.col_gap);
        if blocks.len() < 2 {
            return Err(DiagramError::new("ring() needs at least two blocks"));
        }
        let mut sizes = Vec::new();
        for block in &blocks {
            sizes.push(block.measure(&self.theme)?);
        }
        let mut mid_sizes = Vec::new();
        for block in &center {
            mid_sizes.push(block.measure(&self.theme)?);
        }
        let mid_w = mid_sizes.iter().map(|s| s.0).fold(0.0, f64::max);
        let mid_h = mid_sizes.iter().map(|s| s.1).sum::<f64>()
            + mul(self.theme.row_gap, count(mid_sizes.len().saturating_sub(1)));
        let n = blocks.len();
        let angles: Vec<f64> = (0..n)
            .map(|i| (start_angle + mul(360.0, count(i)) / count(n)).to_radians())
            .collect();
        let mean_w = sizes.iter().map(|s| s.0).sum::<f64>() / count(n);
        let mean_h = sizes.iter().map(|s| s.1).sum::<f64>() / count(n);
        let stretch = 1.8_f64.min(1.0_f64.max(mean_w / mean_h));
        let rects = |r: f64| -> Vec<(f64, f64, f64, f64)> {
            angles
                .iter()
                .zip(&sizes)
                .map(|(a, (w, h))| {
                    (
                        mul(mul(r, stretch), a.cos()) - w / 2.0,
                        mul(r, a.sin()) - h / 2.0,
                        *w,
                        *h,
                    )
                })
                .collect()
        };
        let mid_rect = (!center.is_empty()).then_some((-mid_w / 2.0, -mid_h / 2.0, mid_w, mid_h));
        let mut r = 40.0;
        let rs = loop {
            let rs = rects(r);
            let mut ok = rs.iter().enumerate().all(|(i, left)| {
                rs.iter()
                    .skip(i + 1)
                    .all(|right| separation(*left, *right) >= gap)
            });
            if ok && let Some(mid) = mid_rect {
                ok = rs.iter().all(|box_| separation(*box_, mid) >= gap);
            }
            if ok {
                break rs;
            }
            r += 6.0;
        };
        let mut everything = rs.clone();
        if let Some(mid) = mid_rect {
            everything.push(mid);
        }
        let left = everything.iter().map(|b| b.0).fold(f64::INFINITY, f64::min);
        let north = everything.iter().map(|b| b.1).fold(f64::INFINITY, f64::min);
        let mut placed = Vec::new();
        for (block, box_) in blocks.into_iter().zip(rs) {
            placed.push(self.add_width(
                block,
                x + box_.0 - left,
                y + box_.1 - north,
                None,
                None,
            )?);
        }
        let mut middle_placed = Vec::new();
        if let Some(mid) = mid_rect {
            let mut cy = y + mid.1 - north;
            for (block, (w, h)) in center.into_iter().zip(mid_sizes) {
                let _ = w;
                middle_placed.push(self.add_width(
                    block,
                    x + mid.0 - left,
                    cy,
                    Some(mid_w),
                    None,
                )?);
                cy += h + self.theme.row_gap;
            }
        }
        Ok((placed, middle_placed))
    }

    pub fn group(
        &mut self,
        label: impl Into<String>,
        members: &[Placed],
        sub: Option<String>,
        pad: Option<f64>,
    ) -> GroupId {
        let id = self.groups.len();
        self.groups.push(Group {
            label: label.into(),
            members: members.iter().map(Placed::id).collect(),
            sub,
            pad: pad.unwrap_or(self.theme.pad + 8.0),
            rect: (0.0, 0.0, 0.0, 0.0),
            border: "dashed".to_owned(),
            fill: None,
            stroke: None,
            children: Vec::new(),
            enclosure: false,
            style: "outline".to_owned(),
            tone: "default".to_owned(),
        });
        GroupId(id)
    }

    /// # Errors
    /// Returns an error when the border, pad, style, tone, or membership is not valid.
    pub fn enclosure(
        &mut self,
        label: impl Into<String>,
        members: &[Enclosed],
        opts: EnclosureOpts<'_>,
    ) -> Result<GroupId, DiagramError> {
        let EnclosureOpts {
            sub,
            pad,
            border,
            fill,
            stroke,
            style,
            tone,
        } = opts;
        let label = label.into();
        if !matches!(border, "solid" | "dashed" | "dotted") {
            return Err(DiagramError::new(format!(
                "enclosure {}: border must be solid, dashed or dotted",
                py_repr(&label)
            )));
        }
        if !pad.is_finite() || pad < 0.0 {
            return Err(DiagramError::new(format!(
                "enclosure {}: pad must be a non-negative finite number",
                py_repr(&label)
            )));
        }
        if !matches!(style, "outline" | "panel") {
            return Err(DiagramError::new(format!(
                "enclosure {}: style must be outline or panel",
                py_repr(&label)
            )));
        }
        self.theme.tone(tone)?;
        let mut nodes = Vec::new();
        let mut children = Vec::new();
        let reject = || {
            DiagramError::new(format!(
                "enclosure {}: members must belong to this diagram",
                py_repr(&label)
            ))
        };
        for member in members {
            let found = match member {
                Enclosed::Placed(p) => match self.nodes.get(p.id) {
                    Some(node) if node.placed.name == p.name => vec![p.id],
                    Some(_) | None => return Err(reject()),
                },
                Enclosed::Group(GroupId(id)) => match self.groups.get(*id) {
                    Some(group) => {
                        children.push(*id);
                        group.members.clone()
                    }
                    None => return Err(reject()),
                },
            };
            for node in found {
                if !nodes.contains(&node) {
                    nodes.push(node);
                }
            }
        }
        if nodes.is_empty() {
            return Err(DiagramError::new(format!(
                "enclosure {}: needs at least one resource",
                py_repr(&label)
            )));
        }
        let id = self.groups.len();
        self.groups.push(Group {
            label,
            members: nodes,
            sub,
            pad,
            rect: (0.0, 0.0, 0.0, 0.0),
            border: border.to_owned(),
            fill,
            stroke,
            children,
            enclosure: true,
            style: style.to_owned(),
            tone: tone.to_owned(),
        });
        Ok(GroupId(id))
    }

    /// # Errors
    /// Returns an error when a side, kind, dash, or route option is not valid.
    pub fn link(&mut self, a: &Placed, b: &Placed, spec: LinkSpec) -> Result<(), DiagramError> {
        for side in [&spec.start, &spec.end] {
            if side != "auto" && side_vector(side).is_none() {
                return Err(DiagramError::new(format!(
                    "unknown side {}; use left, right, top, bottom or auto",
                    py_repr(side)
                )));
            }
        }
        if !self.theme.link_labels.contains_key(&spec.kind) {
            let names = self
                .theme
                .link_labels
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            return Err(DiagramError::new(format!(
                "unknown line kind {}; use one of {names}",
                py_repr(&spec.kind)
            )));
        }
        if !matches!(
            spec.dash.as_deref(),
            None | Some("solid" | "dashed" | "dotted")
        ) {
            return Err(DiagramError::new(
                "link dash must be solid, dashed, dotted or None",
            ));
        }
        if spec.via_x.is_some() && spec.via_y.is_some() {
            return Err(DiagramError::new("give via_x or via_y, not both"));
        }
        if let Some(routing) = &spec.routing
            && !matches!(routing.as_str(), "curved" | "orthogonal")
        {
            return Err(DiagramError::new(format!(
                "unknown routing {}; use curved or orthogonal",
                py_repr(routing)
            )));
        }
        self.links.push(Link {
            a: a.id,
            b: b.id,
            kind: spec.kind,
            label: spec.label,
            start: spec.start.clone(),
            end: spec.end.clone(),
            via_x: spec.via_x,
            via_y: spec.via_y,
            cross_ok: spec.cross_ok,
            offsets: [0.0, 0.0],
            d: String::new(),
            tip: (0.0, 0.0),
            direction: (0.0, 0.0),
            samples: Vec::new(),
            label_spots: Vec::new(),
            label_at: (0.0, 0.0),
            want_start: spec.start,
            want_end: spec.end,
            start_pt: (0.0, 0.0),
            dash: spec.dash,
            routing: spec.routing,
            points: Vec::new(),
            track: None,
            alone: [false, false],
        });
        Ok(())
    }

    /// # Errors
    /// Returns an error when a state is unknown or a name matches nothing.
    pub fn set_statuses(
        &mut self,
        statuses: &IndexMap<String, String>,
    ) -> Result<(), DiagramError> {
        let bad: BTreeSet<_> = statuses
            .values()
            .filter(|s| !self.theme.statuses.contains_key(s.as_str()))
            .cloned()
            .collect();
        if !bad.is_empty() {
            let bad = bad
                .into_iter()
                .map(|s| py_repr(&s))
                .collect::<Vec<_>>()
                .join(", ");
            let names = self
                .theme
                .statuses
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            return Err(DiagramError::new(format!(
                "unknown state(s) {bad}; this theme has: {names}"
            )));
        }
        let mut found = BTreeSet::new();
        for node in &mut self.nodes {
            apply_statuses(&mut node.block, statuses, &mut found);
        }
        let missing: Vec<_> = statuses
            .keys()
            .filter(|n| !found.contains(n.as_str()))
            .map(|n| py_repr(n))
            .collect();
        if !missing.is_empty() {
            return Err(DiagramError::new(format!(
                "no panel or tile is named {}",
                missing.join(", ")
            )));
        }
        Ok(())
    }

    #[must_use]
    pub fn describe(&self) -> String {
        describe::describe(self)
    }

    /// Write `path`.svg and, when `png` is set, `path`.png. Files are written before a strict layout error.
    ///
    /// # Errors
    /// Returns a diagram error when the picture cannot be drawn, an I/O error when a file cannot be written,
    /// and a layout error when `strict` is set and the checks found problems.
    pub fn save(
        &mut self,
        path: &str,
        png: bool,
        scale: f64,
        strict: bool,
    ) -> Result<Report, SaveError> {
        let base = strip_picture_suffix(path);
        let svg_path = format!("{base}.svg");
        if let Some(parent) = std::path::Path::new(&svg_path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .map_err(|err| SaveError::Io(format!("{svg_path}: {err}")))?;
        }
        let mut report = self.render()?;
        std::fs::write(&svg_path, report.svg.as_bytes())
            .map_err(|err| SaveError::Io(format!("{svg_path}: {err}")))?;
        report.files.push(svg_path);
        if png && let Some(svg_file) = report.files.first() {
            let png_path = format!("{base}.png");
            if let Err(err) = crate::render::svg_to_png(
                std::path::Path::new(svg_file),
                std::path::Path::new(&png_path),
                report.width,
                report.height,
                scale,
                None,
            ) {
                println!("PNG skipped: {err}");
            } else {
                report.files.push(png_path);
            }
        }
        let smallest = report.min_text.map_or_else(|| "none".to_owned(), py_g);
        println!(
            "wrote {}: {} x {} px; smallest text {smallest} px; {} layout problem(s)",
            report.files.join(", "),
            py_g(report.width),
            py_g(report.height),
            report.problems.len()
        );
        if !report.problems.is_empty() && strict {
            return Err(SaveError::Layout(crate::error::LayoutError::new(
                report.problems.clone(),
                report.files.clone(),
            )));
        }
        Ok(report)
    }

    /// # Errors
    /// Returns an error when the diagram has no blocks or a color cannot be read.
    pub fn render(&mut self) -> Result<Report, DiagramError> {
        let own = self.theme.clone();
        self.theme = self.drawing_theme()?;
        let result = self.render_inner();
        self.theme = own;
        result
    }

    fn drawing_theme(&self) -> Result<Theme, DiagramError> {
        let accent = self.theme.tones.get("accent").cloned();
        let mut uses_accent = false;
        if let Some(accent) = &accent {
            for node in &self.nodes {
                if let Block::Panel(panel) = &node.block
                    && self.theme.tone(&panel.tone)? == *accent
                {
                    uses_accent = true;
                }
            }
            for group in &self.groups {
                if group.style == "panel" && self.theme.tone(&group.tone)? == *accent {
                    uses_accent = true;
                }
            }
        }
        let has_steps = !self.steps.is_empty()
            || self
                .nodes
                .iter()
                .any(|n| matches!(&n.block, Block::Panel(p) if p.step.is_some()));
        if uses_accent
            && has_steps
            && self.theme.badge_fill.is_some()
            && self.theme.badge_fill.as_deref() != self.theme.tones.get("dark").map(String::as_str)
        {
            let mut theme = self.theme.clone();
            theme.badge_fill = Some(
                self.theme
                    .tones
                    .get("dark")
                    .cloned()
                    .unwrap_or_else(|| "#2B3036".to_owned()),
            );
            theme.badge_text = Some(self.theme.header_text_on_dark.clone());
            return Ok(theme);
        }
        Ok(self.theme.clone())
    }

    fn render_inner(&mut self) -> Result<Report, DiagramError> {
        if self.nodes.is_empty() {
            return Err(DiagramError::new(
                "the diagram has no blocks; add some with add(), right_of(), below() or columns()",
            ));
        }
        self.measure_groups()?;
        self.route();
        let mut problems = self.check();
        let mut boxes: Vec<(f64, f64, f64, f64)> = self
            .nodes
            .iter()
            .map(|n| n.placed.rect())
            .chain(self.groups.iter().map(|g| g.rect))
            .collect();
        for (i, line) in self.links.iter().enumerate() {
            if line.label.is_some() {
                boxes.push(self.label_box(i, None));
            }
        }
        let pts: Vec<Point> = self
            .links
            .iter()
            .flat_map(|l| l.samples.iter().copied())
            .collect();
        let left = boxes
            .iter()
            .map(|b| b.0)
            .chain(pts.iter().map(|p| p.0))
            .fold(f64::INFINITY, f64::min);
        let north = boxes
            .iter()
            .map(|b| b.1)
            .chain(pts.iter().map(|p| p.1))
            .fold(f64::INFINITY, f64::min);
        let right = boxes
            .iter()
            .map(|b| b.0 + b.2)
            .chain(pts.iter().map(|p| p.0))
            .fold(f64::NEG_INFINITY, f64::max);
        let south = boxes
            .iter()
            .map(|b| b.1 + b.3)
            .chain(pts.iter().map(|p| p.1))
            .fold(f64::NEG_INFINITY, f64::max);
        let margin = self.theme.margin;
        let head = self.header_plan(right - left);
        let width = head.width;
        let top = margin + head.height + 32.0;
        let height = top + (south - north) + margin;
        let mut canvas = Canvas::new(width.ceil(), height.ceil());
        self.draw_header(&mut canvas, &head)?;
        let dx = if head.centre {
            margin - left + (width - mul(2.0, margin) - (right - left)) / 2.0
        } else {
            margin - left
        };
        self.draw_body(&mut canvas, &mut problems, dx, top - north)?;
        if let Some(min_text) = canvas.min_text
            && min_text < self.theme.min_text
        {
            problems.push(format!(
                "smallest text is {} px; the theme's minimum is {} px",
                py_g(min_text),
                py_g(self.theme.min_text)
            ));
        }
        let svg = canvas.svg(
            Some(&self.title),
            Some(&self.theme.paper),
            Some(&self.describe()),
        );
        Ok(Report {
            svg,
            width: canvas.width,
            height: canvas.height,
            min_text: canvas.min_text,
            problems,
            files: Vec::new(),
        })
    }

    fn draw_body(
        &self,
        canvas: &mut Canvas,
        problems: &mut Vec<String>,
        dx: f64,
        dy: f64,
    ) -> Result<(), DiagramError> {
        canvas.open_group(dx, dy);
        let order = self.group_draw_order();
        for id in &order {
            self.draw_group_body(canvas, *id)?;
        }
        self.draw_routed_links(canvas);
        for id in 0..self.groups.len() {
            self.draw_group_title(canvas, id)?;
        }
        self.draw_nodes(canvas, problems)?;
        self.draw_link_labels(canvas);
        canvas.close_group();
        Ok(())
    }

    fn draw_routed_links(&self, canvas: &mut Canvas) {
        let strokes: Vec<_> = self
            .links
            .iter()
            .map(|link| {
                (
                    link.kind == "planned",
                    link_dash(link),
                    link.d.clone(),
                    link.tip,
                    link.direction,
                )
            })
            .collect();
        for (dotted, dash, d, tip, direction) in strokes {
            canvas.path(
                &d,
                "none",
                Some(&self.theme.line),
                if dotted { 1.6 } else { 1.3 },
                dash.as_deref(),
                None,
            );
            canvas.path(
                &arrowhead(tip, direction),
                &self.theme.line,
                None,
                1.0,
                None,
                None,
            );
        }
    }

    fn draw_nodes(
        &self,
        canvas: &mut Canvas,
        problems: &mut Vec<String>,
    ) -> Result<(), DiagramError> {
        for index in 0..self.nodes.len() {
            let theme = self.theme.clone();
            let Some(node) = self.nodes.get(index) else {
                continue;
            };
            let placed = node.placed.clone();
            let mut sink = |msg: &str| problems.push(msg.to_owned());
            node.block.draw(
                canvas,
                (placed.x, placed.y, placed.w, placed.h),
                &theme,
                &mut sink,
            )?;
        }
        Ok(())
    }

    fn draw_link_labels(&self, canvas: &mut Canvas) {
        for index in 0..self.links.len() {
            let Some(label) = self.links.get(index).and_then(|link| link.label.clone()) else {
                continue;
            };
            let (x, y, w, h) = self.label_box(index, None);
            canvas.rect(
                x,
                y,
                w,
                h,
                Paint::new(
                    &self.theme.paper,
                    Some(&self.theme.line),
                    0.8,
                    h / 2.0,
                    None,
                    None,
                ),
            );
            canvas.text_centered(
                &label,
                x + w / 2.0,
                y + h / 2.0,
                self.theme.small,
                "medium",
                &self.theme.ink,
            );
        }
    }

    fn measure_groups(&mut self) -> Result<(), DiagramError> {
        for i in 0..self.groups.len() {
            let (pad, sub, style, label, tone, enclosure, members, children) = {
                let group = self
                    .groups
                    .get(i)
                    .ok_or_else(|| DiagramError::new(format!("missing group {i}")))?;
                (
                    group.pad,
                    group.sub.clone(),
                    group.style.clone(),
                    group.label.clone(),
                    group.tone.clone(),
                    group.enclosure,
                    group.members.clone(),
                    group.children.clone(),
                )
            };
            let mut bounds = Vec::new();
            for id in &members {
                let node = self
                    .nodes
                    .get(*id)
                    .ok_or_else(|| DiagramError::new(format!("missing node {id}")))?;
                bounds.push(node.placed.rect());
            }
            for child in &children {
                let child_group = self
                    .groups
                    .get(*child)
                    .ok_or_else(|| DiagramError::new(format!("missing group {child}")))?;
                bounds.push(child_group.rect);
            }
            let x0 = bounds.iter().map(|b| b.0).fold(f64::INFINITY, f64::min) - pad;
            let head_h = if style == "panel" {
                self.theme.lid
                    + if sub.is_some() {
                        self.theme.header_h_sub
                    } else {
                        self.theme.header_h
                    }
            } else {
                mul(self.theme.line_h, if sub.is_some() { 2.0 } else { 1.0 }) + 6.0
            };
            let y0 = bounds.iter().map(|b| b.1).fold(f64::INFINITY, f64::min) - pad - head_h;
            let mut x1 = bounds
                .iter()
                .map(|b| b.0 + b.2)
                .fold(f64::NEG_INFINITY, f64::max)
                + pad;
            let y1 = bounds
                .iter()
                .map(|b| b.1 + b.3)
                .fold(f64::NEG_INFINITY, f64::max)
                + pad;
            if style == "panel" {
                let (title_w, _) = Panel::new(label)
                    .sub_opt(sub)
                    .tone(tone)
                    .measure(&self.theme)?;
                x1 = x1.max(x0 + title_w);
            } else if enclosure {
                let title_w = text_width(&label, 12.5, "semibold").max(text_width(
                    sub.as_deref().unwrap_or(""),
                    self.theme.small,
                    "regular",
                ));
                x1 = x1.max(x0 + title_w + 28.0);
            }
            let group = self
                .groups
                .get_mut(i)
                .ok_or_else(|| DiagramError::new(format!("missing group {i}")))?;
            group.rect = (x0, y0, x1 - x0, y1 - y0);
        }
        Ok(())
    }

    fn route(&mut self) {
        for i in 0..self.links.len() {
            let (a_id, b_id, want_start, want_end) = {
                let Some(link) = self.links.get(i) else {
                    continue;
                };
                if link.want_start.is_empty() {
                    continue;
                }
                (
                    link.a,
                    link.b,
                    link.want_start.clone(),
                    link.want_end.clone(),
                )
            };
            let start = if want_start == "auto" {
                let (Some(start_node), Some(end_node)) =
                    (self.nodes.get(a_id), self.nodes.get(b_id))
                else {
                    continue;
                };
                facing(&start_node.placed, &end_node.placed)
            } else {
                want_start
            };
            let end = if want_end == "auto" {
                let (Some(end_node), Some(start_node)) =
                    (self.nodes.get(b_id), self.nodes.get(a_id))
                else {
                    continue;
                };
                facing(&end_node.placed, &start_node.placed)
            } else {
                want_end
            };
            if let Some(link) = self.links.get_mut(i) {
                link.start = start;
                link.end = end;
            }
        }
        let mut ends: IndexMap<(usize, String), Vec<(usize, usize)>> = IndexMap::new();
        for (i, line) in self.links.iter().enumerate() {
            ends.entry((line.a, line.start.clone()))
                .or_default()
                .push((i, 0));
            ends.entry((line.b, line.end.clone()))
                .or_default()
                .push((i, 1));
        }
        for ((owner, side), mut items) in ends {
            items.sort_by(|a, b| self.heading(*a, &side).total_cmp(&self.heading(*b, &side)));
            let (lo, hi) = {
                let Some(node) = self.nodes.get(owner) else {
                    continue;
                };
                node.block
                    .ports(&side, node.placed.w, node.placed.h, &self.theme)
            };
            let n = items.len();
            let step = if n > 1 {
                ((hi - lo) / count(n)).min(14.0)
            } else {
                0.0
            };
            let span = lo + hi;
            let mid = span / 2.0;
            for (i, (line, end)) in items.into_iter().enumerate() {
                let slot = count(i) - (count(n) - 1.0) / 2.0;
                let at = mid + mul(slot, step);
                if let Some(link) = self.links.get_mut(line) {
                    if let Some(offset) = link.offsets.get_mut(end) {
                        *offset = at;
                    }
                    if let Some(flag) = link.alone.get_mut(end) {
                        *flag = n == 1;
                    }
                }
            }
        }
        for i in 0..self.links.len() {
            if self.square(i) {
                self.straighten(i);
            }
        }
        for i in 0..self.links.len() {
            self.shape(i);
        }
        self.spread();
        self.place_labels();
    }

    fn heading(&self, item: (usize, usize), side: &str) -> f64 {
        let (other, via_x, via_y) = {
            let Some(line) = self.links.get(item.0) else {
                return 0.0;
            };
            let other = if item.1 == 0 { line.b } else { line.a };
            (other, line.via_x, line.via_y)
        };
        if matches!(side, "left" | "right") {
            via_y.unwrap_or_else(|| self.nodes.get(other).map_or(0.0, |node| node.placed.cy()))
        } else {
            via_x.unwrap_or_else(|| self.nodes.get(other).map_or(0.0, |node| node.placed.cx()))
        }
    }

    fn shape(&mut self, index: usize) {
        let Some((a_id, b_id, start, end, off0, off1, via_x, via_y)) =
            self.links.get(index).map(|link| {
                (
                    link.a,
                    link.b,
                    link.start.clone(),
                    link.end.clone(),
                    link.offsets.first().copied().unwrap_or(0.0),
                    link.offsets.get(1).copied().unwrap_or(0.0),
                    link.via_x,
                    link.via_y,
                )
            })
        else {
            return;
        };
        let p0 = self.anchor_at(a_id, &start, off0);
        let tip = self.anchor_at(b_id, &end, off1);
        let d3 = side_vector(&end).unwrap_or((1.0, 0.0));
        let p3 = (tip.0 + mul(d3.0, ARROW), tip.1 + mul(d3.1, ARROW));
        if let Some(link) = self.links.get_mut(index) {
            link.start_pt = p0;
            link.tip = tip;
            link.direction = (-d3.0, -d3.1);
        }
        let shape = LinkShape {
            index,
            a_id,
            b_id,
            start: &start,
            end: &end,
            p0,
            p3,
        };
        if self.square(index) {
            self.shape_elbow(&shape);
            return;
        }
        if via_x.is_none() && via_y.is_none() {
            self.shape_curve(&shape);
            return;
        }
        self.shape_lane(&shape);
    }

    fn shape_curve(&mut self, shape: &LinkShape<'_>) {
        let LinkShape {
            index,
            a_id,
            b_id,
            start,
            end,
            p0,
            p3,
        } = *shape;
        let along = if matches!(start, "left" | "right") {
            (p3.0 - p0.0).abs()
        } else {
            (p3.1 - p0.1).abs()
        };
        let k = 28.0_f64.max(along * 0.5);
        let (c1, c2) = {
            let (Some(start_node), Some(end_node)) = (self.nodes.get(a_id), self.nodes.get(b_id))
            else {
                return;
            };
            (
                outside(&start_node.placed, p0, start, k),
                outside(&end_node.placed, p3, end, k),
            )
        };
        let samples: Vec<Point> = (0..=80)
            .map(|i| bezier(p0, c1, c2, p3, f64::from(i) / 80.0))
            .collect();
        let label_spots: Vec<Point> = [0.5, 0.42, 0.58, 0.34, 0.66, 0.26, 0.74]
            .into_iter()
            .map(|s| bezier(p0, c1, c2, p3, s))
            .collect();
        let d = format!("M{}C{} {} {}", pt(p0), pt(c1), pt(c2), pt(p3));
        self.store_link_path(index, d, samples, label_spots);
    }

    fn shape_lane(&mut self, shape: &LinkShape<'_>) {
        let LinkShape {
            index,
            a_id,
            b_id,
            start,
            end,
            p0,
            p3,
        } = *shape;
        let (via_x, via_y) = {
            let Some(link) = self.links.get(index) else {
                return;
            };
            (link.via_x, link.via_y)
        };
        let (s0, s3) = {
            let (Some(start_node), Some(end_node)) = (self.nodes.get(a_id), self.nodes.get(b_id))
            else {
                return;
            };
            (
                outside(&start_node.placed, p0, start, STUB),
                outside(&end_node.placed, p3, end, STUB),
            )
        };
        let middle = via_y.map_or_else(
            || {
                let via_x = via_x.unwrap_or(0.0);
                vec![(via_x, s0.1), (via_x, s3.1)]
            },
            |at_y| vec![(s0.0, at_y), (s3.0, at_y)],
        );
        let mut pts = vec![p0, s0];
        pts.extend(middle);
        pts.push(s3);
        pts.push(p3);
        self.set_points(index, simplify(&pts), None);
    }

    fn store_link_path(&mut self, index: usize, d: String, samples: Vec<Point>, spots: Vec<Point>) {
        let Some(link) = self.links.get_mut(index) else {
            return;
        };
        link.d = d;
        link.samples = samples;
        link.label_spots = spots;
        link.points.clear();
        link.track = None;
        if let Some(&at) = link.label_spots.first() {
            link.label_at = at;
        }
    }

    fn set_points(&mut self, index: usize, pts: Vec<Point>, track: Option<(usize, f64, f64)>) {
        let d = rounded(&pts);
        let samples = sample(&pts, 5.0);
        let spots = line_label_spots(&pts);
        self.store_link_path(index, d, samples, spots);
        if let Some(link) = self.links.get_mut(index) {
            link.points = pts;
            link.track = track;
        }
    }

    fn anchor_at(&self, id: usize, side: &str, offset: f64) -> Point {
        let Some(node) = self.nodes.get(id) else {
            return (0.0, 0.0);
        };
        let (x, y) = node
            .block
            .anchor(side, offset, node.placed.w, node.placed.h, &self.theme);
        (node.placed.x + x, node.placed.y + y)
    }

    fn place_labels(&mut self) {
        let titles: Vec<_> = (0..self.groups.len())
            .map(|i| self.group_title_box(i))
            .collect();
        let placed: Vec<_> = self.nodes.iter().map(|n| n.placed.rect()).collect();
        let samples: Vec<Vec<Point>> = self.links.iter().map(|l| l.samples.clone()).collect();
        let mut taken = Vec::new();
        for i in 0..self.links.len() {
            let spots = {
                let Some(link) = self.links.get(i) else {
                    continue;
                };
                if link.label.is_none() {
                    continue;
                }
                link.label_spots.clone()
            };
            let mut chosen = match spots.as_slice() {
                [first, ..] => *first,
                [] => continue,
            };
            for spot in &spots {
                if label_clear(
                    self.label_box(i, Some(*spot)),
                    &placed,
                    &titles,
                    &taken,
                    &samples,
                    i,
                ) {
                    chosen = *spot;
                    break;
                }
            }
            if let Some(link) = self.links.get_mut(i) {
                link.label_at = chosen;
            }
            taken.push(self.label_box(i, Some(chosen)));
        }
    }

    fn label_box(&self, index: usize, at: Option<Point>) -> (f64, f64, f64, f64) {
        let small = self.theme.small;
        let Some(link) = self.links.get(index) else {
            return (0.0, 0.0, 0.0, 0.0);
        };
        let (x, y) = at.unwrap_or(link.label_at);
        let w = text_width(link.label.as_deref().unwrap_or(""), small, "medium") + 14.0;
        (x - w / 2.0, y - 9.0, w, 18.0)
    }

    fn group_title_box(&self, index: usize) -> (f64, f64, f64, f64) {
        let Some(group) = self.groups.get(index) else {
            return (0.0, 0.0, 0.0, 0.0);
        };
        let (x, y, w, _) = group.rect;
        if group.style == "panel" {
            let h = self.theme.lid
                + if group.sub.is_some() {
                    self.theme.header_h_sub
                } else {
                    self.theme.header_h
                };
            return (x, y, w, h);
        }
        let mut tw = text_width(&group.label, 12.5, "semibold");
        if let Some(sub) = &group.sub {
            tw = tw.max(text_width(sub, self.theme.small, "regular"));
        }
        (
            x + 10.0,
            y + 7.0,
            tw + 8.0,
            mul(
                self.theme.line_h,
                if group.sub.is_some() { 2.0 } else { 1.0 },
            ) + 8.0,
        )
    }

    fn check(&self) -> Vec<String> {
        let mut problems = self.early.clone();
        let min_gap = self.theme.min_gap;
        let nodes = &self.nodes;
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let (Some(node_a), Some(node_b)) = (nodes.get(i), nodes.get(j)) else {
                    continue;
                };
                let a = &node_a.placed;
                let b = &node_b.placed;
                let gap = separation(a.rect(), b.rect());
                if gap < 0.0 {
                    problems.push(format!(
                        "'{}' and '{}' overlap; move one (right_of() and below() keep a gap)",
                        a.name, b.name
                    ));
                } else if gap < min_gap {
                    problems.push(format!(
                        "'{}' and '{}' are only {} px apart (minimum {})",
                        a.name,
                        b.name,
                        py_fixed0(gap),
                        py_fixed0(min_gap)
                    ));
                }
            }
        }
        for g in &self.groups {
            for node in &self.nodes {
                if !g.members.contains(&node.placed.id)
                    && separation(g.rect, node.placed.rect()) < 0.0
                {
                    problems.push(format!(
                        "group '{}' overlaps '{}', which is not in it; add it to the group or move it",
                        g.label, node.placed.name
                    ));
                }
            }
        }
        let groups = &self.groups;
        for i in 0..groups.len() {
            for h in (i + 1)..groups.len() {
                let (Some(left), Some(right)) = (groups.get(i), groups.get(h)) else {
                    continue;
                };
                if separation(left.rect, right.rect) < 0.0
                    && !contains(left.rect, right.rect)
                    && !contains(right.rect, left.rect)
                {
                    problems.push(format!(
                        "groups '{}' and '{}' overlap",
                        left.label, right.label
                    ));
                }
            }
        }
        self.check_lines(&mut problems);
        problems
    }

    fn check_lines(&self, problems: &mut Vec<String>) {
        for line in &self.links {
            let (Some(from_name), Some(to_name)) = (
                self.nodes.get(line.a).map(|node| node.placed.name.clone()),
                self.nodes.get(line.b).map(|node| node.placed.name.clone()),
            ) else {
                continue;
            };
            let name = format!("line '{from_name}' -> '{to_name}'");
            if line.via_x.is_none() && line.via_y.is_none() {
                let d0 = side_vector(&line.start).unwrap_or((0.0, 0.0));
                let d3 = side_vector(&line.end).unwrap_or((0.0, 0.0));
                let vx = line.tip.0 - line.start_pt.0;
                let vy = line.tip.1 - line.start_pt.1;
                let fix = "use start='auto' and end='auto', other sides, or a lane (via_x/via_y)";
                if mul(d0.0, vx) + mul(d0.1, vy) < 1.0 {
                    problems.push(format!(
                        "{name} leaves '{from_name}' from its {} side, facing away from '{to_name}'; {fix}",
                        line.start
                    ));
                } else if mul(d3.0, -vx) + mul(d3.1, -vy) < 1.0 {
                    problems.push(format!(
                        "{name} enters '{to_name}' from its {} side, facing away from '{from_name}'; {fix}",
                        line.end
                    ));
                }
            }
            if !line.cross_ok {
                for node in &self.nodes {
                    if node.placed.id == line.a || node.placed.id == line.b {
                        continue;
                    }
                    let (x, y, w, h) = node.placed.rect();
                    if line.samples.iter().any(|s| {
                        x + 2.0 < s.0 && s.0 < x + w - 2.0 && y + 2.0 < s.1 && s.1 < y + h - 2.0
                    }) {
                        problems.push(format!("{name} crosses '{}'; route it with via_x/via_y, move a block, or pass cross_ok=True", node.placed.name));
                    }
                }
            }
            if line.label.is_some() {
                let box_ = self.label_box_of(line);
                for node in &self.nodes {
                    if separation(box_, node.placed.rect()) < 0.0 {
                        problems.push(format!(
                            "the label '{}' on {name} sits on '{}'",
                            line.label.as_deref().unwrap_or(""),
                            node.placed.name
                        ));
                    }
                }
                for (gi, g) in self.groups.iter().enumerate() {
                    if separation(box_, self.group_title_box(gi)) < 0.0 {
                        problems.push(format!(
                            "the label '{}' on {name} sits on the title of group '{}'; move the label's line, or the group",
                            line.label.as_deref().unwrap_or(""),
                            g.label
                        ));
                    }
                }
            }
        }
    }

    fn label_box_of(&self, line: &Link) -> (f64, f64, f64, f64) {
        let (x, y) = line.label_at;
        let w = text_width(
            line.label.as_deref().unwrap_or(""),
            self.theme.small,
            "medium",
        ) + 14.0;
        (x - w / 2.0, y - 9.0, w, 18.0)
    }

    fn group_draw_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.groups.len()).collect();
        if self.groups.iter().any(|g| g.enclosure) {
            let areas: Vec<f64> = self.groups.iter().map(|g| g.rect.2 * g.rect.3).collect();
            order.sort_by(|&a, &b| {
                let right = areas.get(b).copied().unwrap_or(0.0);
                let left = areas.get(a).copied().unwrap_or(0.0);
                right.total_cmp(&left)
            });
        }
        order
    }

    fn draw_group_body(&self, canvas: &mut Canvas, id: usize) -> Result<(), DiagramError> {
        let group = self
            .groups
            .get(id)
            .ok_or_else(|| DiagramError::new(format!("missing group {id}")))?;
        let (x, y, w, h) = group.rect;
        if group.style == "panel" {
            let tone = self.theme.tone(&group.tone)?;
            let fill = group.fill.as_deref().unwrap_or(&self.theme.panel_fill);
            let stroke = group.stroke.as_deref().unwrap_or(&self.theme.panel_stroke);
            let dash = (group.border != "solid").then_some(group.border.as_str());
            blocks::panel_frame(
                canvas,
                (x, y, w, h),
                blocks::PanelFace {
                    theme: &self.theme,
                    tone: &tone,
                    fill,
                    stroke,
                    width: 1.0,
                    dash,
                },
            );
        } else {
            let fill = group.fill.as_deref().unwrap_or(&self.theme.group_fill);
            let stroke = group.stroke.as_deref().unwrap_or(&self.theme.group_stroke);
            let dash = (group.border != "solid").then_some(group.border.as_str());
            canvas.rect(
                x,
                y,
                w,
                h,
                Paint::new(fill, Some(stroke), 1.1, self.theme.radius + 4.0, dash, None),
            );
        }
        Ok(())
    }

    fn draw_group_title(&self, canvas: &mut Canvas, id: usize) -> Result<(), DiagramError> {
        let (g_label, g_sub, g_style, g_tone, g_fill, x, y, w) = {
            let group = self
                .groups
                .get(id)
                .ok_or_else(|| DiagramError::new(format!("missing group {id}")))?;
            (
                group.label.clone(),
                group.sub.clone(),
                group.style.clone(),
                group.tone.clone(),
                group.fill.clone(),
                group.rect.0,
                group.rect.1,
                group.rect.2,
            )
        };
        if g_style == "panel" {
            let tone = self.theme.tone(&g_tone)?;
            let (title_color, sub_color) = self.theme.header_colors(&tone)?;
            let y = y + self.theme.lid;
            let hh = if g_sub.is_some() {
                self.theme.header_h_sub
            } else {
                self.theme.header_h
            };
            canvas.rect(x, y, w, hh, Paint::new(&tone, None, 1.0, 0.0, None, None));
            canvas.text_centered(
                &g_label,
                x + self.theme.pad + text_width(&g_label, self.theme.panel_title, "semibold") / 2.0,
                y + if g_sub.is_some() { 18.0 } else { hh / 2.0 },
                self.theme.panel_title,
                "semibold",
                &title_color,
            );
            if let Some(sub) = &g_sub {
                canvas.text(
                    sub,
                    x + self.theme.pad,
                    y + 36.0 + cap_height(self.theme.panel_sub, "regular") / 2.0,
                    Letter::new(self.theme.panel_sub, "regular", &sub_color, "start"),
                );
            }
            return Ok(());
        }
        let (bx, by, bw, bh) = self.group_title_box(id);
        let backing = if g_fill.as_deref() == Some("none") {
            self.theme.paper.clone()
        } else {
            g_fill.unwrap_or_else(|| self.theme.group_fill.clone())
        };
        canvas.rect(
            bx,
            by,
            bw,
            bh,
            Paint::new(&backing, None, 1.0, 3.0, None, None),
        );
        canvas.text(
            &g_label,
            x + 14.0,
            y + 12.0 + cap_height(12.5, "semibold"),
            Letter::new(12.5, "semibold", &self.theme.ink, "start"),
        );
        if let Some(sub) = &g_sub {
            canvas.text(
                sub,
                x + 14.0,
                y + 12.0 + self.theme.line_h + cap_height(self.theme.small, "regular"),
                Letter::new(self.theme.small, "regular", &self.theme.muted, "start"),
            );
        }
        Ok(())
    }

    fn header_plan(&self, content_w: f64) -> HeaderPlan {
        let t = &self.theme;
        let m = t.margin;
        let tag_w = self
            .tag
            .as_ref()
            .map_or(0.0, |tag| text_width(tag, 13.0, "bold") + 24.0);
        let title_w = tag_w
            + if tag_w == 0.0 { 0.0 } else { 12.0 }
            + text_width(&self.title, t.title, "bold");
        let sub_w = self
            .subtitle
            .as_ref()
            .map_or(0.0, |s| text_width(s, t.subtitle, "regular"));
        let (note_w, note_h) = self.note.as_ref().map_or((0.0, 0.0), |n| n.measure(t));
        let min_w = mul(2.0, m)
            + title_w.max(sub_w)
            + if self.note.is_some() {
                48.0 + note_w
            } else {
                0.0
            };
        let width = (content_w + mul(2.0, m)).max(min_w).max(640.0);
        let room = width
            - mul(2.0, m)
            - if self.note.is_some() {
                note_w + 40.0
            } else {
                0.0
            };
        let mut rows = Vec::new();
        if self.legend {
            let mut row = Vec::new();
            let mut used = 0.0;
            for item in self.legend_items() {
                let w = self.item_w(&item);
                if !row.is_empty() && used + w > room {
                    rows.push(row);
                    row = Vec::new();
                    used = 0.0;
                }
                used += w;
                row.push(item);
            }
            if !row.is_empty() {
                rows.push(row);
            }
        }
        let steps = !self.steps.is_empty() && self.legend;
        let h = 34.0
            + if self.subtitle.is_some() { 24.0 } else { 0.0 }
            + if rows.is_empty() && !steps { 0.0 } else { 14.0 }
            + mul(24.0, count(rows.len()))
            + if steps { 24.0 } else { 0.0 };
        HeaderPlan {
            width,
            height: h.max(note_h),
            rows,
            steps,
            tag_w,
            note_w,
            centre: content_w + mul(2.0, m) < width,
        }
    }

    fn legend_items(&self) -> Vec<LegendItem> {
        let mut used = BTreeSet::new();
        for node in &self.nodes {
            used.extend(node.block.statuses());
        }
        if used.contains("") && self.theme.statuses.contains_key("live") {
            used.insert("live".to_owned());
        }
        let mut items = Vec::new();
        for (key, status) in &self.theme.statuses {
            if used.contains(key) {
                items.push(LegendItem {
                    kind: "status".to_owned(),
                    key: key.clone(),
                    label: status.label.clone(),
                });
            }
        }
        for (kind, label) in &self.theme.link_labels {
            let mut styles = Vec::new();
            for line in &self.links {
                if &line.kind == kind {
                    let dash = link_dash(line);
                    if !styles.contains(&dash) {
                        styles.push(dash);
                    }
                }
            }
            for dash in &styles {
                let default = if kind == "planned" {
                    Some("dotted".to_owned())
                } else {
                    None
                };
                let style = dash.clone().unwrap_or_else(|| "solid".to_owned());
                let item_kind = if dash == &default {
                    "line".to_owned()
                } else {
                    format!("line-{style}")
                };
                let text = if styles.len() == 1 {
                    label.clone()
                } else {
                    format!("{label} ({style})")
                };
                items.push(LegendItem {
                    kind: item_kind,
                    key: kind.clone(),
                    label: text,
                });
            }
        }
        items
    }

    fn item_w(&self, item: &LegendItem) -> f64 {
        (if item.kind == "status" { 30.0 } else { 34.0 })
            + text_width(&item.label, self.theme.legend, "regular")
            + 22.0
    }

    fn draw_legend_rows(&self, canvas: &mut Canvas, rows: &[Vec<LegendItem>], mut y: f64) -> f64 {
        let t = &self.theme;
        let m = t.margin;
        for row in rows {
            let mut lx = m;
            let ly = y + 10.0;
            for item in row {
                if item.kind == "status" {
                    let Some(st) = t.statuses.get(&item.key) else {
                        continue;
                    };
                    canvas.rect(
                        lx,
                        ly - 7.0,
                        22.0,
                        14.0,
                        Paint::new(
                            &st.fill,
                            Some(&st.stroke),
                            1.2,
                            3.0,
                            st.dash.as_deref(),
                            None,
                        ),
                    );
                    lx += 30.0;
                } else {
                    let dotted = item.key == "planned";
                    let dash = if item.kind == "line" {
                        if dotted {
                            Some("dotted".to_owned())
                        } else {
                            None
                        }
                    } else {
                        let Some(style) = item.kind.strip_prefix("line-") else {
                            continue;
                        };
                        Some(style.to_owned())
                    };
                    let dash = dash.filter(|d| d != "solid");
                    canvas.line(
                        (lx, ly),
                        (lx + 22.0, ly),
                        &t.line,
                        if dotted { 1.6 } else { 1.3 },
                        dash.as_deref(),
                    );
                    canvas.path(
                        &arrowhead((lx + 28.0, ly), (1.0, 0.0)),
                        &t.line,
                        None,
                        1.0,
                        None,
                        None,
                    );
                    lx += 34.0;
                }
                lx += canvas.text(
                    &item.label,
                    lx,
                    ly + cap_height(t.legend, "regular") / 2.0,
                    Letter::new(t.legend, "regular", &t.ink, "start"),
                ) + 22.0;
            }
            y += 24.0;
        }
        y
    }

    fn draw_header(&self, canvas: &mut Canvas, plan: &HeaderPlan) -> Result<(), DiagramError> {
        let t = &self.theme;
        let m = t.margin;
        let mut x = m;
        let cy = m + 15.0;
        if let Some(tag) = &self.tag {
            let w = plan.tag_w;
            canvas.rect(
                x,
                cy - 14.0,
                w,
                28.0,
                Paint::new(&t.paper, Some(&t.tag_color), 1.4, 14.0, None, None),
            );
            canvas.text_centered(tag, x + w / 2.0, cy, 13.0, "bold", &t.tag_color);
            x += w + 12.0;
        }
        canvas.text(
            &self.title,
            x,
            cy + cap_height(t.title, "bold") / 2.0,
            Letter::new(t.title, "bold", &t.ink, "start"),
        );
        let mut y = m + 34.0;
        if let Some(subtitle) = &self.subtitle {
            canvas.text(
                subtitle,
                m,
                y + 10.0 + cap_height(t.subtitle, "regular") / 2.0,
                Letter::new(t.subtitle, "regular", &t.muted, "start"),
            );
            y += 24.0;
        }
        if !plan.rows.is_empty() || plan.steps {
            y += 14.0;
        }
        y = self.draw_legend_rows(canvas, &plan.rows, y);
        if plan.steps {
            let mut lx = m;
            let ly = y + 10.0;
            lx += canvas.text(
                &format!("{}:", self.steps_label),
                lx,
                ly + cap_height(t.legend, "semibold") / 2.0,
                Letter::new(t.legend, "semibold", &t.ink, "start"),
            ) + 12.0;
            let tone = t.tone("default")?;
            for (n, label) in &self.steps {
                canvas.circle(
                    lx + 9.0,
                    ly,
                    9.0,
                    t.badge_fill.as_deref().unwrap_or(&tone),
                    None,
                    1.0,
                );
                canvas.text_centered(
                    &n.to_string(),
                    lx + 9.0,
                    ly,
                    10.5,
                    "bold",
                    t.badge_text.as_deref().unwrap_or(&t.paper),
                );
                lx +=
                    24.0 + canvas.text(
                        label,
                        lx + 24.0,
                        ly + cap_height(t.legend, "regular") / 2.0,
                        Letter::new(t.legend, "regular", &t.ink, "start"),
                    ) + 18.0;
            }
        }
        if let Some(note) = &self.note {
            let (_, nh) = note.measure(t);
            note.draw(
                canvas,
                (plan.width - m - plan.note_w, m, plan.note_w, nh),
                t,
            );
        }
        Ok(())
    }

    fn square(&self, index: usize) -> bool {
        let Some(line) = self.links.get(index) else {
            return false;
        };
        let chosen = line.routing.as_deref().unwrap_or(self.routing.as_str());
        line.via_x.is_none() && line.via_y.is_none() && chosen == "orthogonal"
    }

    fn straighten(&mut self, index: usize) {
        let Some(line) = self.links.get(index) else {
            return;
        };
        let free_a = line.alone.first().copied().unwrap_or(false);
        let free_b = line.alone.get(1).copied().unwrap_or(false);
        if !free_a && !free_b {
            return;
        }
        let start = line.start.clone();
        let end = line.end.clone();
        let facing = matches!(
            (start.as_str(), end.as_str()),
            ("left", "right") | ("right", "left") | ("top", "bottom") | ("bottom", "top")
        );
        if !facing {
            return;
        }
        let horizontal = matches!(start.as_str(), "left" | "right");
        let forward = match start.as_str() {
            "right" | "bottom" => 1.0,
            _ => -1.0,
        };
        let a_id = line.a;
        let b_id = line.b;
        let off0 = line.offsets.first().copied().unwrap_or(0.0);
        let off1 = line.offsets.get(1).copied().unwrap_or(0.0);
        let Some(node_a) = self.nodes.get(a_id) else {
            return;
        };
        let Some(node_b) = self.nodes.get(b_id) else {
            return;
        };
        let gap = side_edge(&node_b.placed, &end) - side_edge(&node_a.placed, &start);
        if mul(gap, forward) <= 0.0 {
            return;
        }
        let (base_a, base_b) = if horizontal {
            (node_a.placed.y, node_b.placed.y)
        } else {
            (node_a.placed.x, node_b.placed.x)
        };
        let (lo_a, hi_a) = if free_a {
            node_a
                .block
                .ports(&start, node_a.placed.w, node_a.placed.h, &self.theme)
        } else {
            (off0, off0)
        };
        let (lo_b, hi_b) = if free_b {
            node_b
                .block
                .ports(&end, node_b.placed.w, node_b.placed.h, &self.theme)
        } else {
            (off1, off1)
        };
        let lo = (base_a + lo_a).max(base_b + lo_b);
        let hi = (base_a + hi_a).min(base_b + hi_b);
        if lo > hi {
            return;
        }
        let at_sum = (base_a + off0) + (base_b + off1);
        let level = (at_sum / 2.0).max(lo).min(hi);
        let Some(routed) = self.links.get_mut(index) else {
            return;
        };
        if let Some(slot) = routed.offsets.first_mut() {
            *slot = level - base_a;
        }
        if let Some(slot) = routed.offsets.get_mut(1) {
            *slot = level - base_b;
        }
    }

    fn shape_elbow(&mut self, shape: &LinkShape<'_>) {
        let (pts, track) = {
            let (Some(from), Some(to)) = (self.nodes.get(shape.a_id), self.nodes.get(shape.b_id))
            else {
                return;
            };
            elbow(
                shape.start,
                shape.end,
                &from.placed,
                &to.placed,
                shape.p0,
                shape.p3,
            )
        };
        self.set_points(shape.index, pts, track);
    }

    fn spread(&mut self) {
        for axis in [0_usize, 1] {
            let mut runs: Vec<usize> = (0..self.links.len())
                .filter(|&index| self.tracked(index, axis))
                .collect();
            runs.sort_by(|&left, &right| {
                self.middle(left, axis).total_cmp(&self.middle(right, axis))
            });
            let mut clusters: Vec<Vec<usize>> = Vec::new();
            for index in runs {
                let near = clusters
                    .last()
                    .and_then(|cluster| cluster.last())
                    .is_some_and(|&prev| {
                        self.middle(index, axis) - self.middle(prev, axis) < TRACK
                    });
                if near {
                    if let Some(cluster) = clusters.last_mut() {
                        cluster.push(index);
                    }
                } else {
                    clusters.push(vec![index]);
                }
            }
            for cluster in clusters {
                self.separate_spans(&cluster, axis);
            }
        }
    }

    fn tracked(&self, index: usize, axis: usize) -> bool {
        self.links.get(index).is_some_and(|line| {
            line.track.is_some_and(|track| track.0 == axis) && line.points.len() >= 3
        })
    }

    fn middle(&self, index: usize, axis: usize) -> f64 {
        self.links
            .get(index)
            .and_then(|line| line.points.get(1))
            .map_or(0.0, |point| component(*point, axis))
    }

    fn separate_spans(&mut self, cluster: &[usize], axis: usize) {
        let across = 1 - axis;
        let mut spans = cluster.to_vec();
        spans.sort_by(|&left, &right| {
            self.span_low(left, across)
                .total_cmp(&self.span_low(right, across))
        });
        let mut parts: Vec<Vec<usize>> = Vec::new();
        let mut reach = f64::NEG_INFINITY;
        for index in spans {
            let (lo, hi) = self.span_ends(index, across);
            let overlaps = parts.last().is_some() && lo < reach + 2.0;
            if overlaps {
                if let Some(part) = parts.last_mut() {
                    part.push(index);
                }
                reach = reach.max(hi);
            } else {
                parts.push(vec![index]);
                reach = hi;
            }
        }
        for part in parts {
            if part.len() > 1 {
                self.assign_tracks(&part, axis);
            }
        }
    }

    fn span_low(&self, index: usize, axis: usize) -> f64 {
        self.span_ends(index, axis).0
    }

    fn span_ends(&self, index: usize, axis: usize) -> (f64, f64) {
        let Some(line) = self.links.get(index) else {
            return (0.0, 0.0);
        };
        let Some(first) = line.points.get(1).copied() else {
            return (0.0, 0.0);
        };
        let Some(second) = line.points.get(2).copied() else {
            return (0.0, 0.0);
        };
        sorted_pair(component(first, axis), component(second, axis))
    }

    fn assign_tracks(&mut self, part: &[usize], axis: usize) {
        let across = 1 - axis;
        let mut rows = Vec::new();
        for &index in part {
            let Some(row) = self.track_row(index, axis, across) else {
                continue;
            };
            rows.push(row);
        }
        let n = rows.len();
        if n < 2 {
            return;
        }
        let travel = rows.first().map_or(1.0, |row| row.travel);
        let spots = track_spots(&rows, travel);
        rows.sort_by(fan_order);
        let Some(best) = least_crossings(&rows, &spots, axis) else {
            return;
        };
        for (local, pts) in best {
            let Some(row) = rows.get(local) else {
                continue;
            };
            let index = row.index;
            let track = row.track;
            self.set_points(index, simplify(&pts), Some(track));
        }
    }

    fn track_row(&self, index: usize, axis: usize, across: usize) -> Option<TrackRow> {
        let line = self.links.get(index)?;
        let track = line.track?;
        let mid = line.points.get(1).copied()?;
        let next = line.points.get(2).copied()?;
        let start_pt = line.points.first().copied()?;
        Some(TrackRow {
            index,
            points: line.points.clone(),
            low: track.1,
            high: track.2,
            middle: component(mid, axis),
            across_delta: component(next, across) - component(mid, across),
            start_across: component(start_pt, across),
            travel: travel_component(&line.start, axis),
            track,
        })
    }
}

impl Panel {
    #[must_use]
    pub fn sub_opt(mut self, sub: Option<String>) -> Self {
        self.sub = sub;
        self
    }
}

fn side_vector(side: &str) -> Option<Point> {
    match side {
        "left" => Some((-1.0, 0.0)),
        "right" => Some((1.0, 0.0)),
        "top" => Some((0.0, -1.0)),
        "bottom" => Some((0.0, 1.0)),
        _ => None,
    }
}

fn facing(a: &Placed, b: &Placed) -> String {
    let gx = (b.x - a.right()).max(a.x - b.right());
    let gy = (b.y - a.bottom()).max(a.y - b.bottom());
    if gx >= gy {
        if b.cx() >= a.cx() {
            "right".to_owned()
        } else {
            "left".to_owned()
        }
    } else if b.cy() >= a.cy() {
        "bottom".to_owned()
    } else {
        "top".to_owned()
    }
}

fn link_dash(line: &Link) -> Option<String> {
    match line.dash.as_deref() {
        None => (line.kind == "planned").then(|| "dotted".to_owned()),
        Some("solid") => None,
        Some(other) => Some(other.to_owned()),
    }
}

fn outside(p: &Placed, point: Point, side: &str, minimum: f64) -> Point {
    let (dx, dy) = side_vector(side).unwrap_or((0.0, 0.0));
    let edge = match side {
        "left" => p.x,
        "right" => p.right(),
        "top" => p.y,
        _ => p.bottom(),
    };
    let inset = if dx == 0.0 {
        (edge - point.1) * dy
    } else {
        (edge - point.0) * dx
    };
    let distance = minimum.max(inset + STUB);
    (point.0 + mul(dx, distance), point.1 + mul(dy, distance))
}

fn pt(p: Point) -> String {
    format!("{} {}", fmt(p.0), fmt(p.1))
}

fn bezier_axis(u: f64, s: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let cubic = mul(u.powi(3), p0);
    let u_then_u = mul(mul(3.0, u), u);
    let first = mul(mul(u_then_u, s), p1);
    let s_then_s = mul(mul(3.0, u), s);
    let second = mul(mul(s_then_s, s), p2);
    let last = mul(s.powi(3), p3);
    cubic + first + second + last
}

fn bezier(p0: Point, p1: Point, p2: Point, p3: Point, s: f64) -> Point {
    let u = 1.0 - s;
    (
        bezier_axis(u, s, p0.0, p1.0, p2.0, p3.0),
        bezier_axis(u, s, p0.1, p1.1, p2.1, p3.1),
    )
}

fn dist(a: Point, b: Point) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

fn segment_len(pts: &[Point], index: usize) -> f64 {
    let Some(next) = index.checked_add(1) else {
        return 0.0;
    };
    let (Some(start), Some(end)) = (pts.get(index), pts.get(next)) else {
        return 0.0;
    };
    dist(*start, *end)
}

fn simplify(pts: &[Point]) -> Vec<Point> {
    let mut out = Vec::new();
    for pt in pts {
        if let Some(last) = out.last()
            && dist(*last, *pt) < 0.01
        {
            continue;
        }
        if out.len() >= 2 {
            let (a, b) = {
                let Some((&b, earlier)) = out.split_last() else {
                    continue;
                };
                let Some(&a) = earlier.last() else {
                    continue;
                };
                (a, b)
            };
            let cross = mul(b.0 - a.0, pt.1 - b.1) - mul(b.1 - a.1, pt.0 - b.0);
            let dot = mul(b.0 - a.0, pt.0 - b.0) + mul(b.1 - a.1, pt.1 - b.1);
            if cross.abs() < 0.01 && dot >= 0.0 {
                if let Some(last) = out.last_mut() {
                    *last = *pt;
                }
                continue;
            }
        }
        out.push(*pt);
    }
    out
}

fn rounded(pts: &[Point]) -> String {
    let Some(&start) = pts.first() else {
        return String::new();
    };
    let mut path = vec![format!("M{}", pt(start))];
    for window in pts.windows(3) {
        let (prev, corner, next) = match window {
            [prev, corner, next] => (*prev, *corner, *next),
            [_, ..] | [] => continue,
        };
        let radius = CORNER
            .min(dist(prev, corner) / 2.0)
            .min(dist(corner, next) / 2.0);
        let incoming = (
            (corner.0 - prev.0) / dist(prev, corner),
            (corner.1 - prev.1) / dist(prev, corner),
        );
        let outgoing = (
            (next.0 - corner.0) / dist(corner, next),
            (next.1 - corner.1) / dist(corner, next),
        );
        path.push(format!(
            "L{}Q{} {}",
            pt((
                corner.0 - mul(incoming.0, radius),
                corner.1 - mul(incoming.1, radius)
            )),
            pt(corner),
            pt((
                corner.0 + mul(outgoing.0, radius),
                corner.1 + mul(outgoing.1, radius)
            ))
        ));
    }
    if let Some(last) = pts.last() {
        path.push(format!("L{}", pt(*last)));
    }
    path.join("")
}

fn sample(pts: &[Point], step: f64) -> Vec<Point> {
    let mut out = Vec::new();
    for pair in pts.windows(2) {
        let (a, b) = match pair {
            [a, b] => (*a, *b),
            [_, ..] | [] => continue,
        };
        let n = trunc(dist(a, b) / step).max(1);
        for i in 0..n {
            let t = whole(i) / whole(n);
            out.push((a.0 + mul(b.0 - a.0, t), a.1 + mul(b.1 - a.1, t)));
        }
    }
    if let Some(last) = pts.last() {
        out.push(*last);
    }
    out
}

fn arrowhead(tip: Point, direction: Point) -> String {
    let (dx, dy) = direction;
    let (bx, by) = (tip.0 - mul(dx, ARROW), tip.1 - mul(dy, ARROW));
    let (px, py) = (-dy * 3.6, dx * 3.6);
    format!(
        "M{}L{}L{}Z",
        pt(tip),
        pt((bx + px, by + py)),
        pt((bx - px, by - py))
    )
}

fn separation(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> f64 {
    let dx = (b.0 - (a.0 + a.2)).max(a.0 - (b.0 + b.2));
    let dy = (b.1 - (a.1 + a.3)).max(a.1 - (b.1 + b.3));
    dx.max(dy)
}

fn contains(outer: (f64, f64, f64, f64), inner: (f64, f64, f64, f64)) -> bool {
    outer.0 <= inner.0
        && outer.1 <= inner.1
        && outer.0 + outer.2 >= inner.0 + inner.2
        && outer.1 + outer.3 >= inner.1 + inner.3
}

fn label_clear(
    box_: (f64, f64, f64, f64),
    placed: &[(f64, f64, f64, f64)],
    titles: &[(f64, f64, f64, f64)],
    taken: &[(f64, f64, f64, f64)],
    samples: &[Vec<Point>],
    index: usize,
) -> bool {
    if placed.iter().any(|p| separation(box_, *p) < 0.0) {
        return false;
    }
    if titles
        .iter()
        .chain(taken.iter())
        .any(|b| separation(box_, *b) < 0.0)
    {
        return false;
    }
    samples
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .flat_map(|(_, s)| s)
        .all(|s| {
            !(box_.0 - 2.0 < s.0
                && s.0 < box_.0 + box_.2 + 2.0
                && box_.1 - 2.0 < s.1
                && s.1 < box_.1 + box_.3 + 2.0)
        })
}

struct TrackRow {
    index: usize,
    points: Vec<Point>,
    low: f64,
    high: f64,
    middle: f64,
    across_delta: f64,
    start_across: f64,
    travel: f64,
    track: (usize, f64, f64),
}

const fn component(point: Point, axis: usize) -> f64 {
    if axis == 0 { point.0 } else { point.1 }
}

const fn placed_on(axis: usize, along: f64, across: f64) -> Point {
    if axis == 0 {
        (along, across)
    } else {
        (across, along)
    }
}

fn travel_component(side: &str, axis: usize) -> f64 {
    if matches!((side, axis), ("left", 0) | ("top", 1)) {
        -1.0
    } else {
        1.0
    }
}

fn side_edge(placed: &Placed, side: &str) -> f64 {
    match side {
        "left" => placed.x,
        "right" => placed.right(),
        "top" => placed.y,
        _ => placed.bottom(),
    }
}

const fn sorted_pair(left: f64, right: f64) -> (f64, f64) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn line_label_spots(pts: &[Point]) -> Vec<Point> {
    let mut order: Vec<usize> = (0..pts.len().saturating_sub(1)).collect();
    order.sort_by(|&left, &right| segment_len(pts, right).total_cmp(&segment_len(pts, left)));
    let mut spots = Vec::new();
    for index in order.into_iter().take(2) {
        let Some(next) = index.checked_add(1) else {
            continue;
        };
        let Some((&start, &end)) = pts.get(index).zip(pts.get(next)) else {
            continue;
        };
        for factor in [0.5, 0.35, 0.65, 0.2, 0.8] {
            spots.push((
                start.0 + mul(end.0 - start.0, factor),
                start.1 + mul(end.1 - start.1, factor),
            ));
        }
    }
    spots
}

// Upstream bd276519b62301fcf76d91aacab95c30eff4a925. Orthogonal routing is opt-in;
// a via lane already turns on its own path. The track is the only run that may slide,
// and only between the two stubs, so lines that share a gap can separate without moving an end another line uses.
fn elbow(
    start: &str,
    end: &str,
    from: &Placed,
    to: &Placed,
    p0: Point,
    p3: Point,
) -> (Vec<Point>, Option<(usize, f64, f64)>) {
    let axis = usize::from(!matches!(start, "left" | "right"));
    let s0 = outside(from, p0, start, STUB);
    let s3 = outside(to, p3, end, STUB);
    let same_axis = matches!(start, "left" | "right") == matches!(end, "left" | "right");
    if same_axis {
        return elbow_parallel(start, end, axis, p0, p3, s0, s3);
    }
    elbow_corner(start, end, axis, p0, p3, s0, s3)
}

fn elbow_parallel(
    start: &str,
    end: &str,
    axis: usize,
    p0: Point,
    p3: Point,
    s0: Point,
    s3: Point,
) -> (Vec<Point>, Option<(usize, f64, f64)>) {
    let across = 1 - axis;
    let dir = travel_component(start, axis);
    if start == end {
        let far_key = mul(component(s0, axis), dir).max(mul(component(s3, axis), dir));
        let far = mul(far_key, dir);
        let pts = simplify(&[
            p0,
            placed_on(axis, far, component(p0, across)),
            placed_on(axis, far, component(p3, across)),
            p3,
        ]);
        return (pts, None);
    }
    let ahead = mul(component(p3, axis) - component(p0, axis), dir);
    if ahead > 0.0 {
        return elbow_between(axis, across, dir, p0, p3, s0, s3);
    }
    let sum = component(p0, across) + component(p3, across);
    let mid = sum / 2.0;
    let pts = simplify(&[
        p0,
        s0,
        placed_on(axis, component(s0, axis), mid),
        placed_on(axis, component(s3, axis), mid),
        s3,
        p3,
    ]);
    (pts, None)
}

fn elbow_between(
    axis: usize,
    across: usize,
    dir: f64,
    p0: Point,
    p3: Point,
    s0: Point,
    s3: Point,
) -> (Vec<Point>, Option<(usize, f64, f64)>) {
    if (component(p0, across) - component(p3, across)).abs() < 0.5 {
        return (vec![p0, p3], None);
    }
    let (lo, hi) = if mul(component(s3, axis) - component(s0, axis), dir) >= 0.0 {
        sorted_pair(component(s0, axis), component(s3, axis))
    } else {
        let sum = component(p0, axis) + component(p3, axis);
        let mid = sum / 2.0;
        (mid, mid)
    };
    let sum = lo + hi;
    let mid = sum / 2.0;
    let pts = simplify(&[
        p0,
        placed_on(axis, mid, component(p0, across)),
        placed_on(axis, mid, component(p3, across)),
        p3,
    ]);
    let track = (pts.len() == 4).then_some((axis, lo, hi));
    (pts, track)
}

fn elbow_corner(
    start: &str,
    end: &str,
    axis: usize,
    p0: Point,
    p3: Point,
    s0: Point,
    s3: Point,
) -> (Vec<Point>, Option<(usize, f64, f64)>) {
    let across = 1 - axis;
    let corner = placed_on(axis, component(p3, axis), component(p0, across));
    let leaves = mul(
        component(corner, axis) - component(s0, axis),
        travel_component(start, axis),
    ) >= 0.0;
    let arrives = mul(
        component(corner, across) - component(s3, across),
        travel_component(end, across),
    ) >= 0.0;
    if leaves && arrives {
        return (vec![p0, corner, p3], None);
    }
    let pts = simplify(&[
        p0,
        s0,
        placed_on(axis, component(s0, axis), component(s3, across)),
        s3,
        p3,
    ]);
    (pts, None)
}

fn track_spots(rows: &[TrackRow], travel: f64) -> Vec<f64> {
    let n = rows.len();
    let mut low = rows
        .iter()
        .map(|row| row.low)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut high = rows
        .iter()
        .map(|row| row.high)
        .fold(f64::INFINITY, f64::min);
    let sum: f64 = rows.iter().map(|row| row.middle).sum();
    let mut centre = sum / count(n);
    if low > high {
        low = centre;
        high = centre;
    }
    let gaps = n.saturating_sub(1).max(1);
    let step_fit = (high - low) / count(gaps);
    let step = TRACK.min(step_fit).max(4.0);
    let half = mul(step, count(n.saturating_sub(1))) / 2.0;
    let room = high - low;
    centre = if room >= mul(2.0, half) {
        (centre.max(low + half)).min(high - half)
    } else {
        let span = low + high;
        span / 2.0
    };
    let mut spots: Vec<f64> = (0..n)
        .map(|i| {
            let slot = count(i) - (count(n) - 1.0) / 2.0;
            centre + mul(slot, step)
        })
        .collect();
    spots.sort_by(|left, right| mul(*left, travel).total_cmp(&mul(*right, travel)));
    spots
}

fn fan_order(left: &TrackRow, right: &TrackRow) -> std::cmp::Ordering {
    let left_key = track_order(left.across_delta, left.start_across);
    let right_key = track_order(right.across_delta, right.start_across);
    left_key
        .0
        .cmp(&right_key.0)
        .then_with(|| left_key.1.total_cmp(&right_key.1))
}

const fn track_order(delta: f64, start_across: f64) -> (u8, f64) {
    if delta < 0.0 {
        (0, start_across)
    } else {
        (1, -start_across)
    }
}

type TrackPaths = Vec<(usize, Vec<Point>)>;

fn least_crossings(rows: &[TrackRow], spots: &[f64], axis: usize) -> Option<TrackPaths> {
    let n = rows.len();
    if n > 6 {
        return fewer_crossings(rows, spots, axis);
    }
    let mut best: Option<(i64, TrackPaths)> = None;
    for order in lex_orders(n) {
        let Some((cost, paths)) = order_cost(rows, spots, axis, &order) else {
            continue;
        };
        if best.as_ref().is_none_or(|(low, _)| cost < *low) {
            best = Some((cost, order.into_iter().zip(paths).collect()));
        }
    }
    best.map(|(_, placed)| placed)
}

// Too many lines to try every order: start from the order that suits a fan of lines,
// then swap any two lines while that removes crossings.
fn fewer_crossings(rows: &[TrackRow], spots: &[f64], axis: usize) -> Option<TrackPaths> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&left, &right| match (rows.get(left), rows.get(right)) {
        (Some(left), Some(right)) => fan_order(left, right),
        _ => std::cmp::Ordering::Equal,
    });
    let (mut cost, mut paths) = order_cost(rows, spots, axis, &order)?;
    let mut improved = true;
    while improved && cost > 0 {
        improved = false;
        for left in 0..order.len() {
            for right in (left + 1)..order.len() {
                order.swap(left, right);
                match order_cost(rows, spots, axis, &order) {
                    Some((next, next_paths)) if next < cost => {
                        cost = next;
                        paths = next_paths;
                        improved = true;
                    }
                    _ => order.swap(left, right),
                }
            }
        }
    }
    Some(order.into_iter().zip(paths).collect())
}

fn order_cost(
    rows: &[TrackRow],
    spots: &[f64],
    axis: usize,
    order: &[usize],
) -> Option<(i64, Vec<Vec<Point>>)> {
    let mut paths = Vec::new();
    for (place, local) in order.iter().enumerate() {
        let row = rows.get(*local)?;
        let spot = spots.get(place).copied()?;
        // A track slides only between its own two stubs.
        paths.push(with_track(&row.points, axis, spot.clamp(row.low, row.high)));
    }
    let mut cost = 0_i64;
    for (left, path_left) in paths.iter().enumerate() {
        for path_right in paths.iter().skip(left + 1) {
            cost += crossings(path_left, path_right);
        }
    }
    Some((cost, paths))
}

fn lex_orders(n: usize) -> Vec<Vec<usize>> {
    fn walk(prefix: &[usize], rest: &[usize], out: &mut Vec<Vec<usize>>) {
        if rest.is_empty() {
            out.push(prefix.to_vec());
            return;
        }
        for (index, &head) in rest.iter().enumerate() {
            let mut next = prefix.to_vec();
            next.push(head);
            let mut remain = Vec::new();
            remain.extend(rest.iter().take(index).copied());
            remain.extend(rest.iter().skip(index.saturating_add(1)).copied());
            walk(&next, &remain, out);
        }
    }
    let mut out = Vec::new();
    walk(&[], &(0..n).collect::<Vec<_>>(), &mut out);
    out
}

fn with_track(pts: &[Point], axis: usize, spot: f64) -> Vec<Point> {
    pts.iter()
        .enumerate()
        .map(|(index, point)| {
            if index == 1 || index == 2 {
                if axis == 0 {
                    (spot, point.1)
                } else {
                    (point.0, spot)
                }
            } else {
                *point
            }
        })
        .collect()
}

fn crossings(left: &[Point], right: &[Point]) -> i64 {
    let mut total = 0_i64;
    for left_pair in left.windows(2) {
        for right_pair in right.windows(2) {
            let (Some(&a), Some(&b)) = (left_pair.first(), left_pair.get(1)) else {
                continue;
            };
            let (Some(&c), Some(&d)) = (right_pair.first(), right_pair.get(1)) else {
                continue;
            };
            total += meets(a, b, c, d);
        }
    }
    total
}

fn meets(from_left: Point, to_left: Point, from_right: Point, to_right: Point) -> i64 {
    let flat_left = (from_left.1 - to_left.1).abs() < 0.01;
    let flat_right = (from_right.1 - to_right.1).abs() < 0.01;
    if flat_left == flat_right {
        let axis = usize::from(!flat_left);
        let other = 1 - axis;
        if (component(from_left, other) - component(from_right, other)).abs() > 0.5 {
            return 0;
        }
        let lo = component(from_left, axis)
            .min(component(to_left, axis))
            .max(component(from_right, axis).min(component(to_right, axis)));
        let hi = component(from_left, axis)
            .max(component(to_left, axis))
            .min(component(from_right, axis).max(component(to_right, axis)));
        return if hi - lo > 1.0 { 2 } else { 0 };
    }
    let (from_left, to_left, from_right, to_right) = if flat_left {
        (from_left, to_left, from_right, to_right)
    } else {
        (from_right, to_right, from_left, to_left)
    };
    let cross_x = from_right.0;
    let cross_y = from_left.1;
    let inside_x =
        from_left.0.min(to_left.0) + 0.5 < cross_x && cross_x < from_left.0.max(to_left.0) - 0.5;
    let inside_y = from_right.1.min(to_right.1) + 0.5 < cross_y
        && cross_y < from_right.1.max(to_right.1) - 0.5;
    i64::from(inside_x && inside_y)
}

#[cfg(test)]
#[path = "behavior.rs"]
mod behavior;
