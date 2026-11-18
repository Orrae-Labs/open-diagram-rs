"""A small map of the things an invented community library keeps track of, with one worked example.
Run: python3 examples/concept_map.py"""
import os

from open_diagram import Card, Diagram, Graph, Panel, Row

HERE = os.path.dirname(os.path.abspath(__file__))

MAP = Graph(
    nodes={"member": "Member", "loan": "Loan", "copy": "Copy", "title": "Title", "branch": "Branch",
           "event": "Event", "hold": "Hold"},
    links=[("member", "loan", "borrows"), ("loan", "copy", "of"), ("copy", "title", "is a"),
           ("copy", "branch", "kept at"), ("member", "hold", "places"), ("hold", "title", "on"),
           ("branch", "event", "hosts")],
    at={"member": (0, 0), "loan": (1.5, 0), "copy": (3, 0), "title": (3, 2), "hold": (1.5, 2),
        "branch": (4.5, 0), "event": (4.5, 2)},
)

EXAMPLE = Card("Example: one loan", [
    ("Member", "Card 20931"),
    ("Copy", "Copy 3 of 5, main branch"),
    ("Title", "A field guide to moths"),
    ("Due", "In three weeks"),
], intro="One loan, as the library stores it:")


def build() -> Diagram:
    d = Diagram("What the library keeps track of", tag="EXAMPLE")
    d.add(Panel("Things and how they connect", Row(MAP, EXAMPLE), sub="One entry per thing",
                footer="Owner: Library systems team"))
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "concept_map"))
