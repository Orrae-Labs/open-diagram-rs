use std::collections::BTreeSet;

use indexmap::IndexMap;

use crate::blocks::{self, apply_statuses, Block, Note, Panel};
use crate::describe;
use crate::error::{DiagramError, SaveError};
use crate::font::{cap_height, text_width};
use crate::num::{count, fmt, mul, py_fixed0, py_g, py_repr, trunc, whole};
use crate::svg::{Canvas, Letter, Paint};
use crate::theme::Theme;

const ARROW: f64 = 7.0;
const STUB: f64 = 16.0;
const CORNER: f64 = 10.0;

pub type Point = (f64, f64);

fn strip_picture_suffix(path: &str) -> &str {
    let Some(tail) = path.get(path.len().saturating_sub(4)..) else {
        return path;
    };
    if tail.eq_ignore_ascii_case(".svg") || tail.eq_ignore_ascii_case(".png") { &path[..path.len() - 4] } else { path }
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
    pub fn right(&self) -> f64 { self.x + self.w }
    #[must_use]
    pub fn bottom(&self) -> f64 { self.y + self.h }
    #[must_use]
    pub fn cx(&self) -> f64 { self.x + self.w / 2.0 }
    #[must_use]
    pub fn cy(&self) -> f64 { self.y + self.h / 2.0 }
    #[must_use]
    pub const fn rect(&self) -> (f64, f64, f64, f64) { (self.x, self.y, self.w, self.h) }
    #[must_use]
    pub const fn id(&self) -> usize { self.id }
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
        }
    }
}

