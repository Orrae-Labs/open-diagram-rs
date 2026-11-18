"""Colors, sizes and statuses. Change a Theme to restyle every diagram at once."""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, Optional


@dataclass(frozen=True)
class Status:
    """How one state is shown: a word, a border color, a fill color and a border style."""

    label: str
    stroke: str
    fill: str
    dash: Optional[str] = None  # None (solid), "dashed" or "dotted"


def default_statuses() -> Dict[str, Status]:
    return {
        "live": Status("Live", "#27323F", "#FFFFFF"),
        "building": Status("Being built", "#B26B00", "#FFF5E5", "dashed"),
        "trial": Status("In trial", "#6B4FD8", "#F4F1FE", "dashed"),
        "planned": Status("Planned", "#2C6DB5", "#EEF4FC", "dotted"),
        "retiring": Status("Retiring", "#B42318", "#FEF0EE"),
    }


def default_tones() -> Dict[str, str]:
    return {
        "default": "#163C40",
        "accent": "#0C6C67",
        "neutral": "#46505B",
        "warm": "#80501A",
        "plum": "#4A3966",
    }


@dataclass
class Theme:
    # colors
    ink: str = "#1D2733"
    muted: str = "#5E6874"
    faint: str = "#8A939D"
    paper: str = "#FFFFFF"
    line: str = "#3D4F5C"
    panel_fill: str = "#F6F7F5"
    panel_stroke: str = "#C5CCD3"
    tile_fill: str = "#FFFFFF"
    tile_stroke: str = "#27323F"
    header_text: str = "#FFFFFF"
    header_sub: str = "#CFE0E0"
    group_fill: str = "#F1F4F3"
    group_stroke: str = "#9AA7AC"
    note_fill: str = "#FFF8EC"
    note_stroke: str = "#D9B77E"
    tag_color: str = "#0C6C67"
    graph_fill: str = "#FBFCFB"
    graph_grid: str = "#ECEFEE"
    graph_line: str = "#97A3AB"
    tones: Dict[str, str] = field(default_factory=default_tones)
    statuses: Dict[str, Status] = field(default_factory=default_statuses)
    link_labels: Dict[str, str] = field(default_factory=lambda: {
        "flow": "Data flows today", "planned": "Connection to build"})

    # type sizes, in pixels
    title: float = 26
    subtitle: float = 13.5
    legend: float = 12
    panel_title: float = 14.5
    panel_sub: float = 11
    tile: float = 12
    tile_sub: float = 10.5
    body: float = 11
    small: float = 10.5
    min_text: float = 10

    # spacing, in pixels
    pad: float = 12
    tile_gap: float = 8
    section_gap: float = 12
    tile_h: float = 30
    line_h: float = 14.5
    header_h: float = 36
    header_h_sub: float = 50
    panel_min_w: float = 200
    tile_max_w: float = 230
    radius: float = 6
    col_gap: float = 72
    row_gap: float = 28
    min_gap: float = 16
    margin: float = 40

    def status(self, key: Optional[str]) -> Optional[Status]:
        if key is None:
            return None
        if key not in self.statuses:
            raise ValueError(f"unknown status {key!r}; this theme has: {', '.join(self.statuses)}")
        return self.statuses[key]

    def tone(self, key: str) -> str:
        if key.startswith("#"):
            return key
        if key not in self.tones:
            raise ValueError(f"unknown tone {key!r}; this theme has: {', '.join(self.tones)}")
        return self.tones[key]
