"""A dense systems map for an invented video platform: where videos come from, how they are processed and stored,
who watches them, what state each part is in, and which changes come first. Run: python3 examples/systems_map.py"""
import os

from open_diagram import Card, Diagram, Graph, Note, Panel, Row, Tile, Tiles

HERE = os.path.dirname(os.path.abspath(__file__))

STEPS = [(1, "Encode on GPUs"), (2, "One record for each video"), (3, "Recommendations")]

VIDEO_MAP = Graph(
    nodes={"channel": "Channel", "video": "Video", "rendition": "Rendition", "caption": "Caption",
           "view": "View", "language": "Language"},
    links=[("channel", "video", "publishes"), ("video", "rendition", "encoded as"), ("video", "caption", "has"),
           ("video", "view", "watched in"), ("caption", "language", "written in")],
    at={"channel": (0, 0), "video": (1.5, 0), "rendition": (3, 0), "language": (0, 2), "caption": (1.5, 2),
        "view": (3, 2)},
)

VIDEO_CARD = Card("Example: video 7Q4M", [
    ("Length", "12 min 40 s"),
    ("Renditions", "1080p, 720p, 480p"),
    ("Captions", "English, Spanish"),
    ("Status", "Ready to play"),
])


def build() -> Diagram:
    d = Diagram(
        "Quillfin Video: from upload to screen",
        subtitle="What runs today, what is being built, and what comes next",
        tag="EXAMPLE",
        note=Note("Example", "Quillfin Video is an invented company.", "Every system, owner and status is made up."),
        steps=STEPS, steps_label="Rollout",
    )
    (uploads,), (processing, record), (storage, recommend), (players, health) = d.columns([
        [Panel("Where videos come from", Tiles(["Creator app", "Web uploader", "Partner API",
                                               Tile("Live streams", status="trial")], cols=2),
               sub="Four ways in", footer="Owner: Creator team")],
        [Panel("Processing", Tiles(["Upload checks", "Transcoding", Tile("GPU encoding", status="building"),
                                   Tile("Auto captions", status="planned")], cols=2),
               step=1, footer="Owner: Media team"),
         Panel("Video record", Row(VIDEO_MAP, VIDEO_CARD), sub="Everything known about each video",
               step=2, status="building", footer="Owner: Media team")],
        [Panel("Storage", Tiles(["Original files", "Renditions"], label="Files"),
               Tiles(["Metadata database", Tile("Search index", status="trial"),
                      Tile("Cold archive", status="planned")], label="Records"),
               footer="Owner: Platform team"),
         Panel("Recommendations", Tiles(["Watch history", "Ranking model"]), step=3, status="planned",
               footer="Owner: Not yet named")],
        [Panel("Player apps", Tiles(["Web", "Phones", Tile("TV", status="building")], cols=1),
               tone="neutral", footer="Owner: Apps team"),
         Panel("Service health", Tiles(["Playback errors", "Start-up time",
                                        Tile("Old dashboards", status="retiring")], cols=1),
               tone="neutral", footer="Owner: Reliability team")],
    ], gap=88, row_gap=40)
    d.group("Media pipeline", [processing, record], sub="Run by the media team")

    d.link(uploads, processing)
    d.link(processing, record, start="bottom", end="top")
    d.link(record, storage)
    d.link(record, recommend, kind="planned")
    d.link(storage, players)
    d.link(storage, health, label="logs")
    d.link(recommend, players, kind="planned")

    top = min(p.y for p in d.placed) - 80
    bottom = max(p.bottom for p in d.placed) + 48
    d.link(uploads, players, start="top", end="top", via_y=top, label="Live streams skip processing")
    d.link(uploads, health, start="bottom", end="bottom", via_y=bottom,
           label="Upload failures go straight to service health")
    return d


if __name__ == "__main__":
    build().save(os.path.join(HERE, "..", "docs", "images", "systems_map"))
