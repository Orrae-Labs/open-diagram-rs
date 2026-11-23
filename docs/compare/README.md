# Comparison sources

The pictures in [../comparison.md](../comparison.md) come from these files. Each tool drew its picture with default settings.

| Picture | Source | Command |
| --- | --- | --- |
| open-diagram 0.5.0 | `photo-upload.json` | `open-diagram draw photo-upload.json` |
| Mermaid CLI 12.2.0 | `photo-upload.mmd` | `npx @mermaid-js/mermaid-cli -i photo-upload.mmd -o mermaid.svg` |
| D2 0.9.0 | `photo-upload.d2` | `d2 photo-upload.d2 d2.svg` |
| Excalidraw 0.18.1 | `photo-upload.mmd` | `parseMermaidToExcalidraw` (@excalidraw/mermaid-to-excalidraw 3.6.0), then `exportToSvg` |
| archify 3.3.0 | `photo-upload.architecture.json` | `node bin/archify.mjs preview architecture photo-upload.architecture.json out.html --quality showcase` |

Chrome saved each SVG or page as a PNG at twice its size.
