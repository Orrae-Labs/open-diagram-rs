# Rust

Build a diagram from blocks, place them, and set its colors and sizes with a theme. The crate is `open-diagram`.

## Building blocks

| Type | What it draws |
| --- | --- |
| `Panel::new(title)` then `.section`, `.subtitle`, `.status`, `.step`, `.footer`, `.tone`, `.name` | The main box: a colored header, sections stacked inside, an optional footer such as "Owner: Data team" |
| `Tiles::new(items, cols, label)` | A grid of tiles with one cell size |
| `Tile::new(label)` then `.subtitle`, `.status`, `.icon` | One small box. `.icon` takes a PNG or SVG path, or an icon name resolved by JSON |
| `Resource::new(...)` | A standalone cloud resource: large icon above its workload label and service name |
| `Card::new(title, rows, intro)` | A worked example: rows of name and value |
| `Graph::new(...)` | Things on a grid, joined by labelled lines. The build reports labels that touch nodes |
| `Row::new(items, gap)` | Sections side by side inside a panel |
| `Text::new(text)` | A wrapped paragraph |
| `Note::new(title, lines)` | A boxed note for the top right |

`Block` is the enum these types convert into.

## Diagram

| Method | What it does |
| --- | --- |
| `Diagram::new(title)` then `.subtitle`, `.tag`, `.note`, `.theme`, `.steps` | Start a picture |
| `add(block, x, y)` | Place a block |
| `right_of(refer, block, gap, align, width, name)` and `below(...)` | Place a block next to another. `gap` and `width` are `Option<f64>`. `align` is `center`, `left`, `right`, `top` or `bottom` |
| `columns(cols, origin, gap, row_gap, valign, same_width)` and `stack(blocks, x, y, gap)` | Place columns left to right, or one column top to bottom. `valign` is `top`, `center` or `bottom` |
| `ring(blocks, x, y, gap, start_angle, center)` | Place blocks around a circle. Returns `(ring, center)` |
| `group(label, members, sub, pad)` | A shaded area around some blocks |
| `enclosure(label, members, opts)` | A boundary around resources or nested enclosures. `EnclosureOpts` carries border, style, pad, fill, stroke and tone. `style = "panel"` adds the sloped lid and a filled header |
| `link(a, b, spec)` | An arrow. `LinkSpec::default().kind("planned").label("new").start("auto").end("auto").via_y(y).dash("dashed")` |
| `set_statuses(map)` | Set states by panel title, id or tile label |
| `render()` | SVG text and a `Report` |
| `save(path, png, scale, strict)` | Write `<path>.svg` and, when `png` is true, `<path>.png`. When `strict` is true and problems remain, the files are already written and the error is `SaveError::Layout` |
| `describe()` | What the picture says, in plain words. The same text is the SVG `<desc>` |

JSON goes through `from_spec` / `load_spec`. Mermaid goes through `from_mermaid` / `load_mermaid`. `SpecError` and `MermaidError` name the place of the mistake.

## Statuses, colors and sizes

Panels have light gray headers and yellow step numbers. When any panel uses the yellow `accent` header, step numbers turn charcoal so yellow keeps one meaning. Each panel has a sloped top and each tile a raised edge. `Theme` fields `lid` and `depth` set to `0.0` turn that off. `tone` `accent` is a soft yellow header. `tone` `dark` is charcoal. Header text turns white on a dark header.

Statuses are Live, Being built, In trial, Planned and Retiring. Add one with `theme.statuses.insert("pilot".into(), Status { ... })`.

The smallest text drawn is recorded on `Report.min_text`. The theme minimum is 10 px.

## Checked pictures

`examples::pictures()` returns every named builder the golden test renders. `examples::quickstart()` is the search diagram in the README. Cloud templates load from the files shipped inside this crate.
