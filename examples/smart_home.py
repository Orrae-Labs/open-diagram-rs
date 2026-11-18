"""An invented smart home system, from the sensors in each room to the apps on a phone.
Run: python3 examples/smart_home.py"""
import os

from open_diagram import Diagram, Note, Panel, Tile, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))


def build() -> Diagram:
    d = Diagram("A smart home, from sensor to phone", tag="EXAMPLE",
                note=Note("Example", "An invented product.", "Every part and status is made up."))
    (devices,), (hub,), (cloud,), (apps,) = d.columns([
        [Panel("Devices", Tiles(["Thermostat", "Door lock", "Lights", "Motion sensors"], cols=2), sub="In every room")],
        [Panel("Home hub", Tiles(["Device pairing", "Local rules", Tile("Works offline", status="building")], cols=1),
               sub="Talks to every device")],
        [Panel("Cloud", Tiles(["Device registry", "Usage history", Tile("Firmware updates", status="building")],
                              cols=1), footer="Owner: Platform team")],
        [Panel("Apps", Tiles(["Phone app", Tile("Voice assistants", status="trial"),
                              Tile("Web dashboard", status="retiring")], cols=1), tone="neutral",
               footer="Owner: Apps team")],
    ], gap=80, valign="center")
    d.link(devices, hub)
    d.link(hub, cloud)
    d.link(cloud, apps)
    top = min(p.y for p in d.placed) - 56
    bottom = max(p.bottom for p in d.placed) + 44
    d.link(apps, hub, start="top", end="top", via_y=top, label="At home, the phone talks to the hub directly")
    d.link(cloud, devices, start="bottom", end="bottom", via_y=bottom, kind="planned",
           label="New firmware sent to devices at night")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "smart_home"))
