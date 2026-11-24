# open-diagram

Draw and check architecture diagrams from JSON or Mermaid.

```bash
cargo install open-diagram-cli --locked
open-diagram check plan.json
open-diagram draw plan.json
```

Commands: `draw`, `check`, `describe`, `png`, `new`. PNG output needs Chrome, Chromium or Edge (`CHROME` when the usual locations miss it).

The library is the `open-diagram` crate. This command embeds that crate's fonts and cloud icons, and those files keep the licenses named there. Repository: <https://github.com/Orrae-Labs/open-diagram-rs>
