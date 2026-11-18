"""Turn an SVG into a PNG with headless Chrome (Chrome with no window). Chromium and Edge also work."""
from __future__ import annotations

import math
import os
import pathlib
import shutil
import subprocess
import tempfile
from typing import Optional

_PATHS = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
]
_NAMES = ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "chrome", "microsoft-edge"]


def find_chrome() -> Optional[str]:
    """Return a Chrome-family browser path: the CHROME environment variable first, then common install places."""
    env = os.environ.get("CHROME")
    if env and os.path.exists(env):
        return env
    for path in _PATHS:
        if os.path.exists(path):
            return path
    for name in _NAMES:
        found = shutil.which(name)
        if found:
            return found
    return None


def svg_to_png(svg_path: str, png_path: str, width: float, height: float, scale: float = 2.0,
               chrome: Optional[str] = None) -> bool:
    """Screenshot the SVG at its own size times `scale`. Returns False if no browser was found or it failed."""
    chrome = chrome or find_chrome()
    if not chrome:
        return False
    w, h = math.ceil(width), math.ceil(height)
    png_path = os.path.abspath(png_path)
    with open(svg_path, encoding="utf-8") as fh:
        svg = fh.read()
    with tempfile.TemporaryDirectory() as tmp:
        page = os.path.join(tmp, "page.html")
        with open(page, "w", encoding="utf-8") as fh:
            fh.write("<!doctype html><html><head><meta charset='utf-8'><style>html,body{margin:0;padding:0;"
                     "background:#fff;overflow:hidden}svg{display:block}</style></head><body>" + svg +
                     "</body></html>")
        cmd = [chrome, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--no-first-run",
               "--no-default-browser-check", f"--user-data-dir={os.path.join(tmp, 'profile')}",
               f"--force-device-scale-factor={scale:g}", f"--window-size={w},{h}", f"--screenshot={png_path}",
               pathlib.Path(page).as_uri()]
        try:
            subprocess.run(cmd, capture_output=True, timeout=180)
        except (OSError, subprocess.TimeoutExpired):
            return False
    if not os.path.exists(png_path):
        return False
    try:
        from PIL import Image
    except ImportError:
        return True
    target = (round(w * scale), round(h * scale))
    with Image.open(png_path) as im:
        if im.size != target and im.size[0] >= target[0] and im.size[1] >= target[1]:
            im.crop((0, 0) + target).save(png_path)
    return True
