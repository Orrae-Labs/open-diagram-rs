"""The built-in templates and their icons: where they are, what each is for, and how to read one.

The templates are the files in the repository's templates/ folder. The package ships that same folder as
open_diagram/templates, so `open-diagram new` and the MCP server can use them after `pip install`.
"""
from __future__ import annotations

import os
import re
from typing import List, Optional, Sequence, Tuple

TEMPLATE_TYPES = (".json", ".mmd")
ICON_TYPES = (".svg", ".png", ".jpg", ".jpeg")
_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")


def template_dir() -> str:
    """The templates folder: inside the installed package, or the repository's templates/ in a source checkout."""
    here = os.path.dirname(os.path.abspath(__file__))
    installed = os.path.join(here, "templates")
    if os.path.isdir(installed):
        return installed
    return os.path.join(os.path.dirname(os.path.dirname(here)), "templates")


def icon_dir() -> str:
    """The built-in icon folder. An icon named 'aws/s3' is the file assets/aws/s3.svg in it."""
    return os.path.join(template_dir(), "assets")


def table(readme: Optional[str] = None) -> List[Tuple[str, str]]:
    """(file name, purpose) for each row of the first table in templates/README.md whose first column is
    'Template'. The README is the one place a template's purpose is written; the website reads it too."""
    path = readme or os.path.join(template_dir(), "README.md")
    rows, header = [], None
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line.startswith("|"):
                if header is not None and rows:
                    break
                continue
            cells = [c.strip() for c in line.strip("|").split("|")]
            if header is None:
                if cells and cells[0].lower() == "template":
                    header = [c.lower() for c in cells]
                continue
            if set(line) <= set("|-: "):
                continue
            link = _LINK.search(cells[0])
            name = os.path.basename(link.group(1)) if link else cells[0].strip("`")
            purpose = cells[header.index("purpose")] if "purpose" in header else ""
            rows.append((name, purpose))
    return rows


def names() -> List[str]:
    """Every template that `open-diagram new` and the MCP server can hand out, by name without extension."""
    return [os.path.splitext(f)[0] for f, _ in table() if os.path.splitext(f)[1] in TEMPLATE_TYPES]


def path(name: str) -> str:
    """The file for a template name such as 'azure_web_app'. Raises ValueError listing the names."""
    stem = os.path.splitext(os.path.basename(name or ""))[0]
    for ext in TEMPLATE_TYPES:
        candidate = os.path.join(template_dir(), stem + ext)
        if stem in names() and os.path.isfile(candidate):
            return candidate
    raise ValueError(f"no template named {name!r}; templates: {', '.join(names())}")


def read(name: str) -> str:
    with open(path(name), encoding="utf-8") as fh:
        return fh.read()


def icon_names(folder: Optional[str] = None) -> List[str]:
    """Icon names in a folder, such as 'aws/s3', for listing in messages and guides."""
    root = folder or icon_dir()
    found = []
    for base, _, files in os.walk(root):
        for f in files:
            stem, ext = os.path.splitext(f)
            if ext.lower() in ICON_TYPES:
                rel = os.path.relpath(os.path.join(base, stem), root)
                found.append(rel.replace(os.sep, "/"))
    return sorted(found)


def find_icon(name: str, folders: Sequence[str]) -> Optional[str]:
    """The file for an icon name inside one of the folders, or None. The name is a path inside a folder, with or
    without its extension ('aws/s3' or 'aws/s3.svg'). Absolute paths, '..' and files outside the folders are never
    returned, so a diagram description cannot read other files on the computer."""
    clean = (name or "").replace("\\", "/")
    parts = clean.split("/")
    if not clean or clean.startswith("/") or ":" in clean or any(p in ("", ".", "..") for p in parts):
        return None
    ext = os.path.splitext(clean)[1].lower()
    tries = [clean] if ext in ICON_TYPES else [clean + e for e in ICON_TYPES]
    for folder in folders:
        root = os.path.realpath(folder)
        for t in tries:
            real = os.path.realpath(os.path.join(root, *t.split("/")))
            if os.path.isfile(real) and real.startswith(root + os.sep):
                return real
    return None
