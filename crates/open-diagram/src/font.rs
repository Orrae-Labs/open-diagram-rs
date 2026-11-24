use std::cell::RefCell;
use std::collections::HashMap;

use ttf_parser::gpos::PositioningSubtable;
use ttf_parser::opentype_layout::LayoutTable;
use ttf_parser::{Face as TtfFace, GlyphId, OutlineBuilder, Tag};

use crate::num::{fmt, mul};

const WEIGHTS: [&str; 4] = ["regular", "medium", "semibold", "bold"];

pub struct Typeface {
    bytes: Vec<u8>,
    upm: f64,
    cap: f64,
    kern: RefCell<HashMap<(u16, u16), i32>>,
}

impl Typeface {
    pub fn load(weight: &str) -> Result<Self, String> {
        let (name, bytes) = match weight {
            "regular" => ("Inter-Regular.ttf", &include_bytes!("../fonts/Inter-Regular.ttf")[..]),
            "medium" => ("Inter-Medium.ttf", &include_bytes!("../fonts/Inter-Medium.ttf")[..]),
            "semibold" => ("Inter-SemiBold.ttf", &include_bytes!("../fonts/Inter-SemiBold.ttf")[..]),
            "bold" => ("Inter-Bold.ttf", &include_bytes!("../fonts/Inter-Bold.ttf")[..]),
            _ => {
                return Err(format!(
                    "unknown font weight {weight:?}; use one of {}",
                    WEIGHTS.join(", ")
                ));
            }
        };
        let face = TtfFace::parse(bytes, 0).map_err(|err| format!("parsing {name}: {err}"))?;
        let upm = f64::from(face.units_per_em());
        let cap = f64::from(face.capital_height().unwrap_or_else(|| face.ascender()));
        Ok(Self { bytes: bytes.to_vec(), upm, cap, kern: RefCell::new(HashMap::new()) })
    }

    fn ttf(&self) -> TtfFace<'_> {
        TtfFace::parse(&self.bytes, 0).unwrap_or_else(|err| panic!("bundled Inter font failed to parse: {err}"))
    }

    pub fn cap_height(&self, size: f64) -> f64 {
        self.cap * size / self.upm
    }

    pub fn width(&self, text: &str, size: f64) -> f64 {
        let face = self.ttf();
        let ids = Self::glyphs(&face, text);
        f64::from(self.positions(&face, &ids).1) * size / self.upm
    }

    pub fn outline(&self, text: &str, size: f64, x: f64, baseline: f64) -> String {
        let face = self.ttf();
        let ids = Self::glyphs(&face, text);
        let (xs, _) = self.positions(&face, &ids);
        let scale = size / self.upm;
        let mut pen = Pen::default();
        for (id, gx) in ids.iter().zip(xs) {
            let mut builder = Mapped {
                pen: &mut pen,
                scale,
                tx: x + mul(f64::from(gx), scale),
                baseline,
            };
            face.outline_glyph(*id, &mut builder);
        }
        pen.commands.join("")
    }

    fn glyphs(face: &TtfFace<'_>, text: &str) -> Vec<GlyphId> {
        let fallback = face.glyph_index('\u{FFFD}').unwrap_or(GlyphId(0));
        text.chars().map(|ch| face.glyph_index(ch).unwrap_or(fallback)).collect()
    }

    fn positions(&self, face: &TtfFace<'_>, ids: &[GlyphId]) -> (Vec<i32>, i32) {
        let mut xs = Vec::with_capacity(ids.len());
        let mut x = 0_i32;
        for (i, id) in ids.iter().enumerate() {
            if i > 0 {
                x += self.kern(face, ids[i - 1], *id);
            }
            xs.push(x);
            x += i32::from(face.glyph_hor_advance(*id).unwrap_or(0));
        }
        (xs, x)
    }

    fn kern(&self, face: &TtfFace<'_>, left: GlyphId, right: GlyphId) -> i32 {
        let key = (left.0, right.0);
        if let Some(value) = self.kern.borrow().get(&key) {
            return *value;
        }
        let value = pair_value(face.tables().gpos.as_ref(), left, right);
        self.kern.borrow_mut().insert(key, value);
        value
    }
}

fn pair_value(gpos: Option<&LayoutTable<'_>>, left: GlyphId, right: GlyphId) -> i32 {
    let Some(gpos) = gpos else {
        return 0;
    };
    let mut indices = Vec::new();
    for i in 0..gpos.features.len() {
        let Some(feature) = gpos.features.get(i) else {
            continue;
        };
        if feature.tag != Tag::from_bytes(b"kern") {
            continue;
        }
        for j in 0..feature.lookup_indices.len() {
            if let Some(index) = feature.lookup_indices.get(j) {
                indices.push(index);
            }
        }
    }
    indices.sort_unstable();
    indices.dedup();
    for index in indices {
        let Some(lookup) = gpos.lookups.get(index) else {
            continue;
        };
        for sub in lookup.subtables.into_iter::<PositioningSubtable>() {
            let PositioningSubtable::Pair(pair) = sub else {
                continue;
            };
            let Some(covered) = pair.coverage().get(left) else {
                continue;
            };
            match pair {
                ttf_parser::gpos::PairAdjustment::Format1 { sets, .. } => {
                    let Some(set) = sets.get(covered) else {
                        continue;
                    };
                    if let Some((value, _)) = set.get(right) {
                        return i32::from(value.x_advance);
                    }
                }
                ttf_parser::gpos::PairAdjustment::Format2 { classes, matrix, .. } => {
                    let c1 = classes.0.get(left);
                    let c2 = classes.1.get(right);
                    return matrix.get((c1, c2)).map_or(0, |(value, _)| i32::from(value.x_advance));
                }
            }
        }
    }
    0
}

