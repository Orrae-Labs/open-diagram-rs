"""Convert an SVG to a PNG: python -m open_diagram picture.svg [--scale 2]"""
import argparse
import re
import sys

from .render import svg_to_png


def main() -> None:
    ap = argparse.ArgumentParser(prog="python -m open_diagram", description=__doc__)
    ap.add_argument("svg")
    ap.add_argument("--scale", type=float, default=2.0)
    a = ap.parse_args()
    with open(a.svg, encoding="utf-8") as fh:
        head = fh.read(2000)
    w = re.search(r'width="([\d.]+)"', head)
    h = re.search(r'height="([\d.]+)"', head)
    if not (w and h):
        sys.exit("the SVG needs width and height attributes")
    out = a.svg[:-4] + ".png" if a.svg.lower().endswith(".svg") else a.svg + ".png"
    if not svg_to_png(a.svg, out, float(w.group(1)), float(h.group(1)), a.scale):
        sys.exit("PNG not written: Chrome or Chromium not found (set the CHROME environment variable to its path)")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
