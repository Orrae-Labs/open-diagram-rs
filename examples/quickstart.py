"""The smallest useful diagram: three panels, two lines, a key. Run: python3 examples/quickstart.py"""
import os

from open_diagram import Diagram, Panel, Tile

HERE = os.path.dirname(os.path.abspath(__file__))


def build() -> Diagram:
    d = Diagram("How a search request is answered", tag="EXAMPLE")
    app = d.add(Panel("Web app", ["Search box", "Results page", "Filters"], footer="Owner: Web team"))
    api = d.right_of(app, Panel("Search API", [Tile("Checks access"), Tile("Caches results", status="building")],
                                footer="Owner: Platform team"), align="center")
    index = d.right_of(api, Panel("Search index", ["Product text", Tile("Spelling fixes", status="planned")],
                                  status="planned"), align="center")
    d.link(app, api)
    d.link(api, index, kind="planned", label="new")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "quickstart"))