#[derive(Default)]
struct Pen {
    commands: Vec<String>,
    last_cmd: Option<char>,
    last: Option<(f64, f64)>,
    start: Option<(f64, f64)>,
    last_was_line: bool,
}

impl Pen {
    fn move_to(&mut self, x: f64, y: f64) {
        if self.last_cmd == Some('M') {
            self.commands.pop();
        }
        self.commands.push(format!("M{} {}", fmt(x), fmt(y)));
        self.last_cmd = Some('M');
        self.last = Some((x, y));
        self.start = Some((x, y));
        self.last_was_line = false;
    }

    #[allow(clippy::float_cmp, reason = "SVG path H and V commands use exact coordinate equality before the numbers are formatted")]
    fn line_to(&mut self, x: f64, y: f64) {
        let Some((lx, ly)) = self.last else {
            self.commands.push(format!("L{} {}", fmt(x), fmt(y)));
            self.last_cmd = Some('L');
            self.last = Some((x, y));
            self.last_was_line = true;
            return;
        };
        if x == lx && y == ly {
            return;
        }
        self.last_was_line = true;
        if x == lx {
            self.commands.push(format!("V{}", fmt(y)));
            self.last_cmd = Some('V');
        } else if y == ly {
            self.commands.push(format!("H{}", fmt(x)));
            self.last_cmd = Some('H');
        } else if self.last_cmd == Some('M') {
            self.commands.push(format!(" {} {}", fmt(x), fmt(y)));
        } else {
            self.commands.push(format!("L{} {}", fmt(x), fmt(y)));
            self.last_cmd = Some('L');
        }
        self.last = Some((x, y));
    }

    fn quad_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.commands.push(format!("Q{} {} {} {}", fmt(x1), fmt(y1), fmt(x), fmt(y)));
        self.last_cmd = Some('Q');
        self.last = Some((x, y));
        self.last_was_line = false;
    }

    fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.commands.push(format!(
            "C{} {} {} {} {} {}",
            fmt(x1),
            fmt(y1),
            fmt(x2),
            fmt(y2),
            fmt(x),
            fmt(y)
        ));
        self.last_cmd = Some('C');
        self.last = Some((x, y));
        self.last_was_line = false;
    }

    fn close(&mut self) {
        if self.last_was_line
            && let (Some(start), Some(end)) = (self.start, self.last)
                && start == end {
                    self.commands.pop();
                }
        self.commands.push("Z".to_owned());
        self.last_cmd = Some('Z');
        self.last = None;
        self.start = None;
        self.last_was_line = false;
    }
}

struct Mapped<'a> {
    pen: &'a mut Pen,
    scale: f64,
    tx: f64,
    baseline: f64,
}

impl Mapped<'_> {
    fn map(&self, x: f32, y: f32) -> (f64, f64) {
        let x = f64::from(x);
        let y = f64::from(y);
        (mul(self.scale, x) + self.tx, mul(-self.scale, y) + self.baseline)
    }
}

impl OutlineBuilder for Mapped<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.pen.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.pen.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.map(x1, y1);
        let (x, y) = self.map(x, y);
        self.pen.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.map(x1, y1);
        let (x2, y2) = self.map(x2, y2);
        let (x, y) = self.map(x, y);
        self.pen.curve_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.pen.close();
    }
}

thread_local! {
    static FACES: RefCell<HashMap<String, Typeface>> = RefCell::new(HashMap::new());
}

#[must_use]
pub fn text_width(text: &str, size: f64, weight: &str) -> f64 {
    with_face(weight, |face| face.width(text, size))
}

#[must_use]
pub fn cap_height(size: f64, weight: &str) -> f64 {
    with_face(weight, |face| face.cap_height(size))
}

pub fn lines_width(lines: &[String], size: f64, weight: &str) -> f64 {
    lines.iter().map(|line| text_width(line, size, weight)).fold(0.0, f64::max)
}

#[must_use]
pub fn outline(text: &str, size: f64, x: f64, baseline: f64, weight: &str) -> String {
    with_face(weight, |face| face.outline(text, size, x, baseline))
}

#[must_use]
pub fn wrap(text: &str, size: f64, max_width: Option<f64>, weight: &str) -> Vec<String> {
    let mut out = Vec::new();
    for para in text.split('\n') {
        let Some(max_width) = max_width else {
            out.push(para.to_owned());
            continue;
        };
        let mut line = String::new();
        for word in para.split(' ') {
            let trial = if line.is_empty() { word.to_owned() } else { format!("{line} {word}") };
            if !line.is_empty() && text_width(&trial, size, weight) > max_width {
                out.push(std::mem::take(&mut line));
                word.clone_into(&mut line);
            } else {
                line = trial;
            }
        }
        out.push(line);
    }
    out
}

fn with_face<T>(weight: &str, body: impl FnOnce(&Typeface) -> T) -> T {
    FACES.with(|faces| {
        let mut faces = faces.borrow_mut();
        if !faces.contains_key(weight) {
            let loaded = Typeface::load(weight).unwrap_or_else(|err| panic!("{err}"));
            faces.insert(weight.to_owned(), loaded);
        }
        body(faces.get(weight).unwrap_or_else(|| unreachable!()))
    })
}
