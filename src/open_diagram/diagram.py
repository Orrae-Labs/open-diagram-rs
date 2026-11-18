"""The diagram: place blocks, join them with lines, group them, then build with checks.

Every build runs the layout checks and lists every problem it finds in plain words, such as
"'Fulfilment' and 'Returns' are only 9 px apart (minimum 16)" or "line 'Website' -> 'Reports' crosses 'Checkout'".
The SVG and PNG are still written, so whoever is fixing the layout (a person or an AI agent) can look at them.
"""
from __future__ import annotations

import math
import os
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Sequence, Tuple, Union

from . import render as _render
from .blocks import Block, BlockLike, Note, as_block
from .fonts import cap_height, fmt, text_width
from .svg import Canvas
from .theme import Theme

SIDES = {"left": (-1.0, 0.0), "right": (1.0, 0.0), "top": (0.0, -1.0), "bottom": (0.0, 1.0)}
ARROW = 7.0
STUB = 16.0
CORNER = 10.0
Point = Tuple[float, float]


class LayoutError(Exception):
    """Raised by Diagram.save() when the layout checks find problems. `problems` lists them."""

    def __init__(self, problems: Sequence[str], files: Sequence[str] = ()):
        self.problems = list(problems)
        msg = f"{len(self.problems)} layout problem(s):\n" + "\n".join(f"  - {p}" for p in self.problems)
        if files:
            msg += "\nThe drawing was still written, so you can look at it: " + ", ".join(files)
        super().__init__(msg)


class Placed:
    """A block at a position. Returned by every placing method; pass it to link(), group(), right_of() and below()."""

    def __init__(self, name: str, block: Block, x: float, y: float, w: float, h: float):
        self.name, self.block, self.x, self.y, self.w, self.h = name, block, x, y, w, h

    @property
    def right(self) -> float:
        return self.x + self.w

    @property
    def bottom(self) -> float:
        return self.y + self.h

    @property
    def cx(self) -> float:
        return self.x + self.w / 2

    @property
    def cy(self) -> float:
        return self.y + self.h / 2

    def rect(self) -> Tuple[float, float, float, float]:
        return self.x, self.y, self.w, self.h

    def __repr__(self) -> str:
        return f"Placed({self.name!r}, x={self.x:.0f}, y={self.y:.0f}, w={self.w:.0f}, h={self.h:.0f})"


@dataclass
class Link:
    a: Placed
    b: Placed
    kind: str
    label: Optional[str]
    start: str
    end: str
    via_x: Optional[float]
    via_y: Optional[float]
    cross_ok: bool
    offsets: List[float] = field(default_factory=lambda: [0.0, 0.0])
    d: str = ""
    tip: Point = (0.0, 0.0)
    direction: Point = (0.0, 0.0)
    samples: List[Point] = field(default_factory=list)
    label_at: Point = (0.0, 0.0)


@dataclass
class Group:
    label: str
    members: List[Placed]
    sub: Optional[str]
    pad: float
    rect: Tuple[float, float, float, float] = (0.0, 0.0, 0.0, 0.0)


@dataclass
class Report:
    svg: str
    width: float
    height: float
    min_text: Optional[float]
    problems: List[str]
    files: List[str] = field(default_factory=list)


