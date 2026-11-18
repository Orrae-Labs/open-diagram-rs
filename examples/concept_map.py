"""A map of how git stores a project: its objects and how they point to each other, with one worked example.
Run: python3 examples/concept_map.py"""
import os

from open_diagram import Card, Diagram, Graph, Panel, Row

HERE = os.path.dirname(os.path.abspath(__file__))

MAP = Graph(
    nodes={"branch": "Branch", "tag": "Tag", "commit": "Commit", "parent": "Parent commit", "tree": "Tree",
           "blob": "Blob", "subtree": "Subfolder tree"},
    links=[("branch", "commit", "points to"), ("tag", "commit", "points to"), ("commit", "parent", "follows"),
           ("commit", "tree", "saves"), ("tree", "blob", "lists file"), ("tree", "subtree", "lists folder")],
    at={"branch": (0, 0), "commit": (1.5, 0), "tree": (3, 0), "blob": (4.5, 0), "tag": (0, 2),
        "parent": (1.5, 2), "subtree": (4.5, 2)},
)

EXAMPLE = Card("Example: one commit", [
    ("Snapshot", "Tree 9c1e4a"),
    ("Parent", "Commit 52f0b7"),
    ("Message", "Fix sign-in on phones"),
    ("Branch", "main points here"),
], intro="One commit, as git stores it:")


def build() -> Diagram:
    d = Diagram("How git stores a project", tag="EXAMPLE")
    d.add(Panel("Objects and how they connect", Row(MAP, EXAMPLE),
                sub="Each object is named by a hash of its contents"))
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "concept_map"))
