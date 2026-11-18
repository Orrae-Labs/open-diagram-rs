"""A dense systems map for an invented furniture store: where orders come from, what handles them, who uses the
results, what state each part is in, and which changes come first. Run: python3 examples/systems_map.py"""
import os

from open_diagram import Card, Diagram, Graph, Note, Panel, Row, Tile, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))

STEPS = [(1, "Fraud checks in checkout"), (2, "One record for each order"), (3, "Book returns online")]

ORDER_MAP = Graph(
    nodes={"customer": "Customer", "order": "Order", "payment": "Payment", "item": "Item",
           "delivery": "Delivery", "return": "Return"},
    links=[("customer", "order", "places"), ("order", "payment", "paid by"), ("order", "item", "includes"),
           ("order", "delivery", "ships as"), ("item", "return", "may come back as")],
    at={"customer": (0, 0), "order": (1.5, 0), "payment": (3, 0), "return": (0, 2), "item": (1.5, 2),
        "delivery": (3, 2)},
)

ORDER_CARD = Card("Example: order 10482", [
    ("Customer", "Online account"),
    ("Items", "Sofa, two cushions"),
    ("Paid by", "Card, in full"),
    ("Delivery", "Two-person, Tuesday"),
])


def build() -> Diagram:
    d = Diagram(
        "Pinemoor Home: from order to doorstep",
        subtitle="What runs today, what is being built, and what comes next",
        tag="EXAMPLE",
        note=Note("Example", "Pinemoor Home is an invented company.", "Every system, owner and status is made up."),
        steps=STEPS, steps_label="Rollout",
    )
    (ways_in,), (checkout, record), (fulfil, returns), (reports, service) = d.columns([
        [Panel("Where orders come from", Tiles(["Website", "Mobile app", Tile("Showroom tablets", status="retiring"),
                                               Tile("Marketplace listings", status="trial")], cols=2),
               sub="Four ways in", footer="Owner: Digital team")],
        [Panel("Checkout", Tiles(["Basket", "Payments", Tile("Fraud checks", status="building"),
                                 Tile("Buy now, pay later", status="planned")], cols=2),
               step=1, footer="Owner: Payments team"),
         Panel("Order record", Row(ORDER_MAP, ORDER_CARD), sub="Every order kept in one place",
               step=2, status="building", footer="Owner: Platform team")],
        [Panel("Fulfilment", Tiles(["Pick and pack", "Stock counts"], label="Warehouse"),
               Tiles(["Two-person delivery", Tile("Same-day vans", status="trial"),
                      Tile("Parcel lockers", status="planned")], label="Delivery"),
               footer="Owner: Operations"),
         Panel("Returns", Tiles(["Book a collection", "Refund on pickup"]), step=3, status="planned",
               footer="Owner: Not yet named")],
        [Panel("Reporting", Tiles(["Daily sales", "Stock cover", Tile("Demand forecast", status="trial")], cols=1),
               tone="neutral", footer="Owner: Finance"),
         Panel("Customer service", Tiles(["Order lookup", "Delivery changes",
                                          Tile("Chat assistant", status="building")], cols=1),
               tone="neutral", footer="Owner: Service team")],
    ], gap=88, row_gap=40)
    d.group("Order platform", [checkout, record], sub="Run by the platform team")

    d.link(ways_in, checkout)
    d.link(checkout, record, start="bottom", end="top")
    d.link(record, fulfil)
    d.link(record, returns, kind="planned")
    d.link(fulfil, reports)
    d.link(fulfil, service, label="updates")
    d.link(returns, reports, kind="planned")

    top = min(p.y for p in d.placed) - 80
    bottom = max(p.bottom for p in d.placed) + 48
    d.link(ways_in, reports, start="top", end="top", via_y=top, label="Web traffic, sent nightly")
    d.link(ways_in, service, start="bottom", end="bottom", via_y=bottom,
           label="Marketplace questions come straight to service")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "systems_map"))