class Diagram:
    """A whole picture: a header (title, tag, subtitle, key, note) above blocks, lines and groups.

    Positions are in pixels. Place blocks anywhere; the canvas grows to fit them, and the header is added above.
    """

    def __init__(self, title: str, subtitle: Optional[str] = None, tag: Optional[str] = None,
                 note: Optional[Note] = None, theme: Optional[Theme] = None, legend: bool = True,
                 steps: Sequence[Tuple[int, str]] = (), steps_label: str = "Steps"):
        self.title, self.subtitle, self.tag, self.note = title, subtitle, tag, note
        self.theme = theme or Theme()
        self.legend = legend
        self.steps = list(steps)
        self.steps_label = steps_label
        self.placed: List[Placed] = []
        self.links: List[Link] = []
        self.groups: List[Group] = []
        self._early: List[str] = []

    # ---------------------------------------------------------------------------------------------- placing

    def add(self, block: BlockLike, x: float = 0, y: float = 0, width: Optional[float] = None,
            name: Optional[str] = None) -> Placed:
        """Place a block with its top-left corner at (x, y). `width` stretches it; it never shrinks it."""
        block = as_block(block)
        w, h = block.measure(self.theme)
        label = name or getattr(block, "name", None) or getattr(block, "title", None) or f"block {len(self.placed) + 1}"
        if any(p.name == label for p in self.placed):
            label = f"{label} ({sum(1 for p in self.placed if p.name.startswith(label)) + 1})"
        if width is not None:
            if width + 0.5 < w:
                self._early.append(f"'{label}' needs {w:.0f} px wide but was given {width:.0f}; using {w:.0f}")
            w = max(w, width)
        placed = Placed(label, block, x, y, w, h)
        self.placed.append(placed)
        return placed

    def right_of(self, ref: Placed, block: BlockLike, gap: Optional[float] = None, align: str = "top",
                 width: Optional[float] = None, name: Optional[str] = None) -> Placed:
        """Place a block to the right of another. align: 'top', 'center' or 'bottom'."""
        block = as_block(block)
        w, h = block.measure(self.theme)
        gap = self.theme.col_gap if gap is None else gap
        y = {"top": ref.y, "center": ref.cy - h / 2, "bottom": ref.bottom - h}[align]
        return self.add(block, ref.right + gap, y, width, name)

    def below(self, ref: Placed, block: BlockLike, gap: Optional[float] = None, align: str = "left",
              width: Optional[float] = None, name: Optional[str] = None) -> Placed:
        """Place a block under another. align: 'left', 'center' or 'right'."""
        block = as_block(block)
        w, h = block.measure(self.theme)
        w = max(w, width or 0)
        gap = self.theme.row_gap if gap is None else gap
        x = {"left": ref.x, "center": ref.cx - w / 2, "right": ref.right - w}[align]
        return self.add(block, x, ref.bottom + gap, width, name)

    def stack(self, blocks: Sequence[BlockLike], x: float = 0, y: float = 0, gap: Optional[float] = None,
              same_width: bool = True) -> List[Placed]:
        """Place blocks in one column, top to bottom. With same_width, all take the widest block's width."""
        return self.columns([blocks], x, y, row_gap=gap, same_width=same_width)[0]

    def columns(self, cols: Sequence[Sequence[BlockLike]], x: float = 0, y: float = 0, gap: Optional[float] = None,
                row_gap: Optional[float] = None, valign: str = "center",
                same_width: bool = True) -> List[List[Placed]]:
        """Place columns of blocks left to right. valign: 'top', 'center' or 'bottom' against the tallest column."""
        t = self.theme
        gap = t.col_gap if gap is None else gap
        row_gap = t.row_gap if row_gap is None else row_gap
        blocks = [[as_block(b) for b in col] for col in cols]
        measured = [[(b, b.measure(t)) for b in col] for col in blocks]
        widths = [max(s[0] for _, s in col) for col in measured]
        heights = [sum(s[1] for _, s in col) + row_gap * (len(col) - 1) for col in measured]
        tallest = max(heights)
        out, cx = [], x
        for i, col in enumerate(measured):
            cy = y + {"top": 0, "center": (tallest - heights[i]) / 2, "bottom": tallest - heights[i]}[valign]
            placed = []
            for block, (w, h) in col:
                px = cx if same_width else cx + (widths[i] - w) / 2
                placed.append(self.add(block, px, cy, widths[i] if same_width else None))
                cy += h + row_gap
            out.append(placed)
            cx += widths[i] + gap
        return out

    def group(self, label: str, members: Sequence[Placed], sub: Optional[str] = None,
              pad: Optional[float] = None) -> Group:
        """Draw a shaded, dashed area with a label around some placed blocks."""
        g = Group(label, list(members), sub, self.theme.pad + 8 if pad is None else pad)
        self.groups.append(g)
        return g

    def link(self, a: Placed, b: Placed, kind: str = "flow", label: Optional[str] = None, start: str = "right",
             end: str = "left", via_x: Optional[float] = None, via_y: Optional[float] = None,
             cross_ok: bool = False) -> Link:
        """Join two placed blocks with an arrow from a to b.

        kind: 'flow' (solid: data moves today) or 'planned' (dotted: a connection still to build).
        start, end: the side of each block the line leaves and enters ('left', 'right', 'top', 'bottom').
        via_x / via_y: route the line with right angles through that x or y, for example a lane above everything.
        cross_ok: allow the line to pass behind other blocks without reporting it.
        """
        for p in (a, b):
            if not isinstance(p, Placed):
                raise TypeError("link() takes the Placed values returned by add(), right_of(), below() or columns()")
        for side in (start, end):
            if side not in SIDES:
                raise ValueError(f"unknown side {side!r}; use left, right, top or bottom")
        if kind not in self.theme.link_labels:
            raise ValueError(f"unknown line kind {kind!r}; use one of {', '.join(self.theme.link_labels)}")
        if via_x is not None and via_y is not None:
            raise ValueError("give via_x or via_y, not both")
        line = Link(a, b, kind, label, start, end, via_x, via_y, cross_ok)
        self.links.append(line)
        return line

    # --------------------------------------------------------------------------------------------- routing

    def _route(self) -> None:
        t = self.theme
        ends: Dict[Tuple[int, str], List[Tuple[Link, int]]] = defaultdict(list)
        for line in self.links:
            ends[(id(line.a), line.start)].append((line, 0))
            ends[(id(line.b), line.end)].append((line, 1))
        for (_, side), items in ends.items():
            first, which = items[0]
            owner = first.a if which == 0 else first.b

            def heading(item):
                line, e = item
                other = line.b if e == 0 else line.a
                if side in ("left", "right"):
                    return line.via_y if line.via_y is not None else other.cy
                return line.via_x if line.via_x is not None else other.cx

            items.sort(key=heading)
            lo, hi = owner.block.ports(side, owner.w, owner.h, t)
            n = len(items)
            step = min((hi - lo) / n, 14.0) if n > 1 else 0.0
            mid = (lo + hi) / 2
            for i, (line, e) in enumerate(items):
                line.offsets[e] = mid + (i - (n - 1) / 2) * step
        for line in self.links:
            self._shape(line)

    def _shape(self, line: Link) -> None:
        p0 = _anchor(line.a, line.start, line.offsets[0])
        tip = _anchor(line.b, line.end, line.offsets[1])
        d0, d3 = SIDES[line.start], SIDES[line.end]
        p3 = (tip[0] + d3[0] * ARROW, tip[1] + d3[1] * ARROW)
        line.tip, line.direction = tip, (-d3[0], -d3[1])
        if line.via_x is None and line.via_y is None:
            along = abs(p3[0] - p0[0]) if line.start in ("left", "right") else abs(p3[1] - p0[1])
            k = max(28.0, along * 0.5)
            c1 = (p0[0] + d0[0] * k, p0[1] + d0[1] * k)
            c2 = (p3[0] + d3[0] * k, p3[1] + d3[1] * k)
            line.d = f"M{_p(p0)}C{_p(c1)} {_p(c2)} {_p(p3)}"
            line.samples = [_bezier(p0, c1, c2, p3, i / 80) for i in range(81)]
            line.label_at = _bezier(p0, c1, c2, p3, 0.5)
            return
        s0 = (p0[0] + d0[0] * STUB, p0[1] + d0[1] * STUB)
        s3 = (p3[0] + d3[0] * STUB, p3[1] + d3[1] * STUB)
        if line.via_y is not None:
            middle = [(s0[0], line.via_y), (s3[0], line.via_y)]
        else:
            middle = [(line.via_x, s0[1]), (line.via_x, s3[1])]
        pts = _simplify([p0, s0] + middle + [s3, p3])
        line.d = _rounded(pts, CORNER)
        line.samples = _sample(pts, 5.0)
        longest = max(range(len(pts) - 1), key=lambda i: math.dist(pts[i], pts[i + 1]))
        a, b = pts[longest], pts[longest + 1]
        line.label_at = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)

    def _label_box(self, line: Link) -> Tuple[float, float, float, float]:
        w = text_width(line.label or "", self.theme.small, "medium") + 14
        return line.label_at[0] - w / 2, line.label_at[1] - 9, w, 18

    # ---------------------------------------------------------------------------------------------- checks

    def _check(self) -> List[str]:
        t = self.theme
        problems = list(self._early)
        for i, a in enumerate(self.placed):
            for b in self.placed[i + 1:]:
                gap = _separation(a.rect(), b.rect())
                if gap < 0:
                    problems.append(f"'{a.name}' and '{b.name}' overlap; move one (right_of() and below() keep a gap)")
                elif gap < t.min_gap:
                    problems.append(f"'{a.name}' and '{b.name}' are only {gap:.0f} px apart (minimum {t.min_gap:.0f})")
        for g in self.groups:
            for p in self.placed:
                if p not in g.members and _separation(g.rect, p.rect()) < 0:
                    problems.append(f"group '{g.label}' overlaps '{p.name}', which is not in it; "
                                    f"add it to the group or move it")
        for i, g in enumerate(self.groups):
            for h in self.groups[i + 1:]:
                if _separation(g.rect, h.rect) < 0 and not (_contains(g.rect, h.rect) or _contains(h.rect, g.rect)):
                    problems.append(f"groups '{g.label}' and '{h.label}' overlap")
        for line in self.links:
            name = f"line '{line.a.name}' -> '{line.b.name}'"
            if not line.cross_ok:
                for p in self.placed:
                    if p is line.a or p is line.b:
                        continue
                    x, y, w, h = p.rect()
                    if any(x + 2 < sx < x + w - 2 and y + 2 < sy < y + h - 2 for sx, sy in line.samples):
                        problems.append(f"{name} crosses '{p.name}'; route it with via_x/via_y, move a block, "
                                        f"or pass cross_ok=True")
            if line.label:
                box = self._label_box(line)
                for p in self.placed:
                    if _separation(box, p.rect()) < 0:
                        problems.append(f"the label '{line.label}' on {name} sits on '{p.name}'")
        return problems

    # ------------------------------------------------------------------------------------------------ build

    def render(self) -> Tuple[str, Report]:
        """Build the SVG and run every check. Returns the SVG text and a Report; never raises for layout problems."""
        t = self.theme
        if not self.placed:
            raise ValueError("the diagram has no blocks; add some with add(), right_of(), below() or columns()")
        self._route()
        for g in self.groups:
            x0 = min(p.x for p in g.members) - g.pad
            y0 = min(p.y for p in g.members) - g.pad - (t.line_h * (2 if g.sub else 1) + 6)
            x1 = max(p.right for p in g.members) + g.pad
            y1 = max(p.bottom for p in g.members) + g.pad
            g.rect = (x0, y0, x1 - x0, y1 - y0)
        problems = self._check()

        boxes = [p.rect() for p in self.placed] + [g.rect for g in self.groups]
        boxes += [self._label_box(l) for l in self.links if l.label]
        pts = [s for l in self.links for s in l.samples]
        minx = min([b[0] for b in boxes] + [p[0] for p in pts])
        miny = min([b[1] for b in boxes] + [p[1] for p in pts])
        maxx = max([b[0] + b[2] for b in boxes] + [p[0] for p in pts])
        maxy = max([b[1] + b[3] for b in boxes] + [p[1] for p in pts])

        m = t.margin
        head = self._header_plan(maxx - minx)
        width = head["width"]
        top = m + head["height"] + 32
        height = top + (maxy - miny) + m
        c = Canvas(math.ceil(width), math.ceil(height))
        self._draw_header(c, head)
        c.open_group(m - minx + (width - 2 * m - (maxx - minx)) / 2 if head["centre"] else m - minx, top - miny)
        for g in self.groups:
            x, y, w, h = g.rect
            c.rect(x, y, w, h, t.group_fill, t.group_stroke, 1.1, t.radius + 4, "dashed")
            c.text(g.label, x + 14, y + 12 + cap_height(12.5, "semibold"), 12.5, "semibold", t.ink)
            if g.sub:
                c.text(g.sub, x + 14, y + 12 + t.line_h + cap_height(t.small), t.small, "regular", t.muted)
        for line in self.links:
            dotted = line.kind == "planned"
            c.path(line.d, stroke=t.line, width=1.6 if dotted else 1.3, dash="dotted" if dotted else None)
            c.path(_arrowhead(line.tip, line.direction), fill=t.line)
        report = problems.append
        for p in self.placed:
            p.block.draw(c, p.x, p.y, p.w, p.h, t, report)
        for line in self.links:
            if line.label:
                x, y, w, h = self._label_box(line)
                c.rect(x, y, w, h, t.paper, t.line, 0.8, h / 2)
                c.text_centered(line.label, x + w / 2, y + h / 2, t.small, "medium", t.ink)
        c.close_group()
        if c.min_text is not None and c.min_text < t.min_text:
            problems.append(f"smallest text is {c.min_text:g} px; the theme's minimum is {t.min_text:g} px")
        svg = c.svg(self.title, t.paper, desc=self.describe())
        return svg, Report(svg, c.width, c.height, c.min_text, problems)

    def describe(self) -> str:
        """Plain-text description: blocks left to right, their tiles and states, groups, lines and the key.
        It is also written into the SVG as <desc>, for screen readers."""
        from .describe import describe
        return describe(self)

    def set_statuses(self, statuses: Dict[str, str]) -> None:
        """Set states by name, for example from a file the team updates: {"Fraud checks": "live"}.

        Names match panel titles, panel names and tile labels. "live" clears a state back to the default look.
        Raises ValueError naming any unknown state or any name that matches nothing.
        """
        bad = sorted({s for s in statuses.values() if s not in self.theme.statuses})
        if bad:
            raise ValueError(f"unknown state(s) {', '.join(map(repr, bad))}; this theme has: "
                             f"{', '.join(self.theme.statuses)}")
        found = set()

        def apply(block, *names):
            for n in names:
                if n is not None and n in statuses:
                    block.status = None if statuses[n] == "live" else statuses[n]
                    found.add(n)

        def visit(block):
            body = getattr(block, "body", None)
            items = getattr(block, "items", None)
            if body is not None:
                apply(block, getattr(block, "title", None), getattr(block, "name", None))
                for b in body:
                    visit(b)
            elif items is not None:
                for b in items:
                    visit(b)
            elif hasattr(block, "label") and hasattr(block, "status"):
                apply(block, block.label, getattr(block, "name", None))

        for p in self.placed:
            visit(p.block)
        missing = [n for n in statuses if n not in found]
        if missing:
            raise ValueError(f"no panel or tile is named {', '.join(map(repr, missing))}")

    def save(self, path: str, png: bool = True, scale: float = 2.0, strict: bool = True) -> Report:
        """Write path.svg (and path.png at `scale`), print a one-line summary, then raise LayoutError if the checks
        found problems and strict is True. The files are written either way."""
        base = path[:-4] if path.lower().endswith((".svg", ".png")) else path
        folder = os.path.dirname(os.path.abspath(base))
        os.makedirs(folder, exist_ok=True)
        svg, report = self.render()
        with open(base + ".svg", "w", encoding="utf-8") as fh:
            fh.write(svg)
        report.files.append(base + ".svg")
        if png:
            if _render.svg_to_png(base + ".svg", base + ".png", report.width, report.height, scale):
                report.files.append(base + ".png")
            else:
                print("PNG skipped: Chrome or Chromium not found (set the CHROME environment variable to its path)")
        print(f"wrote {', '.join(report.files)}: {report.width:g} x {report.height:g} px; "
              f"smallest text {report.min_text:g} px; {len(report.problems)} layout problem(s)")
        if report.problems and strict:
            raise LayoutError(report.problems, report.files)
        return report

    # ----------------------------------------------------------------------------------------------- header

    def _legend_items(self) -> List[Tuple[str, str, str]]:
        t = self.theme
        used = set().union(*(p.block.statuses() for p in self.placed))
        if "" in used and "live" in t.statuses:
            used.add("live")
        items = [("status", key, t.statuses[key].label) for key in t.statuses if key in used]
        kinds = {l.kind for l in self.links}
        items += [("line", kind, t.link_labels[kind]) for kind in t.link_labels if kind in kinds]
        return items

    def _item_w(self, item) -> float:
        return (34 if item[0] == "line" else 30) + text_width(item[2], self.theme.legend) + 22

    def _header_plan(self, content_w: float) -> dict:
        t = self.theme
        m = t.margin
        tag_w = text_width(self.tag, 13, "bold") + 24 if self.tag else 0
        title_w = tag_w + (12 if tag_w else 0) + text_width(self.title, t.title, "bold")
        sub_w = text_width(self.subtitle, t.subtitle) if self.subtitle else 0
        note_w, note_h = self.note.measure(t) if self.note else (0, 0)
        min_w = 2 * m + max(title_w, sub_w) + (48 + note_w if self.note else 0)
        width = max(content_w + 2 * m, min_w, 640)
        room = width - 2 * m - (note_w + 40 if self.note else 0)
        rows: List[List[Tuple[str, str, str]]] = []
        if self.legend:
            row: List[Tuple[str, str, str]] = []
            used = 0.0
            for item in self._legend_items():
                w = self._item_w(item)
                if row and used + w > room:
                    rows.append(row)
                    row, used = [], 0.0
                row.append(item)
                used += w
            if row:
                rows.append(row)
        steps = bool(self.steps) and self.legend
        h = 34 + (24 if self.subtitle else 0) + (14 if rows or steps else 0) + 24 * len(rows) + (24 if steps else 0)
        return {"width": width, "height": max(h, note_h), "rows": rows, "steps": steps, "tag_w": tag_w,
                "note_w": note_w, "centre": content_w + 2 * m < width}

    def _draw_header(self, c: Canvas, plan: dict) -> None:
        t = self.theme
        m = t.margin
        x, cy = m, m + 15
        if self.tag:
            w = plan["tag_w"]
            c.rect(x, cy - 14, w, 28, t.paper, t.tag_color, 1.4, 14)
            c.text_centered(self.tag, x + w / 2, cy, 13, "bold", t.tag_color)
            x += w + 12
        c.text(self.title, x, cy + cap_height(t.title, "bold") / 2, t.title, "bold", t.ink)
        y = m + 34
        if self.subtitle:
            c.text(self.subtitle, m, y + 10 + cap_height(t.subtitle) / 2, t.subtitle, "regular", t.muted)
            y += 24
        if plan["rows"] or plan["steps"]:
            y += 14
        for row in plan["rows"]:
            lx, ly = m, y + 10
            for kind, key, label in row:
                if kind == "status":
                    st = t.statuses[key]
                    c.rect(lx, ly - 7, 22, 14, st.fill, st.stroke, 1.2, 3, st.dash)
                    lx += 30
                else:
                    dotted = key == "planned"
                    c.line(lx, ly, lx + 22, ly, t.line, 1.6 if dotted else 1.3, "dotted" if dotted else None)
                    c.path(_arrowhead((lx + 28, ly), (1.0, 0.0)), fill=t.line)
                    lx += 34
                lx += c.text(label, lx, ly + cap_height(t.legend) / 2, t.legend, "regular", t.ink) + 22
            y += 24
        if plan["steps"]:
            lx, ly = m, y + 10
            lx += c.text(self.steps_label + ":", lx, ly + cap_height(t.legend, "semibold") / 2, t.legend,
                         "semibold", t.ink) + 12
            tone = t.tone("default")
            for n, label in self.steps:
                c.circle(lx + 9, ly, 9, t.badge_fill or tone)
                c.text_centered(str(n), lx + 9, ly, 10.5, "bold", t.badge_text or t.paper)
                lx += 24 + c.text(label, lx + 24, ly + cap_height(t.legend) / 2, t.legend, "regular", t.ink) + 18
        if self.note:
            nw, nh = self.note.measure(t)
            self.note.draw(c, plan["width"] - m - nw, m, nw, nh, t, lambda s: None)


