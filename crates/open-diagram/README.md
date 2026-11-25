# open-diagram-rs

A Rust library that draws architecture diagrams and checks their layout. You build a diagram from panels, tiles, lines and groups, or read one from JSON or a Mermaid flowchart. The library measures every label with the bundled Inter fonts, places everything, and names each layout problem: two boxes that overlap, a line that runs through a box, text that would need more room.

```toml
[dependencies]
open-diagram-rs = "0.1.0"
```

`from_spec` reads JSON and `from_mermaid` reads Mermaid, and each returns a `Diagram`. `Diagram::render` returns the SVG and the list of problems. `Diagram::save` writes the SVG, plus a PNG when Chrome, Chromium or Edge is installed.

The same inputs always produce the same SVG, byte for byte. Text is drawn as outlines rather than `<text>` elements, so the picture looks the same on machines without the font.

Version 0.1.0 draws the same pictures as open-diagram 0.6.0, the Python project it is ported from. For the command-line tool, install the `open-diagram-rs-cli` crate.

Documentation and examples: <https://github.com/Orrae-Labs/open-diagram-rs>

## License

The code is MIT. The package also bundles files under other licenses, so its license field is `MIT AND OFL-1.1 AND Apache-2.0 AND CC-BY-ND-2.0`:

- The Inter fonts use the [SIL Open Font License 1.1](fonts/OFL.txt).
- The Google Cloud icons are Apache-2.0.
- The AWS icons, including the QuickSight PNG, are CC BY-ND 2.0.
- The Azure icons are MIT in the catalog that supplied them, and also come with Microsoft's icon terms, which have no SPDX identifier. Those terms ask you not to crop, flip, rotate or reshape the icons, and to keep the product name beside each one.

`templates/assets/README.md` gives the source and terms of each icon set.
