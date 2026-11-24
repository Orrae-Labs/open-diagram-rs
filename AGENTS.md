# open-diagram

Rust library and `open-diagram` command. Every label is measured from the bundled Inter fonts, then the layout is checked. SVG bytes are deterministic.

## Change

1. Edit the crates under `crates/`. The library is `open-diagram`. The command crate is `open-diagram-cli` (binary `open-diagram`). The MCP crate is `open-diagram-mcp` (binary `open-diagram-mcp`). `quickstart-consumer` is `publish = false`.
2. Run `cargo test --workspace --all-features`, then `cargo clippy --workspace --all-targets --all-features -- -D warnings`, then `cargo audit`.
3. Done when all three succeed and every picture in `examples::pictures()` still matches the git blob of `docs/images/<name>.svg`.

## Map

| Path | Role |
| --- | --- |
| `crates/open-diagram` | Library. Fonts, templates and three example inputs are inside this package |
| `crates/open-diagram/src/font.rs` | Inter outlines, advances, GPOS pair kerning |
| `crates/open-diagram/src/svg.rs` | `Canvas`. Text is outline paths plus a `<title>` |
| `crates/open-diagram/src/theme.rs` | `Theme`, `Status`, `shade` |
| `crates/open-diagram/src/blocks.rs` | `Block`, `Panel`, `Tile`, `Tiles`, `Resource`, `Card`, `Graph`, `Note`, `Row`, `Text` |
| `crates/open-diagram/src/diagram.rs` | Placement, links, groups, enclosures, routing, checks, `render`, `save` |
| `crates/open-diagram/src/describe.rs` | Plain-text description, also the SVG `<desc>` |
| `crates/open-diagram/src/spec.rs` | JSON. `SpecError` names the place |
| `crates/open-diagram/src/mermaid.rs` | Mermaid flowchart subset |
| `crates/open-diagram/src/catalog.rs` | Templates and icon lookup. Bytes are embedded for `cargo install` |
| `crates/open-diagram/src/render.rs` | `find_chrome`, `svg_to_png` |
| `crates/open-diagram/src/examples.rs` | Picture builders. The golden test renders these |
| `crates/open-diagram-cli` | `draw`, `check`, `describe`, `png`, `new`, hidden `--soundtrack` |
| `crates/open-diagram-mcp` | stdio MCP server. Tools `format_guide`, `template`, `check`, `describe`, `draw` |
| `docs/images/` | Committed SVG and PNG oracles |
| `examples/` | JSON and Mermaid files a person can pass to the command. Copies used by the library live in the crate and the tests require the bytes to match |
| `.github/workflows/ci.yml` | Test, clippy, rustfmt, and `cargo audit` on pinned runners. Push to `main`, or a pull request after `borrow-checker` approves it |
| `.github/workflows/crate-provenance.yml` | Reusable build. The package job has no OIDC permission. A second job attests the uploaded `.crate` |
| `.github/workflows/publish.yml` | Trusted publishing after the attestation verifies |

## Invariants

- Measure, then draw. A draw call may receive more space than measure asked for. Less space is reported by name. Text is not clipped or shrunk.
- Everything drawn stays inside the measured box, including the 3D lid and raised tile faces.
- Widths come from `text_width` and `wrap`. The SVG has no `<text>` elements and no width guessed from character counts.
- The same inputs produce a byte-identical SVG. No timestamps, randomness, absolute paths, or host-dependent numbers. The renderer writes LF.
- Problem sentences are part of the interface. Stable substrings: `overlap`, `px apart`, `crosses`, `sits on`, `needs N px`, `smallest text`, `group ... overlaps`, `sits on the title of group`, `facing away`, `graph: the label ... touches`, `graph: the line ... runs through`.
- Colors come from `Theme`. Derived shades use `shade`. Nothing is drawn below `theme.min_text` (10 px).
- `describe()` shape: `->` for lines, `labeled "..."`, state words in parentheses, `[step N, state]` after panel titles.
- JSON and Mermaid errors name the place (`columns[1][0].tiles[2]`, `line 7`).
- Arithmetic that formats into the SVG uses `crate::num`. `mul_add` and `f64::midpoint` change those bytes. `base64` stays on 0.22. `ttf-parser` stays; its unmaintained advisory has no upgrade that keeps the outlines.

## Style

Clippy `pedantic` and `nursery` are warn at priority `-1` in the workspace `Cargo.toml`. That priority lets one lint be allowed without turning the group off. An allow is on one item, with a reason. The existing allow is `font.rs` `float_cmp`: SVG path `H` and `V` commands use exact coordinate equality before the numbers are formatted. `clippy.toml` sets `too-many-lines-threshold` to 200 because rustfmt counts wrapped lines and the longest formatted function is under that.

A comment records a constraint the code cannot show. A test that passes by weakening the assertion is a failure.

## Upstream

`upstream` is lukendeasy/open-diagram. Port a commit when it changes a diagram, an error sentence, or the command, and name that commit in the Rust change. `rust-port` keeps the 0.6.0 SVG bytes. This branch, `main`, is where a fix may differ from that snapshot. The pictures in `examples::pictures()` still have to match `docs/images`.

## Publishing

Trusted publishing only. The OIDC exchange is `rust-lang/crates-io-auth-action` in `publish.yml`. Do not add a crates.io token to the repo or the environment.

The attested files are the `.crate` packages from the package job in `crate-provenance.yml`. The publish job verifies that attestation and refuses to continue when a fresh `cargo package --locked` digest differs. `cargo publish` packs again and cannot upload the attested file. The registry token stays on the publish step. `quickstart-consumer` is not published.

The library license field is `MIT AND OFL-1.1 AND Apache-2.0 AND CC-BY-ND-2.0`. Azure artwork also keeps Microsoft's icon terms, which are not SPDX. Leave those identifiers in place.

Push this tree to `git@github.com:Orrae-Labs/open-diagram-rs.git`. The remote named `origin` in a checkout that still tracks `Orrae-Labs/open-diagram` is the Python mirror. Leave it unchanged.

## MCP

`open-diagram-mcp` speaks MCP over stdio. Its tools are `format_guide`, `template`, `check`, `describe`, and `draw`. Input is JSON or Mermaid. Files go in the folder named by `--out`, `$OPEN_DIAGRAM_OUT`, or `./open-diagram`. When `skills/` is visible, or `$OPEN_DIAGRAM_SKILLS` points at it, the SDK skill catalog serves those folders.

The command reads `.json`, `.mmd`, and `.mermaid`. It does not run a `.py` file. A Rust picture is a function that returns `Result<Diagram, DiagramError>`. `examples::pictures` and `quickstart-consumer` are that function.
