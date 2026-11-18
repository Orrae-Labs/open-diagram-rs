"""How an invented team handles an outage, from the first alert to the lessons learned.
Run: python3 examples/incident_response.py"""
import os

from open_diagram import Diagram, Note, Panel, Tile, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))


def build() -> Diagram:
    d = Diagram("When the site goes down: from alert to lesson learned", tag="EXAMPLE",
                note=Note("Example", "An invented team's process.", "Every tool and status is made up."))
    (detect,), (respond,), (fix, tell), (learn,) = d.columns([
        [Panel("Detect", Tiles(["Error-rate alerts", "Uptime checks", Tile("Unusual traffic", status="trial")],
                               cols=1), footer="Owner: Reliability team")],
        [Panel("Respond", Tiles(["Page the on-call engineer", "Open an incident channel",
                                 Tile("Assign roles automatically", status="building")], cols=1),
               footer="Owner: On-call rotation")],
        [Panel("Fix", Tiles(["Roll back", "Add servers", "Turn off a feature"], cols=1)),
         Panel("Tell people", Tiles(["Status page", "Support team", Tile("Email affected users", status="planned")],
                                    cols=1), tone="neutral")],
        [Panel("Learn", Tiles(["Timeline", "Blameless review", "Follow-up tasks"], cols=1), tone="accent",
               footer="Owner: Incident lead")],
    ], gap=136, row_gap=32, valign="center")
    d.link(detect, respond)
    d.link(respond, fix)
    d.link(respond, tell, label="every 30 min")
    d.link(fix, learn)
    d.link(tell, learn, label="what users saw")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "incident_response"))
