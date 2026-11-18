"""The smallest useful diagram: three panels, two lines, a key. Run: python3 examples/quickstart.py"""
import os

from open_diagram import Diagram, Panel, Tile

HERE = os.path.dirname(os.path.abspath(__file__))


def build() -> Diagram:
    d = Diagram("How an order reaches the warehouse", tag="EXAMPLE")
    shop = d.add(Panel("Online shop", ["Product pages", "Basket", "Checkout"], footer="Owner: Web team"))
    orders = d.right_of(shop, Panel("Order service", [Tile("Takes payment"), Tile("Checks stock", status="building")],
                                    footer="Owner: Platform team"), align="center")
    stock = d.right_of(orders, Panel("Warehouse", ["Pick and pack", Tile("Same-day dispatch", status="planned")],
                                     status="planned"), align="center")
    d.link(shop, orders)
    d.link(orders, stock, kind="planned", label="new")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "quickstart"))
