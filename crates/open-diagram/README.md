# open-diagram-rs

Measure every label from the bundled Inter fonts, place the blocks, and report layout problems by name. The SVG is deterministic and contains outline paths rather than `<text>` elements.

```toml
[dependencies]
open-diagram-rs = "0.1.0"
```

`from_spec` and `from_mermaid` build a `Diagram`. `render` returns the SVG. `save` also writes a PNG when Chrome, Chromium or Edge is available.

The command-line tool is the `open-diagram-rs-cli` crate. Templates and the Inter fonts ship inside this package. The package license is `MIT AND OFL-1.1 AND Apache-2.0 AND CC-BY-ND-2.0`. Code is MIT. The Inter fonts are [SIL Open Font License 1.1](fonts/OFL.txt). GCP icons are Apache-2.0. AWS icons, including the QuickSight PNG, are CC BY-ND 2.0. Azure icons are MIT in the catalog that supplied them, and they also keep Microsoft's icon terms, which are not an SPDX license: do not crop, flip, rotate, or reshape them, and keep the product name beside the icon. See `templates/assets/README.md`.

This 0.1.0 release reproduces the diagram behavior of open-diagram 0.6.0.

Repository: <https://github.com/Orrae-Labs/open-diagram-rs>
