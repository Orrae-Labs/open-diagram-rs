use crate::font::{self, cap_height};
use crate::num::fmt;

pub struct Canvas {
    pub width: f64,
    pub height: f64,
    parts: Vec<String>,
    pub min_text: Option<f64>,
}

impl Canvas {
    pub const fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            parts: Vec::new(),
            min_text: None,
        }
    }

    pub fn add(&mut self, raw: impl Into<String>) {
        self.parts.push(raw.into());
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, paint: Paint<'_>) {
        let mut attrs = vec![
            attr_num("x", x),
            attr_num("y", y),
            attr_num("width", w),
            attr_num("height", h),
        ];
        if paint.radius != 0.0 {
            attrs.push(attr_num("rx", paint.radius));
        }
        attrs.push(attr("fill", paint.fill));
        if let Some(stroke) = paint.stroke {
            attrs.push(attr("stroke", stroke));
            attrs.push(attr_num("stroke-width", paint.width));
        }
        if let Some(dash) = dash_value(paint.dash) {
            attrs.push(attr("stroke-dasharray", &dash));
            attrs.push(attr("stroke-linecap", "round"));
        }
        if let Some(opacity) = paint.opacity {
            attrs.push(attr_num("opacity", opacity));
        }
        self.add(format!("<rect {}/>", attrs.join(" ")));
    }

    pub fn path(
        &mut self,
        d: &str,
        fill: &str,
        stroke: Option<&str>,
        width: f64,
        dash: Option<&str>,
        opacity: Option<f64>,
    ) {
        let mut attrs = vec![attr("d", d), attr("fill", fill)];
        if let Some(stroke) = stroke {
            attrs.push(attr("stroke", stroke));
            attrs.push(attr_num("stroke-width", width));
            if let Some(dash) = dash_value(dash) {
                attrs.push(attr("stroke-dasharray", &dash));
            }
            attrs.push(attr("stroke-linecap", "round"));
            attrs.push(attr("stroke-linejoin", "round"));
        }
        if let Some(opacity) = opacity {
            attrs.push(attr_num("opacity", opacity));
        }
        self.add(format!("<path {}/>", attrs.join(" ")));
    }

    pub fn line(
        &mut self,
        from: (f64, f64),
        to: (f64, f64),
        stroke: &str,
        width: f64,
        dash: Option<&str>,
    ) {
        let d = format!(
            "M{} {}L{} {}",
            fmt(from.0),
            fmt(from.1),
            fmt(to.0),
            fmt(to.1)
        );
        self.path(&d, "none", Some(stroke), width, dash, None);
    }

    pub fn circle(
        &mut self,
        cx: f64,
        cy: f64,
        r: f64,
        fill: &str,
        stroke: Option<&str>,
        width: f64,
    ) {
        let mut attrs = vec![
            attr_num("cx", cx),
            attr_num("cy", cy),
            attr_num("r", r),
            attr("fill", fill),
        ];
        if let Some(stroke) = stroke {
            attrs.push(attr("stroke", stroke));
            attrs.push(attr_num("stroke-width", width));
        }
        self.add(format!("<circle {}/>", attrs.join(" ")));
    }

    pub fn image(&mut self, href: &str, x: f64, y: f64, w: f64, h: f64) {
        let attrs = [
            attr_num("x", x),
            attr_num("y", y),
            attr_num("width", w),
            attr_num("height", h),
        ]
        .join(" ");
        self.add(format!("<image {attrs} href=\"{}\"/>", xml_escape(href)));
    }

    pub fn text(&mut self, s: &str, mut x: f64, baseline: f64, letter: Letter<'_>) -> f64 {
        if s.is_empty() {
            return 0.0;
        }
        let w = font::text_width(s, letter.size, letter.weight);
        if letter.anchor == "middle" {
            x -= w / 2.0;
        } else if letter.anchor == "end" {
            x -= w;
        }
        let d = font::outline(s, letter.size, x, baseline, letter.weight);
        if !d.is_empty() {
            self.add(format!(
                "<path d=\"{d}\" fill=\"{}\"><title>{}</title></path>",
                letter.color,
                xml_escape(s)
            ));
        }
        self.min_text = Some(
            self.min_text
                .map_or(letter.size, |cur| cur.min(letter.size)),
        );
        w
    }

    pub fn text_centered(
        &mut self,
        s: &str,
        cx: f64,
        cy: f64,
        size: f64,
        weight: &str,
        color: &str,
    ) -> f64 {
        self.text(
            s,
            cx,
            cy + cap_height(size, weight) / 2.0,
            Letter::new(size, weight, color, "middle"),
        )
    }

    pub fn open_group(&mut self, dx: f64, dy: f64) {
        self.add(format!(
            "<g transform=\"translate({} {})\">",
            fmt(dx),
            fmt(dy)
        ));
    }

    pub fn close_group(&mut self) {
        self.add("</g>");
    }

    pub fn svg(&self, title: Option<&str>, background: Option<&str>, desc: Option<&str>) -> String {
        let w = fmt(self.width);
        let h = fmt(self.height);
        let mut head = vec![format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" role=\"img\">"
        )];
        if let Some(title) = title {
            head.push(format!("<title>{}</title>", xml_escape(title)));
        }
        if let Some(desc) = desc {
            head.push(format!("<desc>{}</desc>", xml_escape(desc)));
        }
        if let Some(background) = background {
            head.push(format!(
                "<rect width=\"{w}\" height=\"{h}\" fill=\"{background}\"/>"
            ));
        }
        let mut body = String::new();
        for part in head.iter().chain(self.parts.iter()) {
            body.push_str(part);
        }
        body.push_str("</svg>\n");
        body
    }
}

#[derive(Clone, Copy)]
pub struct Paint<'a> {
    pub fill: &'a str,
    pub stroke: Option<&'a str>,
    pub width: f64,
    pub radius: f64,
    pub dash: Option<&'a str>,
    pub opacity: Option<f64>,
}

impl<'a> Paint<'a> {
    pub const fn new(
        fill: &'a str,
        stroke: Option<&'a str>,
        width: f64,
        radius: f64,
        dash: Option<&'a str>,
        opacity: Option<f64>,
    ) -> Self {
        Self {
            fill,
            stroke,
            width,
            radius,
            dash,
            opacity,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Letter<'a> {
    pub size: f64,
    pub weight: &'a str,
    pub color: &'a str,
    pub anchor: &'a str,
}

impl<'a> Letter<'a> {
    pub const fn new(size: f64, weight: &'a str, color: &'a str, anchor: &'a str) -> Self {
        Self {
            size,
            weight,
            color,
            anchor,
        }
    }
}

pub fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn attr(key: &str, value: &str) -> String {
    format!("{key}=\"{value}\"")
}

fn attr_num(key: &str, value: f64) -> String {
    format!("{key}=\"{}\"", fmt(value))
}

fn dash_value(dash: Option<&str>) -> Option<String> {
    match dash {
        None => None,
        Some("dashed") => Some("5 3.5".to_owned()),
        Some("dotted") => Some("0.1 3.4".to_owned()),
        Some(other) => Some(other.to_owned()),
    }
}
