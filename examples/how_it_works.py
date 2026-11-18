"""How open-diagram turns a script into a picture, drawn with open-diagram. Run: python3 examples/how_it_works.py"""
import os

from open_diagram import Diagram, Panel, Tile, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))

STAGES = [
    ("Your script", "Written by you or an agent", ["Words in lists", "Panels and tiles", "Lines and groups"]),
    ("Measure", "From the font file", ["Letter widths", "Kerning pairs", "Line wrapping"]),
    ("Place", "Sizes come from Measure", ["Columns", "Next to, below", "Shaded groups"]),
    ("Check", "Every rule, every build", ["Overlaps", "Gaps under 16 px", "Lines through boxes", "Labels on boxes",
                                         "Text under 10 px"]),
    ("Draw", "Same look everywhere", ["Text as outlines", "Curves and lanes", "Key from what is used",
                                      Tile("3D icons", status="planned")]),
    ("Output", "Files first, then problems", ["SVG file", "PNG file", "Problem report"]),
]


def build() -> Diagram:
    d = Diagram("How open-diagram turns a script into a picture", tag="HOW IT WORKS")
    placed = d.columns([[Panel(name, Tiles(items, cols=1), sub=sub, tone="accent" if name == "Check" else "default")]
                        for name, sub, items in STAGES], gap=64, valign="center")
    panels = [col[0] for col in placed]
    for a, b in zip(panels, panels[1:]):
        d.link(a, b)
    bottom = max(p.bottom for p in d.placed) + 44
    d.link(panels[-1], panels[0], start="bottom", end="bottom", via_y=bottom,
           label="Each problem comes back by name: fix it and run again")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "how_it_works"))
