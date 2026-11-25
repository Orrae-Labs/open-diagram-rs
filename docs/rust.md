# Rust

```bash
cargo add open-diagram-rs
```

The library crate is `open-diagram-rs`, imported as `open_diagram_rs`. It needs Rust 1.100 or later.

You build a picture by creating blocks, placing each one on a `Diagram`, and linking them. `render` lays everything out and returns the SVG along with any layout problems:

```rust
use open_diagram_rs::{Diagram, DiagramError, LinkSpec, Panel, Tile, Tiles};

fn search() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("How a search request is answered");
    let app = diagram.add(
        Panel::new("Web app")
            .section(Tiles::new(vec![Tile::new("Search box"), Tile::new("Results page")], None, None)?.into())
            .footer("Owner: Web team"),
        0.0,
        0.0,
    )?;
    let api = diagram.right_of(
        &app,
        Panel::new("Search API").section(
            Tiles::new(
                vec![Tile::new("Checks access"), Tile::new("Caches results").status("building")],
                None,
                None,
            )?
            .into(),
        ),
        None,
        "center",
        None,
        None,
    )?;
    diagram.link(&app, &api, LinkSpec::default())?;
    Ok(diagram)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut diagram = search()?;
    let report = diagram.render()?;
    for problem in &report.problems {
        eprintln!("{problem}");
    }
    std::fs::write("search.svg", &report.svg)?;
    Ok(())
}
```

To read the other formats instead, use `from_spec` or `load_spec` for JSON and `from_mermaid` or `load_mermaid` for Mermaid. Their errors, `SpecError` and `MermaidError`, name the place of the mistake.

## Blocks

| Type | What it draws |
| --- | --- |
| `Panel::new(title)`, then `.section`, `.subtitle`, `.status`, `.step`, `.footer`, `.tone`, `.name` | The main box: a header, sections stacked inside it, and an optional footer such as "Owner: Data team" |
| `Tiles::new(items, cols, label)` | A grid of tiles that share one cell size |
| `Tile::new(label)`, then `.subtitle`, `.status`, `.icon` | One small box. `.icon` takes the path of a PNG or SVG file, or an icon name such as `aws/s3` when the diagram comes from JSON |
| `Resource::new(...)` | A cloud resource: a large icon with its label and service name underneath |
| `Card::new(title, rows, intro)` | A worked example, as rows of names and values |
| `Graph::new(...)` | Points on a grid joined by labelled lines. `render` reports a label that touches a point |
| `Row::new(items, gap)` | Sections side by side inside a panel |
| `Text::new(text)` | A wrapped paragraph |
| `Note::new(title, lines)` | A boxed note for the top right corner |

Each of these converts into `Block` with `.into()`. `Panel::section` takes a `Block`.

## Diagram methods

| Method | What it does |
| --- | --- |
| `Diagram::new(title)`, then `.subtitle`, `.tag`, `.note`, `.theme`, `.steps` | Starts a picture |
| `add(block, x, y)` | Places a block at a position |
| `right_of(other, block, gap, align, width, name)` and `below(...)` | Places a block next to another one. `gap` and `width` are `Option<f64>`. `align` is `center`, `left`, `right`, `top` or `bottom` |
| `columns(cols, origin, gap, row_gap, valign, same_width)` | Places columns of blocks left to right. `valign` is `top`, `center` or `bottom` |
| `stack(blocks, x, y, gap)` | Places blocks in one column, top to bottom |
| `ring(blocks, x, y, gap, start_angle, center)` | Places blocks around a circle and returns `(ring, center)` |
| `group(label, members, sub, pad)` | Draws a shaded area around some blocks |
| `enclosure(label, members, opts)` | Draws a boundary around resources or other enclosures. `EnclosureOpts` sets the border, style, padding, fill, stroke and tone. `style = "panel"` adds a sloped lid and a filled header |
| `link(a, b, spec)` | Draws an arrow. For example: `LinkSpec::default().kind("planned").label("new").start("auto").end("auto").via_y(y).dash("dashed")` |
| `set_statuses(map)` | Sets states by panel title, panel id or tile label |
| `render()` | Returns a `Report` with the SVG text and the list of problems |
| `save(path, png, scale, strict)` | Writes `<path>.svg`, and `<path>.png` when `png` is true. With `strict`, it returns `SaveError::Layout` if problems remain, after writing the files |
| `describe()` | Returns what the picture says in plain words. The SVG carries the same text in its `<desc>` element |

## States, colors and sizes

The states are Live, Being built, In trial, Planned and Retiring. To add your own, insert into `theme.statuses`, for example `theme.statuses.insert("pilot".into(), Status { ... })`.

Panel headers are light gray by default. `tone` `accent` makes a header soft yellow, and `tone` `dark` makes it charcoal with white text. Step numbers are yellow, unless some panel uses the `accent` tone. Then they turn charcoal, so that yellow means only one thing in the picture.

Panels have a sloped top and tiles have a raised edge. Set the `Theme` fields `lid` and `depth` to `0.0` for flat boxes.

No text is drawn smaller than the theme's minimum, 10 px. `Report.min_text` records the smallest size in the picture.

## Example pictures

`examples::pictures()` returns the builder of every picture in [docs/images](images). The test suite renders each one and compares it with the committed SVG. `examples::quickstart()` builds the search diagram from the README.
