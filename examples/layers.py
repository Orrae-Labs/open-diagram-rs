"""Stacked layers with a side panel: a web product from the people who use it down to its data.
Run: python3 examples/layers.py"""
import os

from open_diagram import Diagram, Panel, Tile, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))

LAYERS = [
    ("People", ["Shoppers", "Store staff", "Partners"], "accent"),
    ("Apps", ["Website", "Mobile app", Tile("Staff tablets", status="retiring")], "default"),
    ("Services", ["Search", "Orders", "Payments", Tile("Notifications", status="building")], "default"),
    ("Data", ["Main database", "Search index", Tile("Event log", status="planned")], "neutral"),
]


def build() -> Diagram:
    d = Diagram("A web product, top to bottom", tag="EXAMPLE")
    layers = d.stack([Panel(name, Tiles(items, cols=len(items)), tone=tone) for name, items, tone in LAYERS], gap=36)
    for upper, lower in zip(layers, layers[1:]):
        d.link(upper, lower, start="bottom", end="top")
    side = Panel("Sign-in and access", Tiles(["Single sign-on", Tile("Passkeys", status="trial"), "Audit log"],
                                             cols=1), footer="Owner: Security team")
    middle = (layers[1].y + layers[2].bottom) / 2
    access = d.add(side, layers[0].right + 96, middle - side.measure(d.theme)[1] / 2)
    d.link(access, layers[1], start="left", end="right", kind="planned")
    d.link(access, layers[2], start="left", end="right", kind="planned")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "layers"))
