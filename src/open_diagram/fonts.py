"""Measure text and turn it into outline shapes, using the bundled Inter font files.

Every label is measured from the font itself, so boxes are sized to fit their words before anything is placed.
Every label is drawn as a filled path rather than as SVG text, so the picture looks the same on any computer and in
any program (a browser, PowerPoint, Keynote, Figma), whether or not Inter is installed.
"""
from __future__ import annotations

import os
from typing import Dict, List, Optional, Tuple

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

FONT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fonts")
FILES = {
    "regular": "Inter-Regular.ttf",
    "medium": "Inter-Medium.ttf",
    "semibold": "Inter-SemiBold.ttf",
    "bold": "Inter-Bold.ttf",
}


def fmt(v: float) -> str:
    """Format a number for SVG with at most two decimals."""
    s = f"{v:.2f}".rstrip("0").rstrip(".")
    return "0" if s in ("", "-0") else s


class Face:
    """One font file: glyph shapes, advance widths and pair kerning."""

    def __init__(self, path: str):
        font = TTFont(path)
        self.glyphs = font.getGlyphSet()
        self.cmap = font.getBestCmap()
        self.hmtx = font["hmtx"]
        self.upm = font["head"].unitsPerEm
        self.cap = font["OS/2"].sCapHeight
        self._pairs = _kerning_subtables(font)
        self._kern: Dict[Tuple[str, str], int] = {}

    def names(self, text: str) -> List[str]:
        fallback = self.cmap.get(0xFFFD) or ".notdef"
        return [self.cmap.get(ord(ch), fallback) for ch in text]

    def kern(self, left: str, right: str) -> int:
        key = (left, right)
        if key not in self._kern:
            self._kern[key] = _pair_value(self._pairs, left, right)
        return self._kern[key]

    def positions(self, names: List[str]) -> Tuple[List[int], int]:
        """Return each glyph's x position and the total advance, in font units."""
        xs, x = [], 0
        for i, name in enumerate(names):
            if i:
                x += self.kern(names[i - 1], name)
            xs.append(x)
            x += self.hmtx[name][0]
        return xs, x

    def width(self, text: str, size: float) -> float:
        return self.positions(self.names(text))[1] * size / self.upm

    def outline(self, text: str, size: float, x: float, baseline: float) -> str:
        """Return SVG path data for the text, with its left end at x and its baseline at the given y."""
        names = self.names(text)
        xs, _ = self.positions(names)
        pen = SVGPathPen(self.glyphs, ntos=fmt)
        scale = size / self.upm
        for name, gx in zip(names, xs):
            self.glyphs[name].draw(TransformPen(pen, (scale, 0, 0, -scale, x + gx * scale, baseline)))
        return pen.getCommands()


def _kerning_subtables(font: TTFont):
    """Collect the pair-adjustment subtables used by the font's 'kern' feature."""
    if "GPOS" not in font:
        return []
    table = font["GPOS"].table
    if not table.FeatureList or not table.LookupList:
        return []
    indices = sorted({i for rec in table.FeatureList.FeatureRecord if rec.FeatureTag == "kern"
                      for i in rec.Feature.LookupListIndex})
    out = []
    for i in indices:
        lookup = table.LookupList.Lookup[i]
        for sub in lookup.SubTable:
            kind = lookup.LookupType
            if kind == 9:  # extension lookup: the real subtable sits inside
                kind, sub = sub.ExtensionLookupType, sub.ExtSubTable
            if kind == 2:
                out.append((sub, {g: k for k, g in enumerate(sub.Coverage.glyphs)}))
    return out


def _x_advance(value) -> int:
    return (getattr(value, "XAdvance", 0) or 0) if value is not None else 0


def _pair_value(subtables, left: str, right: str) -> int:
    for sub, coverage in subtables:
        k = coverage.get(left)
        if k is None:
            continue
        if sub.Format == 1:
            for rec in sub.PairSet[k].PairValueRecord:
                if rec.SecondGlyph == right:
                    return _x_advance(rec.Value1)
        elif sub.Format == 2:
            c1 = sub.ClassDef1.classDefs.get(left, 0)
            c2 = sub.ClassDef2.classDefs.get(right, 0)
            return _x_advance(sub.Class1Record[c1].Class2Record[c2].Value1)
    return 0


_FACES: Dict[str, Face] = {}


def face(weight: str = "regular") -> Face:
    if weight not in FILES:
        raise ValueError(f"unknown font weight {weight!r}; use one of {', '.join(FILES)}")
    if weight not in _FACES:
        _FACES[weight] = Face(os.path.join(FONT_DIR, FILES[weight]))
    return _FACES[weight]


def text_width(text: str, size: float, weight: str = "regular") -> float:
    """Width of one line of text in pixels."""
    return face(weight).width(text, size)


def cap_height(size: float, weight: str = "regular") -> float:
    """Height of a capital letter in pixels; used to centre text vertically."""
    f = face(weight)
    return f.cap * size / f.upm


def lines_width(lines: List[str], size: float, weight: str = "regular") -> float:
    return max((text_width(s, size, weight) for s in lines), default=0.0)


def wrap(text: str, size: float, max_width: Optional[float], weight: str = "regular") -> List[str]:
    """Split text into lines no wider than max_width. A newline in the text always starts a new line."""
    out: List[str] = []
    for para in text.split("\n"):
        if max_width is None:
            out.append(para)
            continue
        line = ""
        for word in para.split(" "):
            trial = f"{line} {word}" if line else word
            if line and text_width(trial, size, weight) > max_width:
                out.append(line)
                line = word
            else:
                line = trial
        out.append(line)
    return out
