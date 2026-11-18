"""A small SVG canvas. Shapes go in as they are drawn; text goes in as outline paths."""
from __future__ import annotations

from typing import List, Optional
from xml.sax.saxutils import escape

from . import fonts
from .fonts import fmt

DASHES = {"dashed": "5 3.5", "dotted": "0.1 3.4"}


def _dash(dash: Optional[str]) -> Optional[str]:
    if dash is None:
        return None
    return DASHES.get(dash, dash)


def _attrs(**kw) -> str:
    parts = []
    for key, value in kw.items():
        if value is None:
            continue
        if isinstance(value, float):
            value = fmt(value)
        parts.append(f'{key.rstrip("_").replace("_", "-")}="{value}"')
    return " ".join(parts)


class Canvas:
    def __init__(self, width: float, height: float):
        self.width, self.height = width, height
        self.parts: List[str] = []
        self.min_text: Optional[float] = None

    def add(self, raw: str) -> None:
        self.parts.append(raw)

    def rect(self, x, y, w, h, fill="none", stroke=None, width=1.0, radius=0.0, dash=None, opacity=None):
        self.add(f"<rect {_attrs(x=float(x), y=float(y), width=float(w), height=float(h), rx=float(radius) or None, fill=fill, stroke=stroke, stroke_width=float(width) if stroke else None, stroke_dasharray=_dash(dash), stroke_linecap='round' if dash else None, opacity=opacity)}/>")

    def path(self, d, fill="none", stroke=None, width=1.0, dash=None, opacity=None):
        self.add(f"<path {_attrs(d=d, fill=fill, stroke=stroke, stroke_width=float(width) if stroke else None, stroke_dasharray=_dash(dash), stroke_linecap='round' if stroke else None, stroke_linejoin='round' if stroke else None, opacity=opacity)}/>")

    def line(self, x1, y1, x2, y2, stroke, width=1.0, dash=None):
        self.path(f"M{fmt(x1)} {fmt(y1)}L{fmt(x2)} {fmt(y2)}", stroke=stroke, width=width, dash=dash)

    def circle(self, cx, cy, r, fill="none", stroke=None, width=1.0):
        self.add(f"<circle {_attrs(cx=float(cx), cy=float(cy), r=float(r), fill=fill, stroke=stroke, stroke_width=float(width) if stroke else None)}/>")

    def image(self, href, x, y, w, h):
        self.add(f'<image {_attrs(x=float(x), y=float(y), width=float(w), height=float(h))} href="{escape(href)}"/>')

    def text(self, s: str, x: float, baseline: float, size: float, weight: str = "regular", color: str = "#000000",
             anchor: str = "start") -> float:
        """Draw one line of text as an outline path. Returns its width."""
        if not s:
            return 0.0
        w = fonts.text_width(s, size, weight)
        if anchor == "middle":
            x -= w / 2
        elif anchor == "end":
            x -= w
        d = fonts.face(weight).outline(s, size, x, baseline)
        if d:
            self.add(f'<path d="{d}" fill="{color}"><title>{escape(s)}</title></path>')
        self.min_text = size if self.min_text is None else min(self.min_text, size)
        return w

    def text_centered(self, s: str, cx: float, cy: float, size: float, weight: str = "regular",
                      color: str = "#000000") -> float:
        """Draw one line of text centred on (cx, cy), using the capital-letter height."""
        return self.text(s, cx, cy + fonts.cap_height(size, weight) / 2, size, weight, color, "middle")

    def open_group(self, dx: float = 0, dy: float = 0) -> None:
        self.add(f'<g transform="translate({fmt(dx)} {fmt(dy)})">')

    def close_group(self) -> None:
        self.add("</g>")

    def svg(self, title: Optional[str] = None, background: Optional[str] = "#FFFFFF") -> str:
        w, h = fmt(self.width), fmt(self.height)
        head = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img">']
        if title:
            head.append(f"<title>{escape(title)}</title>")
        if background:
            head.append(f'<rect width="{w}" height="{h}" fill="{background}"/>')
        return "".join(head + self.parts + ["</svg>\n"])
