"""A release pipeline, from pull request to production, with a rollback lane underneath.
Run: python3 examples/pipeline.py"""
import os

from open_diagram import Diagram, Panel, Tile, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))

STEPS = [(1, "Cache every build"), (2, "Scan every image"), (3, "Release to 5% first")]


def build() -> Diagram:
    d = Diagram("From pull request to production", tag="EXAMPLE", steps=STEPS, steps_label="This quarter")
    (code,), (build_,), (checks,), (release,) = d.columns([
        [Panel("Code", Tiles(["Pull request", "Code review"], cols=1), footer="Owner: Every team")],
        [Panel("Build", Tiles(["Compile", "Unit tests", "Container image", Tile("Build cache", status="building")],
                              cols=1), step=1, footer="Owner: Build team")],
        [Panel("Checks", Tiles(["Integration tests", Tile("Security scan", status="building"),
                                Tile("Load test", status="planned")], cols=1), step=2,
               footer="Owner: Quality team")],
        [Panel("Release", Tiles(["Staging", Tile("Canary, 5% of users", status="trial"), "Production"], cols=1),
               step=3, footer="Owner: Platform team")],
    ], gap=72, valign="center")
    d.link(code, build_)
    d.link(build_, checks)
    d.link(checks, release)
    bottom = max(p.bottom for p in d.placed) + 44
    d.link(release, code, start="bottom", end="bottom", via_y=bottom,
           label="If errors rise: roll back and tell the author")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "pipeline"))
