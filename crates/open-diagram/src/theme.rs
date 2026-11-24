use indexmap::IndexMap;

use crate::error::DiagramError;
use crate::num::{mul, py_channel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub label: String,
    pub stroke: String,
    pub fill: String,
    pub dash: Option<String>,
}

impl Status {
    pub fn new(
        label: impl Into<String>,
        stroke: impl Into<String>,
        fill: impl Into<String>,
        dash: Option<String>,
    ) -> Self {
        Self {
            label: label.into(),
            stroke: stroke.into(),
            fill: fill.into(),
            dash,
        }
    }
}

fn statuses() -> IndexMap<String, Status> {
    IndexMap::from([
        (
            "live".to_owned(),
            Status::new("Live", "#27323F", "#FFFFFF", None),
        ),
        (
            "building".to_owned(),
            Status::new(
                "Being built",
                "#B26B00",
                "#FFF5E5",
                Some("dashed".to_owned()),
            ),
        ),
        (
            "trial".to_owned(),
            Status::new("In trial", "#6B4FD8", "#F4F1FE", Some("dashed".to_owned())),
        ),
        (
            "planned".to_owned(),
            Status::new("Planned", "#5C636B", "#F1F2F4", Some("dotted".to_owned())),
        ),
        (
            "retiring".to_owned(),
            Status::new("Retiring", "#B42318", "#FEF0EE", None),
        ),
    ])
}

fn tones() -> IndexMap<String, String> {
    IndexMap::from([
        ("default".to_owned(), "#E2E5E9".to_owned()),
        ("neutral".to_owned(), "#E2E5E9".to_owned()),
        ("accent".to_owned(), "#FFE08A".to_owned()),
        ("dark".to_owned(), "#2B3036".to_owned()),
    ])
}

fn link_labels() -> IndexMap<String, String> {
    IndexMap::from([
        ("flow".to_owned(), "Data flows today".to_owned()),
        ("planned".to_owned(), "Connection to build".to_owned()),
    ])
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub ink: String,
    pub muted: String,
    pub faint: String,
    pub paper: String,
    pub line: String,
    pub panel_fill: String,
    pub panel_stroke: String,
    pub tile_fill: String,
    pub tile_stroke: String,
    pub header_text: String,
    pub header_sub: String,
    pub header_text_on_dark: String,
    pub header_sub_on_dark: String,
    pub group_fill: String,
    pub group_stroke: String,
    pub note_fill: String,
    pub note_stroke: String,
    pub tag_color: String,
    pub badge_fill: Option<String>,
    pub badge_text: Option<String>,
    pub graph_fill: String,
    pub graph_grid: String,
    pub graph_line: String,
    pub tones: IndexMap<String, String>,
    pub statuses: IndexMap<String, Status>,
    pub link_labels: IndexMap<String, String>,
    pub title: f64,
    pub subtitle: f64,
    pub legend: f64,
    pub panel_title: f64,
    pub panel_sub: f64,
    pub tile: f64,
    pub tile_sub: f64,
    pub body: f64,
    pub small: f64,
    pub min_text: f64,
    pub lid: f64,
    pub depth: f64,
    pub pad: f64,
    pub tile_gap: f64,
    pub section_gap: f64,
    pub tile_h: f64,
    pub line_h: f64,
    pub header_h: f64,
    pub header_h_sub: f64,
    pub panel_min_w: f64,
    pub tile_max_w: f64,
    pub radius: f64,
    pub col_gap: f64,
    pub row_gap: f64,
    pub min_gap: f64,
    pub margin: f64,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            ink: "#1F2328".to_owned(),
            muted: "#5E6874".to_owned(),
            faint: "#8A939D".to_owned(),
            paper: "#FFFFFF".to_owned(),
            line: "#3F454C".to_owned(),
            panel_fill: "#F7F7F8".to_owned(),
            panel_stroke: "#C5CCD3".to_owned(),
            tile_fill: "#FFFFFF".to_owned(),
            tile_stroke: "#2B3036".to_owned(),
            header_text: "#1F2328".to_owned(),
            header_sub: "#5A6067".to_owned(),
            header_text_on_dark: "#FFFFFF".to_owned(),
            header_sub_on_dark: "#D5D9DE".to_owned(),
            group_fill: "#F2F3F5".to_owned(),
            group_stroke: "#A3A8B0".to_owned(),
            note_fill: "#FFF8EC".to_owned(),
            note_stroke: "#D9B77E".to_owned(),
            tag_color: "#2B3036".to_owned(),
            badge_fill: Some("#FFC83D".to_owned()),
            badge_text: Some("#1F2328".to_owned()),
            graph_fill: "#FBFBFC".to_owned(),
            graph_grid: "#EBECEF".to_owned(),
            graph_line: "#97A3AB".to_owned(),
            tones: tones(),
            statuses: statuses(),
            link_labels: link_labels(),
            title: 26.0,
            subtitle: 13.5,
            legend: 12.0,
            panel_title: 14.5,
            panel_sub: 11.0,
            tile: 12.0,
            tile_sub: 10.5,
            body: 11.0,
            small: 10.5,
            min_text: 10.0,
            lid: 9.0,
            depth: 4.0,
            pad: 12.0,
            tile_gap: 8.0,
            section_gap: 12.0,
            tile_h: 30.0,
            line_h: 14.5,
            header_h: 36.0,
            header_h_sub: 50.0,
            panel_min_w: 200.0,
            tile_max_w: 230.0,
            radius: 6.0,
            col_gap: 72.0,
            row_gap: 28.0,
            min_gap: 16.0,
            margin: 40.0,
        }
    }
}

