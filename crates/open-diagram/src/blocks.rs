#![expect(
    clippy::redundant_pub_crate,
    reason = "sibling modules use these items; pub would fail unreachable_pub because they are not the external API"
)]

use std::collections::BTreeSet;
use std::path::Path;

use base64::Engine;
use indexmap::IndexMap;

use crate::error::DiagramError;
use crate::font::{cap_height, lines_width, text_width, wrap};
use crate::num::{count, fmt, mul, py_fixed0, py_g, py_repr};
use crate::svg::{Canvas, Letter, Paint};
use crate::theme::{Theme, shade};

const ICON: f64 = 16.0;

pub(crate) type ReportFn<'a> = &'a mut dyn FnMut(&str);
type GraphRect = (f64, f64, f64, f64);
type GraphLabel = (String, GraphRect);

#[derive(Clone, Debug)]
pub struct Resource {
    pub label: String,
    pub icon: Option<String>,
    pub sub: Option<String>,
    pub icon_size: f64,
    pub max_width: f64,
    pub border: Option<String>,
    pub name: String,
}

impl Resource {
    /// # Errors
    /// Returns an error when the label, sizes, border, or icon path is not valid.
    pub fn new(
        label: impl Into<String>,
        icon: Option<String>,
        sub: Option<String>,
        icon_size: f64,
        max_width: f64,
        border: Option<String>,
        name: Option<String>,
    ) -> Result<Self, DiagramError> {
        let label = label.into();
        if label.is_empty() {
            return Err(DiagramError::new("resource: label must not be empty"));
        }
        for (key, value) in [("icon_size", icon_size), ("max_width", max_width)] {
            if !value.is_finite() || value <= 0.0 {
                return Err(DiagramError::new(format!(
                    "resource {}: {key} must be a positive finite number",
                    py_repr(&label)
                )));
            }
        }
        if !matches!(
            border.as_deref(),
            None | Some("solid" | "dashed" | "dotted")
        ) {
            return Err(DiagramError::new(format!(
                "resource {}: border must be solid, dashed, dotted or None",
                py_repr(&label)
            )));
        }
        if let Some(icon) = &icon
            && !Path::new(icon).is_file()
        {
            return Err(DiagramError::new(format!(
                "resource {}: icon file not found: {icon}",
                py_repr(&label)
            )));
        }
        let name = name.unwrap_or_else(|| label.clone());
        Ok(Self {
            label,
            icon,
            sub,
            icon_size,
            max_width,
            border,
            name,
        })
    }

    fn lines(&self, t: &Theme) -> (Vec<String>, Vec<String>) {
        let room = (self.max_width - mul(2.0, t.pad)).max(1.0);
        let main = wrap(&self.label, t.tile, Some(room), "semibold");
        let sub = self
            .sub
            .as_ref()
            .map_or_default(|s| wrap(s, t.tile_sub, Some(room), "regular"));
        (main, sub)
    }

    fn measure(&self, t: &Theme) -> (f64, f64) {
        let (main, sub) = self.lines(t);
        let image_h = if self.icon.is_some() {
            self.icon_size + t.tile_gap
        } else {
            0.0
        };
        let w = self
            .icon
            .as_ref()
            .map_or(0.0, |_| self.icon_size)
            .max(lines_width(&main, t.tile, "semibold"))
            .max(lines_width(&sub, t.tile_sub, "regular"))
            + mul(2.0, t.pad);
        let line_count = count(main.len() + sub.len());
        let h = mul(2.0, t.pad)
            + image_h
            + mul(line_count, t.line_h)
            + if sub.is_empty() { 0.0 } else { 4.0 };
        (w, h)
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        report: ReportFn<'_>,
    ) -> Result<(), DiagramError> {
        let (x, y, w, h) = at;
        let (need_w, need_h) = self.measure(theme);
        if w + 0.5 < need_w || h + 0.5 < need_h {
            report(&format!(
                "resource {} needs {} x {} px but has {} x {} px",
                py_repr(&self.label),
                py_g(need_w),
                py_g(need_h),
                py_g(w),
                py_g(h)
            ));
        }
        if let Some(border) = &self.border {
            let dash = (border != "solid").then_some(border.as_str());
            canvas.rect(
                x,
                y,
                w,
                h,
                Paint::new(
                    &theme.paper,
                    Some(&theme.tile_stroke),
                    1.0,
                    theme.radius,
                    dash,
                    None,
                ),
            );
        }
        let mut cy = y + theme.pad;
        if let Some(icon) = &self.icon {
            canvas.image(
                &data_uri(icon)?,
                x + (w - self.icon_size) / 2.0,
                cy,
                self.icon_size,
                self.icon_size,
            );
            cy += self.icon_size + theme.tile_gap;
        }
        let (main, sub) = self.lines(theme);
        for line in &main {
            canvas.text_centered(
                line,
                x + w / 2.0,
                cy + theme.line_h / 2.0,
                theme.tile,
                "semibold",
                &theme.ink,
            );
            cy += theme.line_h;
        }
        if !sub.is_empty() {
            cy += 4.0;
        }
        for line in &sub {
            canvas.text_centered(
                line,
                x + w / 2.0,
                cy + theme.line_h / 2.0,
                theme.tile_sub,
                "regular",
                &theme.muted,
            );
            cy += theme.line_h;
        }
        Ok(())
    }

    fn ports(&self, side: &str, w: f64, h: f64, t: &Theme) -> (f64, f64) {
        if self.icon.is_some() && matches!(side, "left" | "right") {
            let at = t.pad + self.icon_size / 2.0;
            return (at, at);
        }
        default_ports(side, w, h)
    }

