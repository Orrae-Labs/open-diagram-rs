# Comparison sources

These files produced the pictures in [../comparison.md](../comparison.md). Each tool ran with default settings.

| Picture | Source | Command |
| --- | --- | --- |
| open-diagram 0.5.0 (Python) | `photo-upload.json` | `open-diagram draw photo-upload.json` |
| Mermaid CLI 12.2.0 | `photo-upload.mmd` | `npx @mermaid-js/mermaid-cli -i photo-upload.mmd -o mermaid.svg` |
| D2 0.9.0 | `photo-upload.d2` | `d2 photo-upload.d2 d2.svg` |
| Excalidraw 0.18.1 | `photo-upload.mmd` | `parseMermaidToExcalidraw` (@excalidraw/mermaid-to-excalidraw 3.6.0), then `exportToSvg` |
| archify 3.3.0 | `photo-upload.architecture.json` | `node bin/archify.mjs export-svg architecture photo-upload.architecture.json archify.svg --theme light --quality showcase` |

Chrome converted each SVG to a PNG at twice its size.

The times were measured on one Mac on November 23, 2026: one warm-up run, then the median of five runs, from starting the command to the finished SVG. Excalidraw has no command, so its time covers the conversion and export inside Chrome, after the page and libraries had loaded.

| Tool | Time |
| --- | --- |
| open-diagram 0.5.0 | 0.59 s |
| Mermaid | 0.76 s |
| archify | 0.28 s |
| D2 | 0.017 s |
| Excalidraw | 0.020 s |