# ---------------------------------------------------------------------------------------------------- geometry

def _anchor(p: Placed, side: str, offset: float) -> Point:
    if side == "left":
        return p.x, p.y + offset
    if side == "right":
        return p.right, p.y + offset
    if side == "top":
        return p.x + offset, p.y
    return p.x + offset, p.bottom


def _p(pt: Point) -> str:
    return f"{fmt(pt[0])} {fmt(pt[1])}"


def _bezier(p0, p1, p2, p3, s) -> Point:
    u = 1 - s
    return (u ** 3 * p0[0] + 3 * u * u * s * p1[0] + 3 * u * s * s * p2[0] + s ** 3 * p3[0],
            u ** 3 * p0[1] + 3 * u * u * s * p1[1] + 3 * u * s * s * p2[1] + s ** 3 * p3[1])


def _simplify(pts: List[Point]) -> List[Point]:
    out: List[Point] = []
    for pt in pts:
        if out and math.dist(out[-1], pt) < 0.01:
            continue
        if len(out) >= 2:
            a, b = out[-2], out[-1]
            if abs((b[0] - a[0]) * (pt[1] - b[1]) - (b[1] - a[1]) * (pt[0] - b[0])) < 0.01 and \
                    (b[0] - a[0]) * (pt[0] - b[0]) + (b[1] - a[1]) * (pt[1] - b[1]) >= 0:
                out[-1] = pt
                continue
        out.append(pt)
    return out