    fn anchor(&self, side: &str, offset: f64, w: f64, h: f64, t: &Theme) -> (f64, f64) {
        if self.icon.is_some() && self.border.is_none() {
            if matches!(side, "left" | "right") {
                let span = w + if side == "left" {
                    -self.icon_size
                } else {
                    self.icon_size
                };
                let x = span / 2.0;
                return (x, offset);
            }
            if side == "top" {
                return (offset, t.pad);
            }
        }
        default_anchor(side, offset, w, h)
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub label: String,
    pub sub: Option<String>,
    pub status: Option<String>,
    pub icon: Option<String>,
    pub width: Option<f64>,
    pub name: Option<String>,
}

impl Tile {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            sub: None,
            status: None,
            icon: None,
            width: None,
            name: None,
        }
    }

    #[must_use]
    pub fn subtitle(mut self, sub: impl Into<String>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// # Errors
    /// Returns an error when the icon file does not exist.
    pub fn icon(mut self, icon: impl Into<String>) -> Result<Self, DiagramError> {
        let icon = icon.into();
        if !Path::new(&icon).exists() {
            return Err(DiagramError::new(format!(
                "tile {}: icon file not found: {icon}",
                py_repr(&self.label)
            )));
        }
        self.icon = Some(icon);
        Ok(self)
    }

    fn lines(&self, t: &Theme) -> (Vec<String>, Vec<String>) {
        let room = t.tile_max_w - mul(2.0, t.pad);
        let icon = if self.icon.is_some() { ICON + 6.0 } else { 0.0 };
        let main = wrap(&self.label, t.tile, Some(room - icon), "semibold");
        let sub = self
            .sub
            .as_ref()
            .map_or_default(|s| wrap(s, t.tile_sub, Some(room), "regular"));
        (main, sub)
    }

    #[must_use]
    pub fn measure(&self, t: &Theme) -> (f64, f64) {
        let (main, sub) = self.lines(t);
        let icon = if self.icon.is_some() { ICON + 6.0 } else { 0.0 };
        let w = lines_width(&main, t.tile, "semibold").max(0.0) + icon;
        let text_w = w.max(lines_width(&sub, t.tile_sub, "regular")) + mul(2.0, t.pad);
        let h = t.tile_h
            + mul(count(main.len().saturating_sub(1)), t.line_h)
            + mul(count(sub.len()), t.line_h - 1.5);
        (text_w.max(self.width.unwrap_or(0.0)) + t.depth, h + t.depth)
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        report: ReportFn<'_>,
    ) -> Result<(), DiagramError> {
        let (x, mut y, mut w, mut h) = at;
        let st = theme.status(self.status.as_deref())?;
        let (fill, stroke, dash) = st.map_or(
            (theme.tile_fill.as_str(), theme.tile_stroke.as_str(), None),
            |s| (s.fill.as_str(), s.stroke.as_str(), s.dash.as_deref()),
        );
        let need = self.measure(theme).0;
        if need > w + 0.5 {
            report(&format!(
                "tile {} needs {} px but has {} px",
                py_repr(&self.label),
                py_fixed0(need),
                py_fixed0(w)
            ));
        }
        let depth = theme.depth;
        if depth != 0.0 {
            let top = format!(
                "M{} {}L{} {}H{}L{} {}Z",
                fmt(x),
                fmt(y + depth),
                fmt(x + depth),
                fmt(y),
                fmt(x + w),
                fmt(x + w - depth),
                fmt(y + depth)
            );
            let side = format!(
                "M{} {}L{} {}V{}L{} {}Z",
                fmt(x + w - depth),
                fmt(y + depth),
                fmt(x + w),
                fmt(y),
                fmt(y + h - depth),
                fmt(x + w - depth),
                fmt(y + h)
            );
            canvas.path(&top, &shade(fill, 0.05), Some(stroke), 0.8, dash, None);
            canvas.path(&side, &shade(fill, 0.15), Some(stroke), 0.8, dash, None);
            y += depth;
            w -= depth;
            h -= depth;
        }
        canvas.rect(
            x,
            y,
            w,
            h,
            Paint::new(
                fill,
                Some(stroke),
                1.1,
                if depth == 0.0 { 4.0 } else { 2.0 },
                dash,
                None,
            ),
        );
        let (main, sub) = self.lines(theme);
        let mut steps = Vec::new();
        if !main.is_empty() {
            steps.extend(std::iter::repeat_n(theme.line_h, main.len() - 1));
        }
        steps.extend(std::iter::repeat_n(theme.line_h - 1.5, sub.len()));
        let mut centres = vec![y + h / 2.0 - steps.iter().sum::<f64>() / 2.0];
        for step in steps {
            let next = centres.last().copied().unwrap_or(0.0) + step;
            centres.push(next);
        }
        let icon = if self.icon.is_some() { ICON + 6.0 } else { 0.0 };
        for (i, (line, &cy)) in main.iter().zip(&centres).enumerate() {
            let lw = text_width(line, theme.tile, "semibold") + if i == 0 { icon } else { 0.0 };
            let mut lx = x + w / 2.0 - lw / 2.0;
            if i == 0
                && let Some(path) = &self.icon
            {
                canvas.image(&data_uri(path)?, lx, cy - ICON / 2.0, ICON, ICON);
                lx += icon;
            }
            canvas.text(
                line,
                lx,
                cy + cap_height(theme.tile, "semibold") / 2.0,
                Letter::new(theme.tile, "semibold", &theme.ink, "start"),
            );
        }
        for (line, cy) in sub.iter().zip(centres.iter().skip(main.len())) {
            canvas.text_centered(
                line,
                x + w / 2.0,
                *cy,
                theme.tile_sub,
                "regular",
                &theme.muted,
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Tiles {
    pub items: Vec<Tile>,
    pub cols: usize,
    pub label: Option<String>,
}

impl Tiles {
    /// # Errors
    /// Returns an error when `items` is empty.
    pub fn new(
        items: impl IntoIterator<Item = Tile>,
        cols: Option<usize>,
        label: Option<String>,
    ) -> Result<Self, DiagramError> {
        let items: Vec<Tile> = items.into_iter().collect();
        if items.is_empty() {
            return Err(DiagramError::new("Tiles needs at least one item"));
        }
        let cols = 1.max((cols.unwrap_or(3)).min(items.len()));
        Ok(Self { items, cols, label })
    }

    fn cell(&self, t: &Theme) -> (f64, f64) {
        let sizes: Vec<_> = self.items.iter().map(|i| i.measure(t)).collect();
        (
            sizes.iter().map(|s| s.0).fold(0.0, f64::max),
            sizes.iter().map(|s| s.1).fold(0.0, f64::max),
        )
    }

    fn label_h(&self, t: &Theme) -> f64 {
        if self.label.is_some() {
            t.line_h + 4.0
        } else {
            0.0
        }
    }

    fn measure(&self, t: &Theme) -> (f64, f64) {
        let (cw, ch) = self.cell(t);
        let rows = self.items.len().div_ceil(self.cols);
        let mut w = mul(count(self.cols), cw) + mul(count(self.cols - 1), t.tile_gap);
        if let Some(label) = &self.label {
            w = w.max(text_width(label, t.small, "semibold"));
        }
        (
            w,
            self.label_h(t) + mul(count(rows), ch) + mul(count(rows - 1), t.tile_gap),
        )
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        report: ReportFn<'_>,
    ) -> Result<(), DiagramError> {
        let (x, y, w, _h) = at;
        let (_, ch) = self.cell(theme);
        let g = theme.tile_gap;
        let cw = (w - mul(count(self.cols - 1), g)) / count(self.cols);
        if let Some(label) = &self.label {
            canvas.text(
                label,
                x,
                y + 11.0,
                Letter::new(theme.small, "semibold", &theme.muted, "start"),
            );
        }
        let mut top = y + self.label_h(theme);
        for row in self.items.chunks(self.cols) {
            let row_w = mul(count(row.len()), cw) + mul(count(row.len() - 1), g);
            let mut rx = x + (w - row_w) / 2.0;
            for item in row {
                item.draw(canvas, (rx, top, cw, ch), theme, report)?;
                rx += cw + g;
            }
            top += ch + g;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,
    pub size: Option<f64>,
    pub weight: String,
    pub color: Option<String>,
    pub max_width: f64,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: None,
            weight: "regular".to_owned(),
            color: None,
            max_width: 320.0,
        }
    }

    fn lines(&self, t: &Theme) -> Vec<String> {
        wrap(
            &self.text,
            self.size.unwrap_or(t.body),
            Some(self.max_width),
            &self.weight,
        )
    }

    fn measure(&self, t: &Theme) -> (f64, f64) {
        let lines = self.lines(t);
        let size = self.size.unwrap_or(t.body);
        (
            lines_width(&lines, size, &self.weight),
            mul(count(lines.len()), t.line_h),
        )
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        _report: ReportFn<'_>,
    ) {
        let (x, y, _w, _h) = at;
        let size = self.size.unwrap_or(theme.body);
        let color = self.color.as_deref().unwrap_or(&theme.ink);
        for (i, line) in self.lines(theme).iter().enumerate() {
            let i = count(i);
            canvas.text(
                line,
                x,
                y + mul(theme.line_h, i)
                    + theme.line_h / 2.0
                    + cap_height(size, &self.weight) / 2.0,
                Letter::new(size, &self.weight, color, "start"),
            );
        }
    }
}

#[derive(Clone, Debug)]
pub struct Card {
    pub title: String,
    pub rows: Vec<(String, String)>,
    pub intro: Option<String>,
    pub max_width: f64,
}

impl Card {
    pub fn new(
        title: impl Into<String>,
        rows: Vec<(String, String)>,
        intro: Option<String>,
    ) -> Self {
        Self {
            title: title.into(),
            rows,
            intro,
            max_width: 380.0,
        }
    }

    fn layout(&self, t: &Theme) -> (f64, Vec<Vec<String>>, Vec<String>) {
        let key_w = self
            .rows
            .iter()
            .map(|(k, _)| text_width(k, t.body, "semibold"))
            .fold(0.0, f64::max);
        let room = self.max_width - mul(2.0, t.pad);
        let values = self
            .rows
            .iter()
            .map(|(_, v)| wrap(v, t.body, Some(room - key_w - 14.0), "regular"))
            .collect();
        let intro = self
            .intro
            .as_ref()
            .map_or_default(|s| wrap(s, t.body, Some(room), "regular"));
        (key_w, values, intro)
    }

    fn measure(&self, t: &Theme) -> (f64, f64) {
        let (key_w, values, intro) = self.layout(t);
        let value_w = values
            .iter()
            .map(|v| lines_width(v, t.body, "regular"))
            .fold(0.0, f64::max);
        let inner = text_width(&self.title, t.tile, "semibold")
            .max(lines_width(&intro, t.body, "regular"))
            .max(key_w + 14.0 + value_w);
        let lines = 1 + intro.len() + values.iter().map(Vec::len).sum::<usize>();
        let intro_extra = if intro.is_empty() { 0.0 } else { 4.0 };
        (
            inner + mul(2.0, t.pad),
            mul(2.0, t.pad) + mul(count(lines), t.line_h) + intro_extra + 4.0,
        )
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        _report: ReportFn<'_>,
    ) {
        let (x, y, w, h) = at;
        let (key_w, values, intro) = self.layout(theme);
        canvas.rect(
            x,
            y,
            w,
            h,
            Paint::new(
                &theme.tile_fill,
                Some(&theme.panel_stroke),
                1.0,
                4.0,
                None,
                None,
            ),
        );
        let base = |row: f64| {
            y + theme.pad
                + mul(row, theme.line_h)
                + theme.line_h / 2.0
                + cap_height(theme.body, "regular") / 2.0
        };
        canvas.text(
            &self.title,
            x + theme.pad,
            base(0.0),
            Letter::new(theme.tile, "semibold", &theme.ink, "start"),
        );
        let mut row = 1.0;
        let mut extra = 4.0;
        for line in &intro {
            canvas.text(
                line,
                x + theme.pad,
                base(row) + 2.0,
                Letter::new(theme.body, "regular", &theme.muted, "start"),
            );
            row += 1.0;
        }
        if !intro.is_empty() {
            extra += 4.0;
        }
        for ((key, _), lines) in self.rows.iter().zip(values) {
            canvas.text(
                key,
                x + theme.pad,
                base(row) + extra,
                Letter::new(theme.body, "semibold", &theme.ink, "start"),
            );
            for line in lines {
                canvas.text(
                    &line,
                    x + theme.pad + key_w + 14.0,
                    base(row) + extra,
                    Letter::new(theme.body, "regular", &theme.ink, "start"),
                );
                row += 1.0;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Note {
    pub title: String,
    pub lines: Vec<String>,
    pub max_width: f64,
}

impl Note {
    pub fn new(
        title: impl Into<String>,
        lines: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            title: title.into(),
            lines: lines.into_iter().map(Into::into).collect(),
            max_width: 330.0,
        }
    }

    fn wrapped(&self, t: &Theme) -> Vec<String> {
        self.lines
            .iter()
            .flat_map(|line| {
                wrap(
                    line,
                    t.body,
                    Some(self.max_width - mul(2.0, t.pad)),
                    "regular",
                )
            })
            .collect()
    }

    #[must_use]
    pub fn measure(&self, t: &Theme) -> (f64, f64) {
        let lines = self.wrapped(t);
        let w = text_width(&self.title, 13.0, "semibold")
            .max(lines_width(&lines, t.body, "regular"))
            + mul(2.0, t.pad);
        (
            w,
            mul(2.0, t.pad) + 18.0 + mul(count(lines.len()), t.line_h),
        )
    }

    pub fn draw(&self, canvas: &mut Canvas, at: (f64, f64, f64, f64), theme: &Theme) {
        let (x, y, w, h) = at;
        canvas.rect(
            x,
            y,
            w,
            h,
            Paint::new(
                &theme.note_fill,
                Some(&theme.note_stroke),
                1.2,
                theme.radius,
                None,
                None,
            ),
        );
        canvas.text(
            &self.title,
            x + theme.pad,
            y + theme.pad + 12.0,
            Letter::new(13.0, "semibold", &theme.ink, "start"),
        );
        for (index, line) in self.wrapped(theme).iter().enumerate() {
            let index = count(index);
            canvas.text(
                line,
                x + theme.pad,
                y + theme.pad + 18.0 + mul(theme.line_h, index) + 11.0,
                Letter::new(theme.body, "regular", &theme.muted, "start"),
            );
        }
    }
}

#[derive(Clone, Debug)]
pub struct Graph {
    pub nodes: IndexMap<String, String>,
    pub links: Vec<(String, String, Option<String>)>,
    pub at: IndexMap<String, (f64, f64)>,
    pub spacing: (f64, f64),
}

impl Graph {
    /// # Errors
    /// Returns an error when a node has no position or a link names an unknown node.
    pub fn new(
        nodes: IndexMap<String, String>,
        links: Vec<(String, String, Option<String>)>,
        at: IndexMap<String, (f64, f64)>,
    ) -> Result<Self, DiagramError> {
        let missing: Vec<_> = nodes
            .keys()
            .filter(|k| !at.contains_key(k.as_str()))
            .cloned()
            .collect();
        if !missing.is_empty() {
            return Err(DiagramError::new(format!(
                "Graph: no position given for {}",
                missing.join(", ")
            )));
        }
        for (a, b, _) in &links {
            for k in [a, b] {
                if !nodes.contains_key(k) {
                    return Err(DiagramError::new(format!(
                        "Graph: link names unknown node {}",
                        py_repr(k)
                    )));
                }
            }
        }
        Ok(Self {
            nodes,
            links,
            at,
            spacing: (132.0, 58.0),
        })
    }

    fn node_w(&self, t: &Theme, key: &str) -> Option<f64> {
        let label = self.nodes.get(key)?;
        Some(text_width(label, t.body, "semibold") + 26.0)
    }

    fn boxes(&self, t: &Theme) -> IndexMap<String, (f64, f64, f64, f64)> {
        let (sx, sy) = self.spacing;
        let mut boxes = IndexMap::new();
        for key in self.nodes.keys() {
            let Some(&(col, row)) = self.at.get(key) else {
                continue;
            };
            let Some(w) = self.node_w(t, key) else {
                continue;
            };
            boxes.insert(
                key.clone(),
                (mul(col, sx) - w / 2.0, mul(row, sy) - 13.0, w, 26.0),
            );
        }
        boxes
    }

    fn extent(&self, t: &Theme) -> (f64, f64, f64, f64) {
        let boxes = self.boxes(t);
        let x0 = boxes.values().map(|b| b.0).fold(f64::INFINITY, f64::min);
        let y0 = boxes.values().map(|b| b.1).fold(f64::INFINITY, f64::min);
        let x1 = boxes
            .values()
            .map(|b| b.0 + b.2)
            .fold(f64::NEG_INFINITY, f64::max);
        let y1 = boxes
            .values()
            .map(|b| b.1 + b.3)
            .fold(f64::NEG_INFINITY, f64::max);
        (x0, y0, x1, y1)
    }

    fn measure(&self, t: &Theme) -> (f64, f64) {
        let (x0, y0, x1, y1) = self.extent(t);
        (x1 - x0 + 32.0, y1 - y0 + 32.0)
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        report: ReportFn<'_>,
    ) {
        let (x, y, w, h) = at;
        canvas.rect(
            x,
            y,
            w,
            h,
            Paint::new(
                &theme.graph_fill,
                Some(&theme.panel_stroke),
                0.8,
                4.0,
                None,
                None,
            ),
        );
        let mut gx = x + 18.0;
        let mut column = 0;
        while column < 10_000 {
            if gx >= x + w - 1.0 {
                break;
            }
            canvas.line(
                (gx, y + 1.0),
                (gx, y + h - 1.0),
                &theme.graph_grid,
                0.6,
                None,
            );
            gx += 18.0;
            column += 1;
        }
        let mut gy = y + 18.0;
        let mut row = 0;
        while row < 10_000 {
            if gy >= y + h - 1.0 {
                break;
            }
            canvas.line(
                (x + 1.0, gy),
                (x + w - 1.0, gy),
                &theme.graph_grid,
                0.6,
                None,
            );
            gy += 18.0;
            row += 1;
        }
        let (x0, y0, x1, y1) = self.extent(theme);
        let ox = x + (w - (x1 - x0)) / 2.0 - x0;
        let oy = y + (h - (y1 - y0)) / 2.0 - y0;
        let boxes: IndexMap<_, _> = self
            .boxes(theme)
            .into_iter()
            .map(|(k, (bx, by, bw, bh))| (k, (bx + ox, by + oy, bw, bh)))
            .collect();
        let centres: IndexMap<_, _> = boxes
            .iter()
            .map(|(k, b)| (k.clone(), (b.0 + b.2 / 2.0, b.1 + b.3 / 2.0)))
            .collect();
        let labels = self.draw_edges(canvas, theme, report, &boxes, &centres);
        self.draw_nodes_and_labels(canvas, theme, report, &boxes, &labels);
    }

    fn draw_edges(
        &self,
        canvas: &mut Canvas,
        theme: &Theme,
        report: ReportFn<'_>,
        boxes: &IndexMap<String, (f64, f64, f64, f64)>,
        centres: &IndexMap<String, (f64, f64)>,
    ) -> Vec<(String, (f64, f64, f64, f64))> {
        let mut labels = Vec::new();
        for (from_id, to_id, label) in &self.links {
            let Some(&(ax, ay)) = centres.get(from_id) else {
                continue;
            };
            let Some(&(bx, by)) = centres.get(to_id) else {
                continue;
            };
            let Some(from_box) = boxes.get(from_id).copied() else {
                continue;
            };
            let Some(to_box) = boxes.get(to_id).copied() else {
                continue;
            };
            let from_pt = clip(from_box, (ax, ay), (bx, by));
            let to_pt = clip(to_box, (bx, by), (ax, ay));
            canvas.line(
                (from_pt.0, from_pt.1),
                (to_pt.0, to_pt.1),
                &theme.graph_line,
                1.0,
                None,
            );
            for (key, box_) in boxes {
                if key != from_id
                    && key != to_id
                    && segment_hits(*box_, from_pt, to_pt)
                    && let (Some(from_name), Some(to_name), Some(through)) = (
                        self.nodes.get(from_id),
                        self.nodes.get(to_id),
                        self.nodes.get(key),
                    )
                {
                    report(&format!(
                        "graph: the line {} to {} runs through {}",
                        py_repr(from_name),
                        py_repr(to_name),
                        py_repr(through)
                    ));
                }
            }
            if let Some(label) = label {
                let lw = text_width(label, theme.small, "regular") + 12.0;
                let mid_x = from_pt.0 + to_pt.0;
                let mid_y = from_pt.1 + to_pt.1;
                let mx = mid_x / 2.0;
                let my = mid_y / 2.0;
                labels.push((label.clone(), (mx - lw / 2.0, my - 8.0, lw, 16.0)));
            }
        }
        labels
    }

    fn draw_nodes_and_labels(
        &self,
        canvas: &mut Canvas,
        theme: &Theme,
        report: ReportFn<'_>,
        boxes: &IndexMap<String, (f64, f64, f64, f64)>,
        labels: &[GraphLabel],
    ) {
        for (key, (bx, by, bw, bh)) in boxes {
            canvas.rect(
                *bx,
                *by,
                *bw,
                *bh,
                Paint::new(
                    &theme.tile_fill,
                    Some(&theme.ink),
                    1.0,
                    bh / 2.0,
                    None,
                    None,
                ),
            );
            if let Some(label) = self.nodes.get(key) {
                canvas.text_centered(
                    label,
                    bx + bw / 2.0,
                    by + bh / 2.0,
                    theme.body,
                    "semibold",
                    &theme.ink,
                );
            }
        }
        for (index, (label, box_)) in labels.iter().enumerate() {
            for (key, nb) in boxes {
                if overlap(*box_, *nb, 2.0)
                    && let Some(name) = self.nodes.get(key)
                {
                    report(&format!(
                        "graph: the label {} touches the node {}",
                        py_repr(label),
                        py_repr(name)
                    ));
                }
            }
            for (other, ob) in labels.iter().skip(index + 1) {
                if overlap(*box_, *ob, 2.0) {
                    report(&format!(
                        "graph: the labels {} and {} touch",
                        py_repr(label),
                        py_repr(other)
                    ));
                }
            }
            canvas.rect(
                box_.0,
                box_.1,
                box_.2,
                box_.3,
                Paint::new(&theme.paper, Some(&theme.graph_line), 0.7, 8.0, None, None),
            );
            canvas.text_centered(
                label,
                box_.0 + box_.2 / 2.0,
                box_.1 + 8.0,
                theme.small,
                "regular",
                &theme.muted,
            );
        }
    }
}

fn clip(box_: (f64, f64, f64, f64), inside: (f64, f64), toward: (f64, f64)) -> (f64, f64) {
    let dx = toward.0 - inside.0;
    let dy = toward.1 - inside.1;
    let hw = box_.2 / 2.0 + 2.0;
    let hh = box_.3 / 2.0 + 2.0;
    let sx = if dx == 0.0 {
        f64::INFINITY
    } else {
        hw / dx.abs()
    };
    let sy = if dy == 0.0 {
        f64::INFINITY
    } else {
        hh / dy.abs()
    };
    let s = sx.min(sy);
    (inside.0 + mul(dx, s), inside.1 + mul(dy, s))
}

fn overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64), pad: f64) -> bool {
    a.0 < b.0 + b.2 + pad && b.0 < a.0 + a.2 + pad && a.1 < b.1 + b.3 + pad && b.1 < a.1 + a.3 + pad
}

fn segment_hits(box_: (f64, f64, f64, f64), p: (f64, f64), q: (f64, f64)) -> bool {
    for step in 1..40 {
        let portion = f64::from(step) / 40.0;
        let point_x = p.0 + mul(q.0 - p.0, portion);
        let point_y = p.1 + mul(q.1 - p.1, portion);
        if box_.0 + 1.0 < point_x
            && point_x < box_.0 + box_.2 - 1.0
            && box_.1 + 1.0 < point_y
            && point_y < box_.1 + box_.3 - 1.0
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Debug)]
pub struct Row {
    pub items: Vec<Block>,
    pub gap: Option<f64>,
}

impl Row {
    pub fn new(items: impl IntoIterator<Item = Block>, gap: Option<f64>) -> Self {
        Self {
            items: items.into_iter().collect(),
            gap,
        }
    }

    fn gap(&self, t: &Theme) -> f64 {
        self.gap.unwrap_or(t.section_gap)
    }

    fn measure(&self, t: &Theme) -> Result<(f64, f64), DiagramError> {
        let g = self.gap(t);
        let mut sizes = Vec::new();
        for item in &self.items {
            sizes.push(item.measure(t)?);
        }
        let w =
            sizes.iter().map(|s| s.0).sum::<f64>() + mul(g, count(sizes.len().saturating_sub(1)));
        let h = sizes.iter().map(|s| s.1).fold(0.0, f64::max);
        Ok((w, h))
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        report: ReportFn<'_>,
    ) -> Result<(), DiagramError> {
        let (x, y, w, _h) = at;
        let g = self.gap(theme);
        let mut sizes = Vec::new();
        for item in &self.items {
            sizes.push(item.measure(theme)?);
        }
        let extra = w
            - (sizes.iter().map(|s| s.0).sum::<f64>()
                + mul(g, count(sizes.len().saturating_sub(1))));
        let mut cx = x;
        for (n, (item, (mut iw, ih))) in self.items.iter().zip(sizes).enumerate() {
            if n + 1 == self.items.len() {
                iw += extra;
            }
            item.draw(canvas, (cx, y, iw, ih), theme, report)?;
            cx += iw + g;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Panel {
    pub title: String,
    pub body: Vec<Block>,
    pub sub: Option<String>,
    pub status: Option<String>,
    pub step: Option<i64>,
    pub footer: Option<String>,
    pub tone: String,
    pub width: Option<f64>,
    pub name: String,
}

impl Panel {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            name: title.clone(),
            title,
            body: Vec::new(),
            sub: None,
            status: None,
            step: None,
            footer: None,
            tone: "default".to_owned(),
            width: None,
        }
    }

    #[must_use]
    pub fn section(mut self, block: Block) -> Self {
        self.body.push(block);
        self
    }

    #[must_use]
    pub fn subtitle(mut self, sub: impl Into<String>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    #[must_use]
    pub const fn step(mut self, step: i64) -> Self {
        self.step = Some(step);
        self
    }

    #[must_use]
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    #[must_use]
    pub fn tone(mut self, tone: impl Into<String>) -> Self {
        self.tone = tone.into();
        self
    }

    #[must_use]
    pub const fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    const fn head_h(&self, t: &Theme) -> f64 {
        if self.sub.is_some() {
            t.header_h_sub
        } else {
            t.header_h
        }
    }

    const fn badge_w(&self) -> f64 {
        if self.step.is_some() { 30.0 } else { 0.0 }
    }

    fn tag(&self, t: &Theme) -> Result<(Option<crate::theme::Status>, f64), DiagramError> {
        let Some(st) = t.status(self.status.as_deref())? else {
            return Ok((None, 0.0));
        };
        let w = text_width(&st.label, t.small, "semibold") + 16.0;
        Ok((Some(st.clone()), w))
    }

    fn footer_lines(&self) -> Vec<String> {
        self.footer
            .as_ref()
            .map_or_default(|f| f.split('\n').map(str::to_owned).collect())
    }

    fn footer_h(&self, t: &Theme) -> f64 {
        let n = self.footer_lines().len();
        if n == 0 {
            0.0
        } else {
            8.0 + mul(count(n), t.line_h) + 6.0
        }
    }

    /// # Errors
    /// Returns an error when the panel status or a nested block cannot be measured.
    pub fn measure(&self, t: &Theme) -> Result<(f64, f64), DiagramError> {
        let (_, tag_w) = self.tag(t)?;
        let head = t.pad
            + self.badge_w()
            + text_width(&self.title, t.panel_title, "semibold")
            + if tag_w == 0.0 { 0.0 } else { 12.0 + tag_w }
            + t.pad;
        let sub = self.sub.as_ref().map_or(0.0, |s| {
            t.pad + self.badge_w() + text_width(s, t.panel_sub, "regular") + t.pad
        });
        let mut sizes = Vec::new();
        for block in &self.body {
            sizes.push(block.measure(t)?);
        }
        let body_w = sizes.iter().map(|s| s.0).fold(0.0, f64::max);
        let body_h = sizes.iter().map(|s| s.1).sum::<f64>()
            + mul(t.section_gap, count(sizes.len().saturating_sub(1)));
        let foot = lines_width(&self.footer_lines(), t.small, "regular") + mul(2.0, t.pad) + 4.0;
        let w = head
            .max(sub)
            .max(body_w + mul(2.0, t.pad))
            .max(foot)
            .max(t.panel_min_w)
            .max(self.width.unwrap_or(0.0));
        let h = t.lid
            + self.head_h(t)
            + if sizes.is_empty() {
                t.pad
            } else {
                mul(2.0, t.pad) + body_h
            }
            + self.footer_h(t);
        Ok((w, h))
    }

    fn draw_heading(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64),
        theme: &Theme,
        tone: &str,
        hh: f64,
    ) -> Result<(), DiagramError> {
        let (x, y, w) = at;
        if theme.lid == 0.0 {
            canvas.path(
                &top_rounded((x, y, w, hh), theme.radius),
                tone,
                None,
                1.0,
                None,
                None,
            );
        } else {
            canvas.rect(x, y, w, hh, Paint::new(tone, None, 1.0, 0.0, None, None));
        }
        let (title_color, sub_color) = theme.header_colors(tone)?;
        let mut tx = x + theme.pad;
        let title_cy = y + if self.sub.is_some() { 18.0 } else { hh / 2.0 };
        if let Some(step) = self.step {
            canvas.circle(
                tx + 11.0,
                title_cy,
                10.5,
                theme.badge_fill.as_deref().unwrap_or(&theme.paper),
                None,
                1.0,
            );
            canvas.text_centered(
                &step.to_string(),
                tx + 11.0,
                title_cy,
                11.5,
                "bold",
                theme.badge_text.as_deref().unwrap_or(tone),
            );
            tx += self.badge_w();
        }
        canvas.text_centered(
            &self.title,
            tx + text_width(&self.title, theme.panel_title, "semibold") / 2.0,
            title_cy,
            theme.panel_title,
            "semibold",
            &title_color,
        );
        if let Some(sub) = &self.sub {
            canvas.text(
                sub,
                tx,
                y + 36.0 + cap_height(theme.panel_sub, "regular") / 2.0,
                Letter::new(theme.panel_sub, "regular", &sub_color, "start"),
            );
        }
        if let (Some(tag), tag_w) = self.tag(theme)? {
            canvas.rect(
                x + w - theme.pad - tag_w,
                title_cy - 9.0,
                tag_w,
                18.0,
                Paint::new(&theme.paper, None, 1.0, 9.0, None, None),
            );
            canvas.text_centered(
                &tag.label,
                x + w - theme.pad - tag_w / 2.0,
                title_cy,
                theme.small,
                "semibold",
                &tag.stroke,
            );
        }
        Ok(())
    }

    fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        report: ReportFn<'_>,
    ) -> Result<(), DiagramError> {
        let (x, mut y, w, mut h) = at;
        let st = theme.status(self.status.as_deref())?;
        let hh = self.head_h(theme);
        let tone = theme.tone(&self.tone)?;
        let fill = st
            .as_ref()
            .map_or(theme.panel_fill.as_str(), |s| s.fill.as_str());
        let stroke = st
            .as_ref()
            .map_or(theme.panel_stroke.as_str(), |s| s.stroke.as_str());
        let width = if st.is_some() { 1.3 } else { 1.0 };
        let dash = st.as_ref().and_then(|s| s.dash.as_deref());
        (y, h) = panel_frame(
            canvas,
            (x, y, w, h),
            PanelFace {
                theme,
                tone: &tone,
                fill,
                stroke,
                width,
                dash,
            },
        );
        self.draw_heading(canvas, (x, y, w), theme, &tone, hh)?;
        let mut cy = y + hh + theme.pad;
        for block in &self.body {
            let (_, bh) = block.measure(theme)?;
            block.draw(
                canvas,
                (x + theme.pad, cy, w - mul(2.0, theme.pad), bh),
                theme,
                report,
            )?;
            cy += bh + theme.section_gap;
        }
        let lines = self.footer_lines();
        if !lines.is_empty() {
            let fy = y + h - self.footer_h(theme);
            canvas.line(
                (x + theme.pad, fy),
                (x + w - theme.pad, fy),
                &theme.panel_stroke,
                0.8,
                None,
            );
            for (i, line) in lines.iter().enumerate() {
                let base = fy
                    + 8.0
                    + mul(theme.line_h, count(i))
                    + theme.line_h / 2.0
                    + cap_height(theme.small, "regular") / 2.0;
                let (head, rest) = line
                    .split_once(':')
                    .map_or((line.as_str(), None), |(h, r)| (h, Some(r)));
                if rest.is_some() && head.chars().count() < 40 {
                    let hw = canvas.text(
                        &format!("{head}:"),
                        x + theme.pad,
                        base,
                        Letter::new(theme.small, "semibold", &theme.ink, "start"),
                    );
                    canvas.text(
                        rest.unwrap_or(""),
                        x + theme.pad + hw,
                        base,
                        Letter::new(theme.small, "regular", &theme.muted, "start"),
                    );
                } else {
                    canvas.text(
                        line,
                        x + theme.pad,
                        base,
                        Letter::new(theme.small, "regular", &theme.muted, "start"),
                    );
                }
            }
        }
        Ok(())
    }

    fn ports(&self, side: &str, w: f64, h: f64, t: &Theme) -> (f64, f64) {
        if matches!(side, "left" | "right") {
            (t.lid + self.head_h(t) + 8.0, h - 10.0)
        } else {
            (18.0, w - 18.0)
        }
    }
}

#[derive(Clone, Debug)]
pub enum Block {
    Resource(Resource),
    Tile(Tile),
    Tiles(Tiles),
    Text(Text),
    Card(Card),
    Note(Note),
    Graph(Graph),
    Row(Row),
    Panel(Panel),
}

impl From<Resource> for Block {
    fn from(v: Resource) -> Self {
        Self::Resource(v)
    }
}
impl From<Tile> for Block {
    fn from(v: Tile) -> Self {
        Self::Tile(v)
    }
}
impl From<Tiles> for Block {
    fn from(v: Tiles) -> Self {
        Self::Tiles(v)
    }
}
impl From<Text> for Block {
    fn from(v: Text) -> Self {
        Self::Text(v)
    }
}
impl From<Card> for Block {
    fn from(v: Card) -> Self {
        Self::Card(v)
    }
}
impl From<Note> for Block {
    fn from(v: Note) -> Self {
        Self::Note(v)
    }
}
impl From<Graph> for Block {
    fn from(v: Graph) -> Self {
        Self::Graph(v)
    }
}
impl From<Row> for Block {
    fn from(v: Row) -> Self {
        Self::Row(v)
    }
}
impl From<Panel> for Block {
    fn from(v: Panel) -> Self {
        Self::Panel(v)
    }
}

impl Block {
    /// # Errors
    /// Returns an error when a row or panel cannot be measured.
    pub fn measure(&self, t: &Theme) -> Result<(f64, f64), DiagramError> {
        match self {
            Self::Resource(v) => Ok(v.measure(t)),
            Self::Tile(v) => Ok(v.measure(t)),
            Self::Tiles(v) => Ok(v.measure(t)),
            Self::Text(v) => Ok(v.measure(t)),
            Self::Card(v) => Ok(v.measure(t)),
            Self::Note(v) => Ok(v.measure(t)),
            Self::Graph(v) => Ok(v.measure(t)),
            Self::Row(v) => v.measure(t),
            Self::Panel(v) => v.measure(t),
        }
    }

    /// # Errors
    /// Returns an error when a block's icon or status cannot be drawn.
    pub fn draw(
        &self,
        canvas: &mut Canvas,
        at: (f64, f64, f64, f64),
        theme: &Theme,
        report: ReportFn<'_>,
    ) -> Result<(), DiagramError> {
        match self {
            Self::Resource(block) => block.draw(canvas, at, theme, report),
            Self::Tile(block) => block.draw(canvas, at, theme, report),
            Self::Tiles(block) => block.draw(canvas, at, theme, report),
            Self::Text(block) => {
                block.draw(canvas, at, theme, report);
                Ok(())
            }
            Self::Card(block) => {
                block.draw(canvas, at, theme, report);
                Ok(())
            }
            Self::Note(block) => {
                block.draw(canvas, at, theme);
                Ok(())
            }
            Self::Graph(block) => {
                block.draw(canvas, at, theme, report);
                Ok(())
            }
            Self::Row(block) => block.draw(canvas, at, theme, report),
            Self::Panel(block) => block.draw(canvas, at, theme, report),
        }
    }

    #[must_use]
    pub fn statuses(&self) -> BTreeSet<String> {
        match self {
            Self::Tile(v) => BTreeSet::from([v.status.clone().unwrap_or_default()]),
            Self::Tiles(v) => v
                .items
                .iter()
                .filter_map(|i| i.status.clone().map_or_else(|| Some(String::new()), Some))
                .collect(),
            Self::Row(v) => v.items.iter().flat_map(Self::statuses).collect(),
            Self::Panel(v) => {
                let mut found = BTreeSet::new();
                if let Some(status) = &v.status {
                    found.insert(status.clone());
                }
                for block in &v.body {
                    found.extend(block.statuses());
                }
                found
            }
            Self::Resource(_) | Self::Text(_) | Self::Card(_) | Self::Note(_) | Self::Graph(_) => {
                BTreeSet::new()
            }
        }
    }

    #[must_use]
    pub fn ports(&self, side: &str, w: f64, h: f64, t: &Theme) -> (f64, f64) {
        match self {
            Self::Resource(v) => v.ports(side, w, h, t),
            Self::Panel(v) => v.ports(side, w, h, t),
            Self::Tile(_)
            | Self::Tiles(_)
            | Self::Text(_)
            | Self::Card(_)
            | Self::Note(_)
            | Self::Graph(_)
            | Self::Row(_) => default_ports(side, w, h),
        }
    }

    #[must_use]
    pub fn anchor(&self, side: &str, offset: f64, w: f64, h: f64, t: &Theme) -> (f64, f64) {
        match self {
            Self::Resource(v) => v.anchor(side, offset, w, h, t),
            Self::Tile(_)
            | Self::Tiles(_)
            | Self::Text(_)
            | Self::Card(_)
            | Self::Note(_)
            | Self::Graph(_)
            | Self::Row(_)
            | Self::Panel(_) => default_anchor(side, offset, w, h),
        }
    }

    #[must_use]
    pub fn explicit_name(&self) -> Option<&str> {
        match self {
            Self::Resource(v) => Some(&v.name),
            Self::Panel(v) => Some(&v.name),
            Self::Tile(v) => v.name.as_deref(),
            Self::Tiles(_)
            | Self::Text(_)
            | Self::Card(_)
            | Self::Note(_)
            | Self::Graph(_)
            | Self::Row(_) => None,
        }
    }

    #[must_use]
    pub fn title(&self) -> Option<&str> {
        match self {
            Self::Panel(v) => Some(&v.title),
            Self::Resource(_)
            | Self::Tile(_)
            | Self::Tiles(_)
            | Self::Text(_)
            | Self::Card(_)
            | Self::Note(_)
            | Self::Graph(_)
            | Self::Row(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PanelFace<'a> {
    pub(crate) theme: &'a Theme,
    pub(crate) tone: &'a str,
    pub(crate) fill: &'a str,
    pub(crate) stroke: &'a str,
    pub(crate) width: f64,
    pub(crate) dash: Option<&'a str>,
}

pub(crate) fn panel_frame(
    canvas: &mut Canvas,
    at: (f64, f64, f64, f64),
    face: PanelFace<'_>,
) -> (f64, f64) {
    let (x, mut y, w, mut h) = at;
    let PanelFace {
        theme,
        tone,
        fill,
        stroke,
        width,
        dash,
    } = face;
    let lid = theme.lid;
    if lid != 0.0 {
        let inset = (lid * 0.7).min(w / 4.0);
        let lid_path = format!(
            "M{} {}L{} {}H{}L{} {}Z",
            fmt(x),
            fmt(y + lid),
            fmt(x + inset),
            fmt(y),
            fmt(x + w - inset),
            fmt(x + w),
            fmt(y + lid)
        );
        canvas.path(
            &lid_path,
            &shade(tone, 0.16),
            Some(&shade(tone, 0.32)),
            0.8,
            None,
            None,
        );
        y += lid;
        h -= lid;
    }
    let radius = if lid == 0.0 { theme.radius } else { 2.0 };
    canvas.rect(
        x,
        y,
        w,
        h,
        Paint::new(fill, Some(stroke), width, radius, dash, None),
    );
    (y, h)
}

fn top_rounded(at: (f64, f64, f64, f64), radius: f64) -> String {
    let (x, y, w, h) = at;
    format!(
        "M{} {}V{}Q{} {} {} {}H{}Q{} {} {} {}V{}Z",
        fmt(x),
        fmt(y + h),
        fmt(y + radius),
        fmt(x),
        fmt(y),
        fmt(x + radius),
        fmt(y),
        fmt(x + w - radius),
        fmt(x + w),
        fmt(y),
        fmt(x + w),
        fmt(y + radius),
        fmt(y + h)
    )
}

fn default_ports(side: &str, w: f64, h: f64) -> (f64, f64) {
    let length = if matches!(side, "left" | "right") {
        h
    } else {
        w
    };
    (length * 0.2, length * 0.8)
}

fn default_anchor(side: &str, offset: f64, w: f64, h: f64) -> (f64, f64) {
    match side {
        "left" => (0.0, offset),
        "right" => (w, offset),
        "top" => (offset, 0.0),
        _ => (offset, h),
    }
}

fn data_uri(path: &str) -> Result<String, DiagramError> {
    let mut bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => crate::catalog::embedded_icon(path)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| DiagramError::new(format!("reading {path}: {err}")))?,
    };
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "svg" {
        bytes.retain(|byte| *byte != b'\r');
    }
    let kind = match ext.as_str() {
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };
    Ok(format!(
        "data:{kind};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

pub(crate) fn apply_statuses(
    block: &mut Block,
    statuses: &IndexMap<String, String>,
    found: &mut BTreeSet<String>,
) {
    match block {
        Block::Panel(panel) => {
            apply_named(
                &mut panel.status,
                [&panel.title, &panel.name],
                statuses,
                found,
            );
            for child in &mut panel.body {
                apply_statuses(child, statuses, found);
            }
        }
        Block::Tiles(tiles) => {
            for tile in &mut tiles.items {
                apply_named(
                    &mut tile.status,
                    [tile.label.as_str(), tile.name.as_deref().unwrap_or("")],
                    statuses,
                    found,
                );
            }
        }
        Block::Row(row) => {
            for child in &mut row.items {
                apply_statuses(child, statuses, found);
            }
        }
        Block::Tile(tile) => {
            apply_named(
                &mut tile.status,
                [tile.label.as_str(), tile.name.as_deref().unwrap_or("")],
                statuses,
                found,
            );
        }
        Block::Resource(_) | Block::Text(_) | Block::Card(_) | Block::Note(_) | Block::Graph(_) => {
        }
    }
}

fn apply_named(
    status: &mut Option<String>,
    names: [&str; 2],
    statuses: &IndexMap<String, String>,
    found: &mut BTreeSet<String>,
) {
    for name in names {
        if name.is_empty() {
            continue;
        }
        if let Some(value) = statuses.get(name) {
            *status = (value != "live").then(|| value.clone());
            found.insert(name.to_owned());
        }
    }
}

#[cfg(test)]
mod resource_fit {
    use super::Resource;
    use crate::svg::Canvas;
    use crate::theme::Theme;

    #[test]
    fn wrapped_labels_report_when_the_box_is_short() {
        let resource = Resource::new(
            "A long resource label that wraps",
            None,
            Some("A detailed resource description".to_owned()),
            64.0,
            140.0,
            None,
            None,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let theme = Theme::default();
        let (w, h) = resource.measure(&theme);
        let mut canvas = Canvas::new(w, h);
        let mut problems = Vec::new();
        resource
            .draw(
                &mut canvas,
                (0.0, 0.0, w - 1.0, h - 1.0),
                &theme,
                &mut |msg| problems.push(msg.to_owned()),
            )
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            problems.iter().any(|problem| problem.contains("needs")),
            "{problems:?}"
        );
        assert!(h > theme.line_h * 2.0, "{h}");
    }
}