impl Theme {
    /// # Errors
    /// Returns an error when `key` is not one of this theme's statuses.
    pub fn status(&self, key: Option<&str>) -> Result<Option<&Status>, DiagramError> {
        let Some(key) = key else {
            return Ok(None);
        };
        self.statuses.get(key).map(Some).ok_or_else(|| {
            let names = self.statuses.keys().cloned().collect::<Vec<_>>().join(", ");
            DiagramError::new(format!(
                "unknown status {key}; this theme has: {names}",
                key = crate::num::py_repr(key)
            ))
        })
    }

    /// # Errors
    /// Returns an error when `key` is not a hex color and not one of this theme's tones.
    pub fn tone(&self, key: &str) -> Result<String, DiagramError> {
        if key.starts_with('#') {
            return Ok(key.to_owned());
        }
        self.tones.get(key).cloned().ok_or_else(|| {
            let names = self.tones.keys().cloned().collect::<Vec<_>>().join(", ");
            DiagramError::new(format!(
                "unknown tone {key}; this theme has: {names}",
                key = crate::num::py_repr(key)
            ))
        })
    }

    /// # Errors
    /// Returns an error when a header color is not a 6-digit hex value.
    pub fn header_colors(&self, background: &str) -> Result<(String, String), DiagramError> {
        let dark = (luminance(background)? + 0.05) / (luminance(&self.header_text)? + 0.05);
        let light =
            (luminance(&self.header_text_on_dark)? + 0.05) / (luminance(background)? + 0.05);
        if dark >= light {
            Ok((self.header_text.clone(), self.header_sub.clone()))
        } else {
            Ok((
                self.header_text_on_dark.clone(),
                self.header_sub_on_dark.clone(),
            ))
        }
    }
}

#[must_use]
pub fn shade(hex_color: &str, amount: f64) -> String {
    let Some((r, g, b)) = rgb(hex_color) else {
        return hex_color.to_owned();
    };
    let (target, a) = if amount >= 0.0 {
        (0.0, amount)
    } else {
        (255.0, -amount)
    };
    let mix = |v: u8| py_channel(mul(f64::from(v), 1.0 - a) + mul(target, a));
    format!("#{:02X}{:02X}{:02X}", mix(r), mix(g), mix(b))
}

fn luminance(hex_color: &str) -> Result<f64, DiagramError> {
    let (r, g, b) =
        rgb(hex_color).ok_or_else(|| DiagramError::new(format!("bad color {hex_color}")))?;
    let channel = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    Ok(mul(0.2126, channel(r)) + mul(0.7152, channel(g)) + mul(0.0722, channel(b)))
}

fn rgb(hex_color: &str) -> Option<(u8, u8, u8)> {
    let h = hex_color.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some((r, g, b))
}
