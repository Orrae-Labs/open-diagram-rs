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
        "planned": Status("Planned", "#5C636B", "#F1F2F4", "dotted"),
        "retiring": Status("Retiring", "#B42318", "#FEF0EE"),
    }


def default_tones() -> Dict[str, str]:
    return {
        "default": "#E2E5E9",  # light gray
        "neutral": "#E2E5E9",
        "accent": "#FFE08A",   # soft yellow, for a panel that should stand out
        "dark": "#2B3036",     # charcoal; its header text turns white
    }


@dataclass
class Theme:
    # colors
    ink: str = "#1F2328"
    muted: str = "#5E6874"
    faint: str = "#8A939D"
    paper: str = "#FFFFFF"
    line: str = "#3F454C"
    panel_fill: str = "#F7F7F8"
    panel_stroke: str = "#C5CCD3"
    tile_fill: str = "#FFFFFF"
    tile_stroke: str = "#2B3036"
    header_text: str = "#1F2328"  # on light headers
    header_sub: str = "#5A6067"
    header_text_on_dark: str = "#FFFFFF"  # used instead when the header color is dark
    header_sub_on_dark: str = "#D5D9DE"
    group_fill: str = "#F2F3F5"
    group_stroke: str = "#A3A8B0"
    note_fill: str = "#FFF8EC"
    note_stroke: str = "#D9B77E"
    tag_color: str = "#2B3036"
    badge_fill: Optional[str] = "#FFC83D"  # step number circles; None: white in headers, the default tone in the key
    badge_text: Optional[str] = "#1F2328"  # step numbers; None: the header color in headers, white in the key
    graph_fill: str = "#FBFBFC"
    graph_grid: str = "#EBECEF"
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

    def header_colors(self, background: str):
        """Title and subtitle colors for a header: whichever of the light-header or dark-header pair reads better."""
        dark = (_luminance(background) + 0.05) / (_luminance(self.header_text) + 0.05)
        light = (_luminance(self.header_text_on_dark) + 0.05) / (_luminance(background) + 0.05)
        if dark >= light:
            return self.header_text, self.header_sub
        return self.header_text_on_dark, self.header_sub_on_dark


def _luminance(hex_color: str) -> float:
    """Relative brightness of a color from 0 (black) to 1 (white), as used for contrast checks."""
    h = hex_color.lstrip("#")
    def channel(v):
        v = int(v, 16) / 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    r, g, b = (channel(h[i:i + 2]) for i in (0, 2, 4))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b
