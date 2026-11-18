"""The open-diagram command.

    open-diagram draw FILE [-o OUT] [--no-png] [--scale 2] [--statuses FILE]
    open-diagram check FILE... [--statuses FILE]
    open-diagram describe FILE [--statuses FILE]
    open-diagram png FILE.svg [--scale 2]

FILE is a diagram description (.json), a Mermaid flowchart (.mmd or .mermaid), or a Python script with a build()
function that returns a Diagram (.py). `draw` and `check` exit with status 1 when the layout has problems, so they can
stop a build.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import types
from typing import List, Optional

from . import __version__
from .diagram import Diagram
from .render import svg_to_png

SOUNDTRACK = "The Gathering Field: https://open.spotify.com/artist/0BfQ8COB7K647uB6a0kMZn"


def load(path: str, statuses: Optional[str] = None, title: Optional[str] = None) -> Diagram:
    """Read a .json description, a .mmd/.mermaid flowchart or a .py script with build(), and return the Diagram."""
    ext = os.path.splitext(path)[1].lower()
    if ext == ".json":
        from .spec import load_spec
        d = load_spec(path)
    elif ext in (".mmd", ".mermaid"):
        from .mermaid import load_mermaid
        d = load_mermaid(path, title=title)
    elif ext == ".py":
        # Compile from the source every time. Python's saved bytecode can be reused when a script is edited
        # within the same second and keeps the same size, which an agent's quick edit-and-check loop can hit.
        source_path = os.path.abspath(path)
        with open(source_path, encoding="utf-8") as fh:
            source = fh.read()
        module = types.ModuleType("_open_diagram_script")
        module.__file__ = source_path
        folder = os.path.dirname(source_path)
        sys.path.insert(0, folder)
        try:
            exec(compile(source, source_path, "exec"), module.__dict__)
        finally:
            if folder in sys.path:
                sys.path.remove(folder)
        if not hasattr(module, "build"):
            raise SystemExit(f"{path}: needs a build() function that returns a Diagram")
        d = module.build()
        if not isinstance(d, Diagram):
            raise SystemExit(f"{path}: build() returned {type(d).__name__}, not a Diagram")
    else:
        raise SystemExit(f"{path}: unknown file type; use .json, .mmd, .mermaid or .py")
    if title and ext != ".mmd" and ext != ".mermaid":
        d.title = title
    if statuses:
        with open(statuses, encoding="utf-8") as fh:
            d.set_statuses(json.load(fh))
    return d


def _png(svg: str, scale: float) -> int:
    with open(svg, encoding="utf-8") as fh:
        head = fh.read(2000)
    w = re.search(r'width="([\d.]+)"', head)
    h = re.search(r'height="([\d.]+)"', head)
    if not (w and h):
        print(f"{svg}: the SVG needs width and height attributes", file=sys.stderr)
        return 2
    out = svg[:-4] + ".png" if svg.lower().endswith(".svg") else svg + ".png"
    if not svg_to_png(svg, out, float(w.group(1)), float(h.group(1)), scale):
        print("PNG not written: Chrome or Chromium not found (set the CHROME environment variable to its path)",
              file=sys.stderr)
        return 2
    print(f"wrote {out}")
    return 0


def main(argv: Optional[List[str]] = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    if "--soundtrack" in argv:
        print(SOUNDTRACK)
        return 0
    if argv and argv[0].lower().endswith(".svg"):          # older form: python -m open_diagram picture.svg
        argv = ["png"] + argv

    ap = argparse.ArgumentParser(prog="open-diagram", description="Draw and check diagrams that check their own layout.")
    ap.add_argument("--version", action="version", version=f"open-diagram {__version__}")
    ap.add_argument("--soundtrack", action="store_true", help=argparse.SUPPRESS)
    sub = ap.add_subparsers(dest="command", required=True)

    p = sub.add_parser("draw", help="write FILE's diagram as SVG and PNG; exit 1 if the layout has problems")
    p.add_argument("file")
    p.add_argument("-o", "--out", help="output path without extension (default: next to FILE)")
    p.add_argument("--no-png", action="store_true", help="write the SVG only")
    p.add_argument("--scale", type=float, default=2.0, help="PNG scale (default 2)")
    p.add_argument("--statuses", help="JSON file of name -> state, applied before drawing")
    p.add_argument("--title", help="title to use (Mermaid files default to their front-matter title)")

    p = sub.add_parser("check", help="build each FILE in memory and list layout problems; exit 1 if any")
    p.add_argument("files", nargs="+")
    p.add_argument("--statuses", help="JSON file of name -> state, applied before checking")

    p = sub.add_parser("describe", help="print a plain-text description of FILE's diagram")
    p.add_argument("file")
    p.add_argument("--statuses", help="JSON file of name -> state, applied first")
    p.add_argument("--title")

    p = sub.add_parser("png", help="convert an SVG to PNG with Chrome, Chromium or Edge")
    p.add_argument("svg")
    p.add_argument("--scale", type=float, default=2.0)

    a = ap.parse_args(argv)
    try:
        if a.command == "png":
            return _png(a.svg, a.scale)
        if a.command == "describe":
            print(load(a.file, a.statuses, a.title).describe())
            return 0
        if a.command == "draw":
            d = load(a.file, a.statuses, a.title)
            out = a.out or os.path.splitext(a.file)[0]
            report = d.save(out, png=not a.no_png, scale=a.scale, strict=False)
            for problem in report.problems:
                print(f"  - {problem}")
            return 1 if report.problems else 0
        failed = 0
        for path in a.files:
            _, report = load(path, a.statuses).render()
            print(f"{path}: {len(report.problems)} layout problem(s)")
            for problem in report.problems:
                print(f"  - {problem}")
            failed += bool(report.problems)
        return 1 if failed else 0
    except (ValueError, OSError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 2


def run() -> None:
    sys.exit(main())
