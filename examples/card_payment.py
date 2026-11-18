"""What happens when you tap a card in a shop, simplified. Run: python3 examples/card_payment.py"""
import os

from open_diagram import Diagram, Panel, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))

STEPS = [(1, "Tap"), (2, "Ask the card's bank"), (3, "Approve or decline"), (4, "Money moves later")]


def build() -> Diagram:
    d = Diagram("What happens when you tap a card", subtitle="Simplified: one in-store payment, start to finish",
                tag="EXAMPLE", steps=STEPS, steps_label="Order")
    (shopper,), (shop,), (acquirer,), (network,), (issuer,) = d.columns([
        [Panel("Shopper", Tiles(["Card", "Phone wallet"], cols=1), step=1)],
        [Panel("Shop", Tiles(["Card reader", "Checkout system"], cols=1))],
        [Panel("Shop's bank", Tiles(["Payment processor", "Fraud screening"], cols=1), sub="The acquirer", step=4)],
        [Panel("Card network", Tiles(["Routes the request", "Network rules"], cols=1), step=2)],
        [Panel("Card's bank", Tiles(["Checks the balance", "Fraud checks", "Approves or declines"], cols=1),
               sub="The issuer", step=3)],
    ], gap=64, valign="center")
    d.link(shopper, shop, label="tap")
    d.link(shop, acquirer)
    d.link(acquirer, network)
    d.link(network, issuer)
    top = min(p.y for p in d.placed) - 56
    bottom = max(p.bottom for p in d.placed) + 44
    d.link(issuer, shop, start="top", end="top", via_y=top, label="The answer comes back the same way, in seconds")
    d.link(issuer, acquirer, start="bottom", end="bottom", via_y=bottom, label="Money moves later, in a batch")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "card_payment"))