impl LinkSpec {
    #[must_use]
    pub fn kind(mut self, kind: impl Into<String>) -> Self { self.kind = kind.into(); self }
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self { self.label = Some(label.into()); self }
    #[must_use]
    pub fn start(mut self, start: impl Into<String>) -> Self { self.start = start.into(); self }
    #[must_use]
    pub fn end(mut self, end: impl Into<String>) -> Self { self.end = end.into(); self }
    #[must_use]
    pub const fn via_x(mut self, x: f64) -> Self { self.via_x = Some(x); self }
    #[must_use]
    pub const fn via_y(mut self, y: f64) -> Self { self.via_y = Some(y); self }
    #[must_use]
    pub const fn cross_ok(mut self, ok: bool) -> Self { self.cross_ok = ok; self }
    #[must_use]
    pub fn dash(mut self, dash: impl Into<String>) -> Self { self.dash = Some(dash.into()); self }
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

pub struct Node {
    pub placed: Placed,
    pub block: Block,
}

pub struct Link {
    pub a: usize,
    pub b: usize,
    pub kind: String,
    pub label: Option<String>,
    pub start: String,
    pub end: String,
    pub via_x: Option<f64>,
    pub via_y: Option<f64>,
    pub cross_ok: bool,
    pub offsets: [f64; 2],
    pub d: String,
    pub tip: Point,
    pub direction: Point,
    pub samples: Vec<Point>,
    pub label_spots: Vec<Point>,
    pub label_at: Point,
    pub want_start: String,
    pub want_end: String,
    pub start_pt: Point,
    pub dash: Option<String>,
}

pub struct Group {
    pub label: String,
    pub members: Vec<usize>,
    pub sub: Option<String>,
    pub pad: f64,
    pub rect: (f64, f64, f64, f64),
    pub border: String,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub children: Vec<usize>,
    pub enclosure: bool,
    pub style: String,
    pub tone: String,
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
            nodes: Vec::new(),
            links: Vec::new(),
            groups: Vec::new(),
            early: Vec::new(),
        }
    }

    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self { self.subtitle = Some(subtitle.into()); self }
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self { self.tag = Some(tag.into()); self }
    #[must_use]
    pub fn note(mut self, note: Note) -> Self { self.note = Some(note); self }
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self { self.theme = theme; self }
    #[must_use]
    pub const fn legend(mut self, legend: bool) -> Self { self.legend = legend; self }
    #[must_use]
    pub fn steps(mut self, steps: Vec<(i64, String)>, label: impl Into<String>) -> Self {
        self.steps = steps;
        self.steps_label = label.into();
        self
    }

    pub fn placed(&self) -> impl Iterator<Item = &Placed> {
        self.nodes.iter().map(|n| &n.placed)
    }

    #[must_use]
    pub fn lines(&self) -> Vec<LineView> {
        self.links.iter().map(|line| LineView {
            a: self.nodes[line.a].placed.name.clone(),
            b: self.nodes[line.b].placed.name.clone(),
            kind: line.kind.clone(),
            label: line.label.clone(),
            start: line.start.clone(),
            end: line.end.clone(),
        }).collect()
    }

    /// # Errors
    /// Returns an error when the block's text or status cannot be measured.
    pub fn add(&mut self, block: impl Into<Block>, x: f64, y: f64) -> Result<Placed, DiagramError> {
        self.add_width(block.into(), x, y, None, None)
    }

    /// # Errors
    /// Returns an error when the block's text or status cannot be measured.
    pub fn add_named(&mut self, block: impl Into<Block>, x: f64, y: f64, width: Option<f64>, name: Option<String>) -> Result<Placed, DiagramError> {
        self.add_width(block.into(), x, y, width, name)
    }

    fn add_width(&mut self, block: Block, x: f64, y: f64, width: Option<f64>, name: Option<String>) -> Result<Placed, DiagramError> {
        let (mut measured_w, measured_h) = block.measure(&self.theme)?;
        let mut label = name
            .or_else(|| block.explicit_name().map(str::to_owned))
            .or_else(|| block.title().map(str::to_owned))
            .unwrap_or_else(|| format!("block {}", self.nodes.len() + 1));
        if self.nodes.iter().any(|n| n.placed.name == label) {
            let copies = self.nodes.iter().filter(|node| node.placed.name.starts_with(&label)).count() + 1;
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
        let placed = Placed { id: self.nodes.len(), name: label, x, y, w: measured_w, h: measured_h };
        let out = placed.clone();
        self.nodes.push(Node { placed, block });
        Ok(out)
    }

    /// # Errors
    /// Returns an error when `align` is not top, center, or bottom, or the block cannot be measured.
    pub fn right_of(&mut self, refer: &Placed, block: impl Into<Block>, gap: Option<f64>, align: &str, width: Option<f64>, name: Option<String>) -> Result<Placed, DiagramError> {
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
    pub fn below(&mut self, refer: &Placed, block: impl Into<Block>, gap: Option<f64>, align: &str, width: Option<f64>, name: Option<String>) -> Result<Placed, DiagramError> {
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
    pub fn stack(&mut self, blocks: Vec<Block>, x: f64, y: f64, gap: Option<f64>) -> Result<Vec<Placed>, DiagramError> {
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
        let widths: Vec<f64> = measured.iter().map(|col| col.iter().map(|(_, s)| s.0).fold(0.0, f64::max)).collect();
        let heights: Vec<f64> = measured
            .iter()
            .map(|col| col.iter().map(|(_, s)| s.1).sum::<f64>() + mul(row_gap, count(col.len().saturating_sub(1))))
            .collect();
        let tallest = heights.iter().copied().fold(0.0, f64::max);
        let mut out = Vec::new();
        let mut cx = origin_x;
        for (index, col) in measured.into_iter().enumerate() {
            let mut cy = origin_y + match valign {
                "top" => 0.0,
                "center" => (tallest - heights[index]) / 2.0,
                "bottom" => tallest - heights[index],
                other => return Err(DiagramError::new(format!("unknown valign {other}"))),
            };
            let mut placed = Vec::new();
            for (block, (width, height)) in col {
                let px = if same_width { cx } else { cx + (widths[index] - width) / 2.0 };
                let column_width = if same_width { Some(widths[index]) } else { None };
                placed.push(self.add_width(block, px, cy, column_width, None)?);
                cy += height + row_gap;
            }
            out.push(placed);
            cx += widths[index] + gap;
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
        let mid_h = mid_sizes.iter().map(|s| s.1).sum::<f64>() + mul(self.theme.row_gap, count(mid_sizes.len().saturating_sub(1)));
        let n = blocks.len();
        let angles: Vec<f64> = (0..n).map(|i| (start_angle + mul(360.0, count(i)) / count(n)).to_radians()).collect();
        let mean_w = sizes.iter().map(|s| s.0).sum::<f64>() / count(n);
        let mean_h = sizes.iter().map(|s| s.1).sum::<f64>() / count(n);
        let stretch = 1.8_f64.min(1.0_f64.max(mean_w / mean_h));
        let rects = |r: f64| -> Vec<(f64, f64, f64, f64)> {
            angles.iter().zip(&sizes).map(|(a, (w, h))| (mul(mul(r, stretch), a.cos()) - w / 2.0, mul(r, a.sin()) - h / 2.0, *w, *h)).collect()
        };
        let mid_rect = (!center.is_empty()).then_some((-mid_w / 2.0, -mid_h / 2.0, mid_w, mid_h));
        let mut r = 40.0;
        let rs = loop {
            let rs = rects(r);
            let mut ok = (0..n).all(|i| (i + 1..n).all(|j| separation(rs[i], rs[j]) >= gap));
            if ok
                && let Some(mid) = mid_rect {
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
            placed.push(self.add_width(block, x + box_.0 - left, y + box_.1 - north, None, None)?);
        }
        let mut middle_placed = Vec::new();
        if let Some(mid) = mid_rect {
            let mut cy = y + mid.1 - north;
            for (block, (w, h)) in center.into_iter().zip(mid_sizes) {
                let _ = w;
                middle_placed.push(self.add_width(block, x + mid.0 - left, cy, Some(mid_w), None)?);
                cy += h + self.theme.row_gap;
            }
        }
        Ok((placed, middle_placed))
    }

    pub fn group(&mut self, label: impl Into<String>, members: &[Placed], sub: Option<String>, pad: Option<f64>) -> GroupId {
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
    pub fn enclosure(&mut self, label: impl Into<String>, members: &[Enclosed], opts: EnclosureOpts<'_>) -> Result<GroupId, DiagramError> {
        let EnclosureOpts { sub, pad, border, fill, stroke, style, tone } = opts;
        let label = label.into();
        if !matches!(border, "solid" | "dashed" | "dotted") {
            return Err(DiagramError::new(format!("enclosure {}: border must be solid, dashed or dotted", py_repr(&label))));
        }
        if !pad.is_finite() || pad < 0.0 {
            return Err(DiagramError::new(format!("enclosure {}: pad must be a non-negative finite number", py_repr(&label))));
        }
        if !matches!(style, "outline" | "panel") {
            return Err(DiagramError::new(format!("enclosure {}: style must be outline or panel", py_repr(&label))));
        }
        self.theme.tone(tone)?;
        let mut nodes = Vec::new();
        let mut children = Vec::new();
        for member in members {
            let found = match member {
                Enclosed::Placed(p) if self.nodes.get(p.id).is_some_and(|n| n.placed.name == p.name) => vec![p.id],
                Enclosed::Group(GroupId(id)) if self.groups.get(*id).is_some() => {
                    children.push(*id);
                    self.groups[*id].members.clone()
                }
                _ => return Err(DiagramError::new(format!("enclosure {}: members must belong to this diagram", py_repr(&label)))),
            };
            for node in found {
                if !nodes.contains(&node) {
                    nodes.push(node);
                }
            }
        }
        if nodes.is_empty() {
            return Err(DiagramError::new(format!("enclosure {}: needs at least one resource", py_repr(&label))));
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
                return Err(DiagramError::new(format!("unknown side {}; use left, right, top, bottom or auto", py_repr(side))));
            }
        }
        if !self.theme.link_labels.contains_key(&spec.kind) {
            let names = self.theme.link_labels.keys().cloned().collect::<Vec<_>>().join(", ");
            return Err(DiagramError::new(format!("unknown line kind {}; use one of {names}", py_repr(&spec.kind))));
        }
        if !matches!(spec.dash.as_deref(), None | Some("solid" | "dashed" | "dotted")) {
            return Err(DiagramError::new("link dash must be solid, dashed, dotted or None"));
        }
        if spec.via_x.is_some() && spec.via_y.is_some() {
            return Err(DiagramError::new("give via_x or via_y, not both"));
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
        });
        Ok(())
    }

    /// # Errors
    /// Returns an error when a state is unknown or a name matches nothing.
    pub fn set_statuses(&mut self, statuses: &IndexMap<String, String>) -> Result<(), DiagramError> {
        let bad: BTreeSet<_> = statuses.values().filter(|s| !self.theme.statuses.contains_key(s.as_str())).cloned().collect();
        if !bad.is_empty() {
            let bad = bad.into_iter().map(|s| py_repr(&s)).collect::<Vec<_>>().join(", ");
            let names = self.theme.statuses.keys().cloned().collect::<Vec<_>>().join(", ");
            return Err(DiagramError::new(format!("unknown state(s) {bad}; this theme has: {names}")));
        }
        let mut found = BTreeSet::new();
        for node in &mut self.nodes {
            apply_statuses(&mut node.block, statuses, &mut found);
        }
        let missing: Vec<_> = statuses.keys().filter(|n| !found.contains(n.as_str())).map(|n| py_repr(n)).collect();
        if !missing.is_empty() {
            return Err(DiagramError::new(format!("no panel or tile is named {}", missing.join(", "))));
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
    pub fn save(&mut self, path: &str, png: bool, scale: f64, strict: bool) -> Result<Report, SaveError> {
        let base = strip_picture_suffix(path);
        let svg_path = format!("{base}.svg");
        if let Some(parent) = std::path::Path::new(&svg_path).parent()
            && !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| SaveError::Io(format!("{svg_path}: {err}")))?;
            }
        let mut report = self.render()?;
        std::fs::write(&svg_path, report.svg.as_bytes()).map_err(|err| SaveError::Io(format!("{svg_path}: {err}")))?;
        report.files.push(svg_path);
        if png {
            let png_path = format!("{base}.png");
            if crate::render::svg_to_png(std::path::Path::new(&report.files[0]), std::path::Path::new(&png_path), report.width, report.height, scale, None) {
                report.files.push(png_path);
            } else {
                println!("PNG skipped: Chrome or Chromium not found (set the CHROME environment variable to its path)");
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
            return Err(SaveError::Layout(crate::error::LayoutError::new(report.problems.clone(), report.files.clone())));
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
                    && self.theme.tone(&panel.tone)? == *accent {
                        uses_accent = true;
                    }
            }
            for group in &self.groups {
                if group.style == "panel" && self.theme.tone(&group.tone)? == *accent {
                    uses_accent = true;
                }
            }
        }
        let has_steps = !self.steps.is_empty() || self.nodes.iter().any(|n| matches!(&n.block, Block::Panel(p) if p.step.is_some()));
        if uses_accent && has_steps && self.theme.badge_fill.is_some() && self.theme.badge_fill.as_deref() != self.theme.tones.get("dark").map(String::as_str) {
            let mut theme = self.theme.clone();
            theme.badge_fill = Some(self.theme.tones.get("dark").cloned().unwrap_or_else(|| "#2B3036".to_owned()));
            theme.badge_text = Some(self.theme.header_text_on_dark.clone());
            return Ok(theme);
        }
        Ok(self.theme.clone())
    }

    fn render_inner(&mut self) -> Result<Report, DiagramError> {
        if self.nodes.is_empty() {
            return Err(DiagramError::new("the diagram has no blocks; add some with add(), right_of(), below() or columns()"));
        }
        self.measure_groups()?;
        self.route();
        let mut problems = self.check();
        let mut boxes: Vec<(f64, f64, f64, f64)> = self.nodes.iter().map(|n| n.placed.rect()).chain(self.groups.iter().map(|g| g.rect)).collect();
        for (i, line) in self.links.iter().enumerate() {
            if line.label.is_some() {
                boxes.push(self.label_box(i, None));
            }
        }
        let pts: Vec<Point> = self.links.iter().flat_map(|l| l.samples.iter().copied()).collect();
        let left = boxes.iter().map(|b| b.0).chain(pts.iter().map(|p| p.0)).fold(f64::INFINITY, f64::min);
        let north = boxes.iter().map(|b| b.1).chain(pts.iter().map(|p| p.1)).fold(f64::INFINITY, f64::min);
        let right = boxes.iter().map(|b| b.0 + b.2).chain(pts.iter().map(|p| p.0)).fold(f64::NEG_INFINITY, f64::max);
        let south = boxes.iter().map(|b| b.1 + b.3).chain(pts.iter().map(|p| p.1)).fold(f64::NEG_INFINITY, f64::max);
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
        canvas.open_group(dx, top - north);
        let order = self.group_draw_order();
        for id in &order {
            self.draw_group_body(&mut canvas, *id)?;
        }
        for i in 0..self.links.len() {
            let dotted = self.links[i].kind == "planned";
            let dash = link_dash(&self.links[i]);
            let d = self.links[i].d.clone();
            let tip = self.links[i].tip;
            let direction = self.links[i].direction;
            canvas.path(&d, "none", Some(&self.theme.line), if dotted { 1.6 } else { 1.3 }, dash.as_deref(), None);
            canvas.path(&arrowhead(tip, direction), &self.theme.line, None, 1.0, None, None);
        }
        for g in 0..self.groups.len() {
            self.draw_group_title(&mut canvas, g)?;
        }
        for i in 0..self.nodes.len() {
            let placed = self.nodes[i].placed.clone();
            let mut sink = |msg: &str| problems.push(msg.to_owned());
            self.nodes[i].block.draw(&mut canvas, (placed.x, placed.y, placed.w, placed.h), &self.theme.clone(), &mut sink)?;
        }
        for i in 0..self.links.len() {
            if self.links[i].label.is_some() {
                let (x, y, w, h) = self.label_box(i, None);
                let label = self.links[i].label.clone().unwrap_or_default();
                canvas.rect(x, y, w, h, Paint::new(&self.theme.paper, Some(&self.theme.line), 0.8, h / 2.0, None, None));
                canvas.text_centered(&label, x + w / 2.0, y + h / 2.0, self.theme.small, "medium", &self.theme.ink);
            }
        }
        canvas.close_group();
        if let Some(min_text) = canvas.min_text
            && min_text < self.theme.min_text {
                problems.push(format!(
                    "smallest text is {} px; the theme's minimum is {} px",
                    py_g(min_text),
                    py_g(self.theme.min_text)
                ));
            }
        let svg = canvas.svg(Some(&self.title), Some(&self.theme.paper), Some(&self.describe()));
        Ok(Report { svg, width: canvas.width, height: canvas.height, min_text: canvas.min_text, problems, files: Vec::new() })
    }

    fn measure_groups(&mut self) -> Result<(), DiagramError> {
        for i in 0..self.groups.len() {
            let pad = self.groups[i].pad;
            let sub = self.groups[i].sub.clone();
            let style = self.groups[i].style.clone();
            let label = self.groups[i].label.clone();
            let tone = self.groups[i].tone.clone();
            let enclosure = self.groups[i].enclosure;
            let members = self.groups[i].members.clone();
            let children = self.groups[i].children.clone();
            let mut bounds: Vec<(f64, f64, f64, f64)> = members.iter().map(|id| self.nodes[*id].placed.rect()).collect();
            for child in &children {
                bounds.push(self.groups[*child].rect);
            }
            let x0 = bounds.iter().map(|b| b.0).fold(f64::INFINITY, f64::min) - pad;
            let head_h = if style == "panel" {
                self.theme.lid + if sub.is_some() { self.theme.header_h_sub } else { self.theme.header_h }
            } else {
                mul(self.theme.line_h, if sub.is_some() { 2.0 } else { 1.0 }) + 6.0
            };
            let y0 = bounds.iter().map(|b| b.1).fold(f64::INFINITY, f64::min) - pad - head_h;
            let mut x1 = bounds.iter().map(|b| b.0 + b.2).fold(f64::NEG_INFINITY, f64::max) + pad;
            let y1 = bounds.iter().map(|b| b.1 + b.3).fold(f64::NEG_INFINITY, f64::max) + pad;
            if style == "panel" {
                let (title_w, _) = Panel::new(label).sub_opt(sub).tone(tone).measure(&self.theme)?;
                x1 = x1.max(x0 + title_w);
            } else if enclosure {
                let title_w = text_width(&label, 12.5, "semibold").max(text_width(sub.as_deref().unwrap_or(""), self.theme.small, "regular"));
                x1 = x1.max(x0 + title_w + 28.0);
            }
            self.groups[i].rect = (x0, y0, x1 - x0, y1 - y0);
        }
        Ok(())
    }

    fn route(&mut self) {
        for i in 0..self.links.len() {
            if !self.links[i].want_start.is_empty() {
                let a = self.links[i].a;
                let b = self.links[i].b;
                let want_start = self.links[i].want_start.clone();
                let want_end = self.links[i].want_end.clone();
                self.links[i].start = if want_start == "auto" { facing(&self.nodes[a].placed, &self.nodes[b].placed) } else { want_start };
                self.links[i].end = if want_end == "auto" { facing(&self.nodes[b].placed, &self.nodes[a].placed) } else { want_end };
            }
        }
        let mut ends: IndexMap<(usize, String), Vec<(usize, usize)>> = IndexMap::new();
        for (i, line) in self.links.iter().enumerate() {
            ends.entry((line.a, line.start.clone())).or_default().push((i, 0));
            ends.entry((line.b, line.end.clone())).or_default().push((i, 1));
        }
        for ((owner, side), mut items) in ends {
            items.sort_by(|a, b| self.heading(*a, &side).total_cmp(&self.heading(*b, &side)));
            let (lo, hi) = self.nodes[owner].block.ports(&side, self.nodes[owner].placed.w, self.nodes[owner].placed.h, &self.theme);
            let n = items.len();
            let step = if n > 1 { ((hi - lo) / count(n)).min(14.0) } else { 0.0 };
            let span = lo + hi;
            let mid = span / 2.0;
            for (i, (line, end)) in items.into_iter().enumerate() {
                let slot = count(i) - (count(n) - 1.0) / 2.0;
                self.links[line].offsets[end] = mid + mul(slot, step);
            }
        }
        for i in 0..self.links.len() {
            self.shape(i);
        }
        self.place_labels();
    }

    fn heading(&self, item: (usize, usize), side: &str) -> f64 {
        let line = &self.links[item.0];
        let other = if item.1 == 0 { line.b } else { line.a };
        if matches!(side, "left" | "right") {
            line.via_y.unwrap_or_else(|| self.nodes[other].placed.cy())
        } else {
            line.via_x.unwrap_or_else(|| self.nodes[other].placed.cx())
        }
    }

    fn shape(&mut self, index: usize) {
        let a = self.links[index].a;
        let b = self.links[index].b;
        let start = self.links[index].start.clone();
        let end = self.links[index].end.clone();
        let off0 = self.links[index].offsets[0];
        let off1 = self.links[index].offsets[1];
        let via_x = self.links[index].via_x;
        let via_y = self.links[index].via_y;
        let p0 = self.anchor_at(a, &start, off0);
        let tip = self.anchor_at(b, &end, off1);
        let d3 = side_vector(&end).unwrap_or((1.0, 0.0));
        let p3 = (tip.0 + mul(d3.0, ARROW), tip.1 + mul(d3.1, ARROW));
        self.links[index].start_pt = p0;
        self.links[index].tip = tip;
        self.links[index].direction = (-d3.0, -d3.1);
        if via_x.is_none() && via_y.is_none() {
            let along = if matches!(start.as_str(), "left" | "right") { (p3.0 - p0.0).abs() } else { (p3.1 - p0.1).abs() };
            let k = 28.0_f64.max(along * 0.5);
            let c1 = outside(&self.nodes[a].placed, p0, &start, k);
            let c2 = outside(&self.nodes[b].placed, p3, &end, k);
            self.links[index].d = format!("M{}C{} {} {}", pt(p0), pt(c1), pt(c2), pt(p3));
            self.links[index].samples = (0..=80).map(|i| bezier(p0, c1, c2, p3, f64::from(i) / 80.0)).collect();
            self.links[index].label_spots = [0.5, 0.42, 0.58, 0.34, 0.66, 0.26, 0.74].into_iter().map(|s| bezier(p0, c1, c2, p3, s)).collect();
            self.links[index].label_at = self.links[index].label_spots[0];
            return;
        }
        let s0 = outside(&self.nodes[a].placed, p0, &start, STUB);
        let s3 = outside(&self.nodes[b].placed, p3, &end, STUB);
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
        let pts = simplify(&pts);
        self.links[index].d = rounded(&pts);
        self.links[index].samples = sample(&pts, 5.0);
        let mut order: Vec<usize> = (0..pts.len() - 1).collect();
        order.sort_by(|&i, &j| dist(pts[j], pts[j + 1]).total_cmp(&dist(pts[i], pts[i + 1])));
        let mut spots = Vec::new();
        for i in order.into_iter().take(2) {
            let a = pts[i];
            let b = pts[i + 1];
            for f in [0.5, 0.35, 0.65, 0.2, 0.8] {
                spots.push((a.0 + mul(b.0 - a.0, f), a.1 + mul(b.1 - a.1, f)));
            }
        }
        self.links[index].label_spots = spots;
        self.links[index].label_at = self.links[index].label_spots[0];
    }

    fn anchor_at(&self, id: usize, side: &str, offset: f64) -> Point {
        let node = &self.nodes[id];
        let (x, y) = node.block.anchor(side, offset, node.placed.w, node.placed.h, &self.theme);
        (node.placed.x + x, node.placed.y + y)
    }

    fn place_labels(&mut self) {
        let titles: Vec<_> = (0..self.groups.len()).map(|i| self.group_title_box(i)).collect();
        let placed: Vec<_> = self.nodes.iter().map(|n| n.placed.rect()).collect();
        let samples: Vec<Vec<Point>> = self.links.iter().map(|l| l.samples.clone()).collect();
        let mut taken = Vec::new();
        for i in 0..self.links.len() {
            if self.links[i].label.is_none() {
                continue;
            }
            let spots = self.links[i].label_spots.clone();
            let mut chosen = spots[0];
            for spot in &spots {
                if label_clear(self.label_box(i, Some(*spot)), &placed, &titles, &taken, &samples, i) {
                    chosen = *spot;
                    break;
                }
            }
            self.links[i].label_at = chosen;
            taken.push(self.label_box(i, Some(chosen)));
        }
    }

    fn label_box(&self, index: usize, at: Option<Point>) -> (f64, f64, f64, f64) {
        let (x, y) = at.unwrap_or(self.links[index].label_at);
        let w = text_width(self.links[index].label.as_deref().unwrap_or(""), self.theme.small, "medium") + 14.0;
        (x - w / 2.0, y - 9.0, w, 18.0)
    }

    fn group_title_box(&self, index: usize) -> (f64, f64, f64, f64) {
        let group = &self.groups[index];
        let (x, y, w, _) = group.rect;
        if group.style == "panel" {
            let h = self.theme.lid + if group.sub.is_some() { self.theme.header_h_sub } else { self.theme.header_h };
            return (x, y, w, h);
        }
        let mut tw = text_width(&group.label, 12.5, "semibold");
        if let Some(sub) = &group.sub {
            tw = tw.max(text_width(sub, self.theme.small, "regular"));
        }
        (x + 10.0, y + 7.0, tw + 8.0, mul(self.theme.line_h, if group.sub.is_some() { 2.0 } else { 1.0 }) + 8.0)
    }

    fn check(&self) -> Vec<String> {
        let mut problems = self.early.clone();
        for i in 0..self.nodes.len() {
            for j in i + 1..self.nodes.len() {
                let a = &self.nodes[i].placed;
                let b = &self.nodes[j].placed;
                let gap = separation(a.rect(), b.rect());
                if gap < 0.0 {
                    problems.push(format!("'{}' and '{}' overlap; move one (right_of() and below() keep a gap)", a.name, b.name));
                } else if gap < self.theme.min_gap {
                    problems.push(format!(
                        "'{}' and '{}' are only {} px apart (minimum {})",
                        a.name,
                        b.name,
                        py_fixed0(gap),
                        py_fixed0(self.theme.min_gap)
                    ));
                }
            }
        }
        for g in &self.groups {
            for node in &self.nodes {
                if !g.members.contains(&node.placed.id) && separation(g.rect, node.placed.rect()) < 0.0 {
                    problems.push(format!(
                        "group '{}' overlaps '{}', which is not in it; add it to the group or move it",
                        g.label, node.placed.name
                    ));
                }
            }
        }
        for i in 0..self.groups.len() {
            for h in i + 1..self.groups.len() {
                if separation(self.groups[i].rect, self.groups[h].rect) < 0.0
                    && !contains(self.groups[i].rect, self.groups[h].rect)
                    && !contains(self.groups[h].rect, self.groups[i].rect)
                {
                    problems.push(format!("groups '{}' and '{}' overlap", self.groups[i].label, self.groups[h].label));
                }
            }
        }
        for line in &self.links {
            let name = format!("line '{}' -> '{}'", self.nodes[line.a].placed.name, self.nodes[line.b].placed.name);
            if line.via_x.is_none() && line.via_y.is_none() {
                let d0 = side_vector(&line.start).unwrap_or((0.0, 0.0));
                let d3 = side_vector(&line.end).unwrap_or((0.0, 0.0));
                let vx = line.tip.0 - line.start_pt.0;
                let vy = line.tip.1 - line.start_pt.1;
                let fix = "use start='auto' and end='auto', other sides, or a lane (via_x/via_y)";
                if mul(d0.0, vx) + mul(d0.1, vy) < 1.0 {
                    problems.push(format!(
                        "{name} leaves '{}' from its {} side, facing away from '{}'; {fix}",
                        self.nodes[line.a].placed.name, line.start, self.nodes[line.b].placed.name
                    ));
                } else if mul(d3.0, -vx) + mul(d3.1, -vy) < 1.0 {
                    problems.push(format!(
                        "{name} enters '{}' from its {} side, facing away from '{}'; {fix}",
                        self.nodes[line.b].placed.name, line.end, self.nodes[line.a].placed.name
                    ));
                }
            }
            if !line.cross_ok {
                for node in &self.nodes {
                    if node.placed.id == line.a || node.placed.id == line.b {
                        continue;
                    }
                    let (x, y, w, h) = node.placed.rect();
                    if line.samples.iter().any(|s| x + 2.0 < s.0 && s.0 < x + w - 2.0 && y + 2.0 < s.1 && s.1 < y + h - 2.0) {
                        problems.push(format!("{name} crosses '{}'; route it with via_x/via_y, move a block, or pass cross_ok=True", node.placed.name));
                    }
                }
            }
            if line.label.is_some() {
                let box_ = self.label_box_of(line);
                for node in &self.nodes {
                    if separation(box_, node.placed.rect()) < 0.0 {
                        problems.push(format!("the label '{}' on {name} sits on '{}'", line.label.as_deref().unwrap_or(""), node.placed.name));
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
        problems
    }

    fn label_box_of(&self, line: &Link) -> (f64, f64, f64, f64) {
        let (x, y) = line.label_at;
        let w = text_width(line.label.as_deref().unwrap_or(""), self.theme.small, "medium") + 14.0;
        (x - w / 2.0, y - 9.0, w, 18.0)
    }

    fn group_draw_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.groups.len()).collect();
        if self.groups.iter().any(|g| g.enclosure) {
            order.sort_by(|&a, &b| {
                let aa = self.groups[a].rect.2 * self.groups[a].rect.3;
                let bb = self.groups[b].rect.2 * self.groups[b].rect.3;
                bb.total_cmp(&aa)
            });
        }
        order
    }

    fn draw_group_body(&self, canvas: &mut Canvas, id: usize) -> Result<(), DiagramError> {
        let group = &self.groups[id];
        let (x, y, w, h) = group.rect;
        if group.style == "panel" {
            let tone = self.theme.tone(&group.tone)?;
            let fill = group.fill.as_deref().unwrap_or(&self.theme.panel_fill);
            let stroke = group.stroke.as_deref().unwrap_or(&self.theme.panel_stroke);
            let dash = (group.border != "solid").then_some(group.border.as_str());
            blocks::panel_frame(
                canvas,
                (x, y, w, h),
                blocks::PanelFace { theme: &self.theme, tone: &tone, fill, stroke, width: 1.0, dash },
            );
        } else {
            let fill = group.fill.as_deref().unwrap_or(&self.theme.group_fill);
            let stroke = group.stroke.as_deref().unwrap_or(&self.theme.group_stroke);
            let dash = (group.border != "solid").then_some(group.border.as_str());
            canvas.rect(x, y, w, h, Paint::new(fill, Some(stroke), 1.1, self.theme.radius + 4.0, dash, None));
        }
        Ok(())
    }

    fn draw_group_title(&self, canvas: &mut Canvas, id: usize) -> Result<(), DiagramError> {
        let g_label = self.groups[id].label.clone();
        let g_sub = self.groups[id].sub.clone();
        let g_style = self.groups[id].style.clone();
        let g_tone = self.groups[id].tone.clone();
        let g_fill = self.groups[id].fill.clone();
        let (x, y, w, _) = self.groups[id].rect;
        if g_style == "panel" {
            let tone = self.theme.tone(&g_tone)?;
            let (title_color, sub_color) = self.theme.header_colors(&tone)?;
            let y = y + self.theme.lid;
            let hh = if g_sub.is_some() { self.theme.header_h_sub } else { self.theme.header_h };
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
                canvas.text(sub, x + self.theme.pad, y + 36.0 + cap_height(self.theme.panel_sub, "regular") / 2.0, Letter::new(self.theme.panel_sub, "regular", &sub_color, "start"));
            }
            return Ok(());
        }
        let (bx, by, bw, bh) = self.group_title_box(id);
        let backing = if g_fill.as_deref() == Some("none") {
            self.theme.paper.clone()
        } else {
            g_fill.unwrap_or_else(|| self.theme.group_fill.clone())
        };
        canvas.rect(bx, by, bw, bh, Paint::new(&backing, None, 1.0, 3.0, None, None));
        canvas.text(&g_label, x + 14.0, y + 12.0 + cap_height(12.5, "semibold"), Letter::new(12.5, "semibold", &self.theme.ink, "start"));
        if let Some(sub) = &g_sub {
            canvas.text(sub, x + 14.0, y + 12.0 + self.theme.line_h + cap_height(self.theme.small, "regular"), Letter::new(self.theme.small, "regular", &self.theme.muted, "start"));
        }
        Ok(())
    }

    fn header_plan(&self, content_w: f64) -> HeaderPlan {
        let t = &self.theme;
        let m = t.margin;
        let tag_w = self.tag.as_ref().map_or(0.0, |tag| text_width(tag, 13.0, "bold") + 24.0);
        let title_w = tag_w + if tag_w == 0.0 { 0.0 } else { 12.0 } + text_width(&self.title, t.title, "bold");
        let sub_w = self.subtitle.as_ref().map_or(0.0, |s| text_width(s, t.subtitle, "regular"));
        let (note_w, note_h) = self.note.as_ref().map_or((0.0, 0.0), |n| n.measure(t));
        let min_w = mul(2.0, m) + title_w.max(sub_w) + if self.note.is_some() { 48.0 + note_w } else { 0.0 };
        let width = (content_w + mul(2.0, m)).max(min_w).max(640.0);
        let room = width - mul(2.0, m) - if self.note.is_some() { note_w + 40.0 } else { 0.0 };
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
        let h = 34.0 + if self.subtitle.is_some() { 24.0 } else { 0.0 } + if rows.is_empty() && !steps { 0.0 } else { 14.0 } + mul(24.0, count(rows.len())) + if steps { 24.0 } else { 0.0 };
        HeaderPlan { width, height: h.max(note_h), rows, steps, tag_w, note_w, centre: content_w + mul(2.0, m) < width }
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
        for key in self.theme.statuses.keys() {
            if used.contains(key) {
                items.push(LegendItem { kind: "status".to_owned(), key: key.clone(), label: self.theme.statuses[key].label.clone() });
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
                let default = if kind == "planned" { Some("dotted".to_owned()) } else { None };
                let style = dash.clone().unwrap_or_else(|| "solid".to_owned());
                let item_kind = if dash == &default { "line".to_owned() } else { format!("line-{style}") };
                let text = if styles.len() == 1 { label.clone() } else { format!("{label} ({style})") };
                items.push(LegendItem { kind: item_kind, key: kind.clone(), label: text });
            }
        }
        items
    }

    fn item_w(&self, item: &LegendItem) -> f64 {
        (if item.kind == "status" { 30.0 } else { 34.0 }) + text_width(&item.label, self.theme.legend, "regular") + 22.0
    }

    fn draw_header(&self, canvas: &mut Canvas, plan: &HeaderPlan) -> Result<(), DiagramError> {
        let t = &self.theme;
        let m = t.margin;
        let mut x = m;
        let cy = m + 15.0;
        if let Some(tag) = &self.tag {
            let w = plan.tag_w;
            canvas.rect(x, cy - 14.0, w, 28.0, Paint::new(&t.paper, Some(&t.tag_color), 1.4, 14.0, None, None));
            canvas.text_centered(tag, x + w / 2.0, cy, 13.0, "bold", &t.tag_color);
            x += w + 12.0;
        }
        canvas.text(&self.title, x, cy + cap_height(t.title, "bold") / 2.0, Letter::new(t.title, "bold", &t.ink, "start"));
        let mut y = m + 34.0;
        if let Some(subtitle) = &self.subtitle {
            canvas.text(subtitle, m, y + 10.0 + cap_height(t.subtitle, "regular") / 2.0, Letter::new(t.subtitle, "regular", &t.muted, "start"));
            y += 24.0;
        }
        if !plan.rows.is_empty() || plan.steps {
            y += 14.0;
        }
        for row in &plan.rows {
            let mut lx = m;
            let ly = y + 10.0;
            for item in row {
                if item.kind == "status" {
                    let st = &t.statuses[&item.key];
                    canvas.rect(lx, ly - 7.0, 22.0, 14.0, Paint::new(&st.fill, Some(&st.stroke), 1.2, 3.0, st.dash.as_deref(), None));
                    lx += 30.0;
                } else {
                    let dotted = item.key == "planned";
                    let dash = if item.kind == "line" {
                        if dotted { Some("dotted".to_owned()) } else { None }
                    } else {
                        Some(item.kind["line-".len()..].to_owned())
                    };
                    let dash = dash.filter(|d| d != "solid");
                    canvas.line((lx, ly), (lx + 22.0, ly), &t.line, if dotted { 1.6 } else { 1.3 }, dash.as_deref());
                    canvas.path(&arrowhead((lx + 28.0, ly), (1.0, 0.0)), &t.line, None, 1.0, None, None);
                    lx += 34.0;
                }
                lx += canvas.text(&item.label, lx, ly + cap_height(t.legend, "regular") / 2.0, Letter::new(t.legend, "regular", &t.ink, "start")) + 22.0;
            }
            y += 24.0;
        }
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
                canvas.circle(lx + 9.0, ly, 9.0, t.badge_fill.as_deref().unwrap_or(&tone), None, 1.0);
                canvas.text_centered(&n.to_string(), lx + 9.0, ly, 10.5, "bold", t.badge_text.as_deref().unwrap_or(&t.paper));
                lx += 24.0 + canvas.text(label, lx + 24.0, ly + cap_height(t.legend, "regular") / 2.0, Letter::new(t.legend, "regular", &t.ink, "start")) + 18.0;
            }
        }
        if let Some(note) = &self.note {
            let (_, nh) = note.measure(t);
            note.draw(canvas, (plan.width - m - plan.note_w, m, plan.note_w, nh), t);
        }
        Ok(())
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
        if b.cx() >= a.cx() { "right".to_owned() } else { "left".to_owned() }
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
    let inset = if dx == 0.0 { (edge - point.1) * dy } else { (edge - point.0) * dx };
    let distance = minimum.max(inset + STUB);
    (point.0 + mul(dx, distance), point.1 + mul(dy, distance))
}

fn pt(p: Point) -> String { format!("{} {}", fmt(p.0), fmt(p.1)) }

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
    (bezier_axis(u, s, p0.0, p1.0, p2.0, p3.0), bezier_axis(u, s, p0.1, p1.1, p2.1, p3.1))
}

fn dist(a: Point, b: Point) -> f64 { (a.0 - b.0).hypot(a.1 - b.1) }

fn simplify(pts: &[Point]) -> Vec<Point> {
    let mut out = Vec::new();
    for pt in pts {
        if let Some(last) = out.last()
            && dist(*last, *pt) < 0.01 {
                continue;
            }
        if out.len() >= 2 {
            let a = out[out.len() - 2];
            let b = out[out.len() - 1];
            let cross = mul(b.0 - a.0, pt.1 - b.1) - mul(b.1 - a.1, pt.0 - b.0);
            let dot = mul(b.0 - a.0, pt.0 - b.0) + mul(b.1 - a.1, pt.1 - b.1);
            if cross.abs() < 0.01 && dot >= 0.0 {
                *out.last_mut().unwrap_or(&mut (0.0, 0.0)) = *pt;
                continue;
            }
        }
        out.push(*pt);
    }
    out
}

fn rounded(pts: &[Point]) -> String {
    let mut path = vec![format!("M{}", pt(pts[0]))];
    for index in 1..pts.len() - 1 {
        let prev = pts[index - 1];
        let corner = pts[index];
        let next = pts[index + 1];
        let radius = CORNER.min(dist(prev, corner) / 2.0).min(dist(corner, next) / 2.0);
        let incoming = ((corner.0 - prev.0) / dist(prev, corner), (corner.1 - prev.1) / dist(prev, corner));
        let outgoing = ((next.0 - corner.0) / dist(corner, next), (next.1 - corner.1) / dist(corner, next));
        path.push(format!(
            "L{}Q{} {}",
            pt((corner.0 - mul(incoming.0, radius), corner.1 - mul(incoming.1, radius))),
            pt(corner),
            pt((corner.0 + mul(outgoing.0, radius), corner.1 + mul(outgoing.1, radius)))
        ));
    }
    path.push(format!("L{}", pt(*pts.last().unwrap_or(&pts[0]))));
    path.join("")
}

fn sample(pts: &[Point], step: f64) -> Vec<Point> {
    let mut out = Vec::new();
    for pair in pts.windows(2) {
        let (a, b) = (pair[0], pair[1]);
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
    format!("M{}L{}L{}Z", pt(tip), pt((bx + px, by + py)), pt((bx - px, by - py)))
}

fn separation(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> f64 {
    let dx = (b.0 - (a.0 + a.2)).max(a.0 - (b.0 + b.2));
    let dy = (b.1 - (a.1 + a.3)).max(a.1 - (b.1 + b.3));
    dx.max(dy)
}

fn contains(outer: (f64, f64, f64, f64), inner: (f64, f64, f64, f64)) -> bool {
    outer.0 <= inner.0 && outer.1 <= inner.1 && outer.0 + outer.2 >= inner.0 + inner.2 && outer.1 + outer.3 >= inner.1 + inner.3
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
    if titles.iter().chain(taken.iter()).any(|b| separation(box_, *b) < 0.0) {
        return false;
    }
    samples.iter().enumerate().filter(|(i, _)| *i != index).flat_map(|(_, s)| s).all(|s| {
        !(box_.0 - 2.0 < s.0 && s.0 < box_.0 + box_.2 + 2.0 && box_.1 - 2.0 < s.1 && s.1 < box_.1 + box_.3 + 2.0)
    })
}

#[cfg(test)]
#[path = "behavior.rs"]
mod behavior;
