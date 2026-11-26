# Changelog

## Unreleased

- A quoted Mermaid label or line label can hold `;` and `%%`. `a["a;b"] --> b` and `a -->|"x; y"| b` used to be cut at the separator and reported as unreadable. Outside quotes, `;` still separates statements and `%%` still starts a comment. Ported from open-diagram e99256ae9bfce8def8df11ed361cbe861b5ad2e8.

## 0.6.2

The version number jumps from 0.1.0 to 0.6.2. From here on, versions count up from 0.6.2.

- Lines can be drawn as straight runs that meet at right angles. Set `"routing": "orthogonal"` at the top of a JSON file, or on one link. In Rust, use `Diagram::routing("orthogonal")` or `LinkSpec::routing`. A Mermaid flowchart whose `curve` is `step`, `stepBefore` or `stepAfter` gets right angles too. Lines that share a gap get their own tracks, ordered to avoid crossing each other. The default is still curved, so existing pictures don't change. Ported from open-diagram bd276519b62301fcf76d91aacab95c30eff4a925.
- `from_mermaid_with` reads a Mermaid flowchart with a given routing. `from_mermaid` keeps its 0.1.0 signature.
- Mermaid `RL` and `BT` flowcharts are laid out in reverse step order instead of being mirrored afterwards, so the space above a group's title stays on the side where the group starts. Ported from open-diagram 49982c0b93766aec4c2483da24afbda18e60c239.
- `base64` is now 0.23. The SVG output is unchanged.
- The README and the guides under `docs/` were rewritten. Release steps moved to `docs/releasing.md`.

## 0.1.0

The first crates.io release of the Rust tree. It was published without a git tag.

- `open-diagram-rs` measures every label from the bundled Inter fonts and writes a deterministic SVG.
- `open-diagram-rs-cli` is the `open-diagram-rs` command. It reads JSON and Mermaid.
- `open-diagram-rs-mcp` is the stdio MCP server. Its tools are `format_guide`, `template`, `check`, `describe`, and `draw`.
