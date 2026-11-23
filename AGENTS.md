# open-diagram

A pure-Python library that turns a short script into a dense architecture diagram (SVG, plus PNG through a headless browser). Every label is measured from the font before anything is placed. The build reports layout problems as plain sentences that name the parts involved, so the picture can be checked without looking at it.

Runtime depends on fontTools only. Pillow is optional (the `png` extra) and is used to crop screenshots. The MCP toolkit is an optional extra (`mcp`, Python 3.10+). The core targets Python 3.9+. PNG output needs Chrome, Chromium, or Edge (`CHROME` if the usual locations miss it). SVG output does not need a browser.

## Map

| Path | Role |
| --- | --- |
| `src/open_diagram/fonts.py` | `Face` (fontTools): glyph outlines, advance widths, GPOS pair kerning. `text_width`, `cap_height`, `wrap`, `fmt` |
| `src/open_diagram/fonts/` | Inter 4.1, four weights, under the SIL OFL (`OFL.txt`) |
| `src/open_diagram/svg.py` | `Canvas`: SVG primitives. `text()` emits outline paths with a `<title>` holding the string |
| `src/open_diagram/theme.py` | `Theme` (colors, statuses, tones, type sizes, spacing, 3D `lid`/`depth`), `Status`, `shade()` |
| `src/open_diagram/blocks.py` | `Block` and `Resource`, `Tile`, `Tiles`, `Text`, `Card`, `Note`, `Graph`, `Row`, `Panel`. Shared `_panel_frame` for panels and resource containers |
| `src/open_diagram/diagram.py` | `Diagram`: placement, `link`/`group`/`enclosure`, routing, `_check()`, legend and header, `render()`, `save()` |
| `src/open_diagram/describe.py` | `describe(diagram)`: the plain-text description behind `Diagram.describe()` and the SVG `<desc>` |
| `src/open_diagram/spec.py` | `from_spec(dict)` / `load_spec(path)`: JSON descriptions. `SpecError` names the place of every mistake |
| `src/open_diagram/mermaid.py` | `parse()` and `from_mermaid(text)`: a Mermaid flowchart subset and its layered, banded layout |
| `src/open_diagram/cli.py` | The `open-diagram` command (`draw`, `check`, `describe`, `png`) and `load(path)` for `.json`, `.mmd`, and `.py` |
| `src/open_diagram/render.py` | `find_chrome()`, `svg_to_png()` (headless browser screenshot, optional Pillow crop) |
| `src/open_diagram/mcp_server.py` | MCP server `open-diagram-mcp`: tools `format_guide`, `check`, `draw`, `describe`. The toolkit is imported only inside `create_server()` |
| `src/open_diagram/__init__.py` | Public API is `__all__` |
| `examples/` | One script per example. Each defines `build() -> Diagram` and saves to `docs/images/<name>` |
| `templates/` | Cloud resource and ETL starters, with cached provider artwork and attribution. `build() -> Diagram`, rendered offline |
| `docs/images/` | Committed SVG and PNG output. The README embeds them |
| `tests/test_open_diagram.py` | unittest suite. `EXAMPLES` lists every example that must build with zero problems |
| `tests/test_cloud.py` | Resource measurement and attachment, enclosure nesting and styles, template checks, byte-identical existing SVGs |
| `tests/test_mcp_server.py` | Helper tests, plus protocol tests over memory and stdio (skipped without the `mcp` extra) |
| `action.yml` | Composite GitHub Action: installs the package from its own checkout, then runs `open-diagram check` or `draw` |
| `.github/workflows/diagrams.yml` | Runs that action on this repo's examples |
| `.github/workflows/test.yml` | Runs the tests on Python 3.10 and 3.14 |
| `.github/workflows/publish.yml` | On a `vX.Y.Z` GitHub release, builds the package and uploads it to PyPI by trusted publishing |

Version is `0.5.0` in both `pyproject.toml` and `src/open_diagram/__init__.py`.

## How a build flows

1. `Block.measure(theme)` returns `(w, h)` computed from text widths.
2. `Diagram.add` / `right_of` / `below` / `stack` / `columns` / `ring` place blocks and return `Placed` records.
3. `Diagram.link`, `Diagram.group`, and `Diagram.enclosure` record lines and shaded areas.
4. `render()` runs `_route()` and `_shape()` for line paths, `_check()` for problems, then draws. It returns `Report(svg, width, height, min_text, problems, files)`.
5. `save(path)` writes the SVG and PNG, prints a one-line summary, then raises `LayoutError(problems, files)` when `strict=True` and problems exist. Files are written before it raises.

`open-diagram --soundtrack` prints The Gathering Field's Spotify link and is hidden from `--help`.

## Properties of the system

- **Measure, then draw.** `draw()` may receive more space than `measure()` asked for, never less. Less space is reported by name. Text is not clipped or shrunk.
- **Everything drawn stays inside the measured box.** The 3D lid on panels and the raised faces on tiles are included in `measure()`, so overlap and crossing checks see the whole shape.
- **Text is outlines, measured from the font.** Widths come from `text_width` / `wrap`. Drawing goes through `Canvas.text`. The SVG contains no `<text>` elements and no width guessed from character counts.
- **Output is deterministic.** The same script produces a byte-identical SVG. The SVG has no timestamps, randomness, absolute paths, or environment-dependent values.
- **Problem messages are part of the interface.** Each one names parts by their labels and, where it can, says how to fix it. Stable substrings: `overlap`, `px apart`, `crosses`, `sits on`, `needs N px`, `smallest text`, `group ... overlaps`, `sits on the title of group`, `facing away`, `graph: the label ... touches`, `graph: the line ... runs through`.
- **Colors come from `Theme`.** Blocks read colors from the theme. Derived shades use `shade()`. Header text color is chosen by contrast in `Theme.header_colors()`.
- **Minimum text size.** Nothing is drawn below `theme.min_text` (10 px). `Report.min_text` records the smallest size actually drawn.
- **`describe()` has a stable shape.** `->` for lines, `labeled "..."`, state words in parentheses, `[step N, state]` after panel titles. The same text is the SVG `<desc>`.
- **JSON and Mermaid errors name the place** (`columns[1][0].tiles[2]`, `line 7`).
- **The MCP server does not use stdout.** Stdout carries the protocol, so work is wrapped in `redirect_stdout(sys.stderr)`. It accepts JSON and Mermaid only, and writes only inside the output folder (`safe_name()`).