def _rounded(pts: List[Point], radius: float) -> str:
    d = [f"M{_p(pts[0])}"]
    for i in range(1, len(pts) - 1):
        a, b, c = pts[i - 1], pts[i], pts[i + 1]
        r = min(radius, math.dist(a, b) / 2, math.dist(b, c) / 2)
        u = ((b[0] - a[0]) / math.dist(a, b), (b[1] - a[1]) / math.dist(a, b))
        v = ((c[0] - b[0]) / math.dist(b, c), (c[1] - b[1]) / math.dist(b, c))
        d.append(f"L{_p((b[0] - u[0] * r, b[1] - u[1] * r))}Q{_p(b)} {_p((b[0] + v[0] * r, b[1] + v[1] * r))}")
    d.append(f"L{_p(pts[-1])}")
    return "".join(d)


def _sample(pts: List[Point], step: float) -> List[Point]:
    out = []
    for a, b in zip(pts, pts[1:]):
        n = max(1, int(math.dist(a, b) / step))
        out += [(a[0] + (b[0] - a[0]) * i / n, a[1] + (b[1] - a[1]) * i / n) for i in range(n)]
    out.append(pts[-1])
    return out


def _arrowhead(tip: Point, direction: Point) -> str:
    dx, dy = direction
    bx, by = tip[0] - dx * ARROW, tip[1] - dy * ARROW
    px, py = -dy * 3.6, dx * 3.6
    return f"M{_p(tip)}L{_p((bx + px, by + py))}L{_p((bx - px, by - py))}Z"


def _separation(a, b) -> float:
    """Distance between two rectangles; negative when they overlap."""
    dx = max(b[0] - (a[0] + a[2]), a[0] - (b[0] + b[2]))
    dy = max(b[1] - (a[1] + a[3]), a[1] - (b[1] + b[3]))
    return max(dx, dy)


def _contains(outer, inner) -> bool:
    return (outer[0] <= inner[0] and outer[1] <= inner[1] and
            outer[0] + outer[2] >= inner[0] + inner[2] and outer[1] + outer[3] >= inner[1] + inner[3])
