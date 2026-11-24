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
TIMEOUT = 180
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


NOT_FOUND = "Chrome, Chromium or Edge was not found (set the CHROME environment variable to its path)"


def svg_to_png(svg_path: str, png_path: str, width: float, height: float, scale: float = 2.0,
               chrome: Optional[str] = None) -> bool:
    """Screenshot the SVG at its own size times `scale`. Returns False if no browser was found or it failed;
    a PNG already at png_path is then left as it was."""
    return png_error(svg_path, png_path, width, height, scale, chrome) is None


def png_error(svg_path: str, png_path: str, width: float, height: float, scale: float = 2.0,
              chrome: Optional[str] = None) -> Optional[str]:
    """Like svg_to_png, but returns None once the PNG is written, or a sentence saying why it was not."""
    chrome = chrome or find_chrome()
    if not chrome:
        return NOT_FOUND
    w, h = math.ceil(width), math.ceil(height)
    png_path = os.path.abspath(png_path)
    with open(svg_path, encoding="utf-8") as fh:
        svg = fh.read()
    with tempfile.TemporaryDirectory() as tmp:
        page = os.path.join(tmp, "page.html")
        shot = os.path.join(tmp, "shot.png")        # written here first, so a failed run never touches png_path
        with open(page, "w", encoding="utf-8") as fh:
            fh.write("<!doctype html><html><head><meta charset='utf-8'><style>html,body{margin:0;padding:0;"
                     "background:#fff;overflow:hidden}svg{display:block}</style></head><body>" + svg +
                     "</body></html>")
        cmd = [chrome, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--no-first-run",
               "--no-default-browser-check", f"--user-data-dir={os.path.join(tmp, 'profile')}",
               f"--force-device-scale-factor={scale:g}", f"--window-size={w},{h}", f"--screenshot={shot}",
               pathlib.Path(page).as_uri()]
        try:
            run = subprocess.run(cmd, capture_output=True, timeout=TIMEOUT)
        except subprocess.TimeoutExpired:
            return f"the browser at {chrome} did not finish within {TIMEOUT} seconds"
        except OSError as e:
            return f"the browser at {chrome} could not be started ({e.strerror or e})"
        if run.returncode != 0:
            return f"the browser at {chrome} stopped with exit code {run.returncode}"
        if not os.path.exists(shot):
            return f"the browser at {chrome} finished without writing a PNG"
        try:
            from PIL import Image
        except ImportError:
            Image = None
        if Image is not None:
            target = (round(w * scale), round(h * scale))
            try:
                im = Image.open(shot)
                im.load()
            except OSError as e:                    # Pillow's "cannot identify image file" is an OSError
                return f"the browser at {chrome} wrote a file that is not a readable PNG ({e})"
            with im:
                if im.size != target and im.size[0] >= target[0] and im.size[1] >= target[1]:
                    im.crop((0, 0) + target).save(png_path, format="PNG")
                    return None
        shutil.copyfile(shot, png_path)
    return None
