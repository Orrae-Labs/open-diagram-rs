"""open-diagram: building blocks for dense architecture diagrams that check their own layout."""
from .blocks import Block, Card, Graph, Note, Panel, Row, Text, Tile, Tiles
from .diagram import Diagram, LayoutError, Link, Placed, Report
from .fonts import text_width, wrap
from .render import find_chrome, svg_to_png
from .theme import Status, Theme

__version__ = "0.1.0"

__all__ = [
    "Block", "Card", "Diagram", "Graph", "LayoutError", "Link", "Note", "Panel", "Placed", "Report", "Row",
    "Status", "Text", "Theme", "Tile", "Tiles", "find_chrome", "svg_to_png", "text_width", "wrap",
]
