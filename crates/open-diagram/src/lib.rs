mod blocks;
pub mod catalog;
mod describe;
mod diagram;
pub mod examples;
mod error;
mod font;
mod mermaid;
#[cfg(test)]
mod messages;
mod num;
mod render;
mod spec;
mod svg;
mod theme;

pub use blocks::{Block, Card, Graph, Note, Panel, Resource, Row, Text, Tile, Tiles};
pub use diagram::{Diagram, Enclosed, EnclosureOpts, GroupId, LineView, LinkSpec, Placed, Report};
pub use error::{DiagramError, LayoutError, MermaidError, SaveError, SpecError};
pub use mermaid::{from_mermaid, load_mermaid, parse};
pub use render::{find_chrome, svg_to_png};
pub use spec::{from_spec, from_spec_with, load_spec, load_spec_with, panel_from_spec, theme_from_spec};
pub use font::{cap_height, lines_width, outline, text_width, wrap};
pub use num::{fmt, py_g, py_round};
pub use theme::{shade, Status, Theme};

#[cfg(test)]
mod tests {
    use super::{outline, text_width};

    #[test]
    fn inter_hello_width_matches_fonttools() {
        let width = text_width("Hello", 16.0, "regular");
        assert!((width - 38.5625).abs() < 1e-9, "{width}");
    }

    /// Repository root when this checkout is the one that holds `docs/images`. A crate built outside that checkout returns `None`.
    fn oracle_root() -> Option<std::path::PathBuf> {
        let start = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let output = std::process::Command::new("git").arg("-C").arg(&start).args(["rev-parse", "--show-toplevel"]).output();
        let toplevel = output.ok().filter(|out| out.status.success()).and_then(|out| {
            let text = String::from_utf8(out.stdout).ok()?;
            let path = std::path::PathBuf::from(text.trim());
            path.join("docs/images").is_dir().then_some(path)
        });
        assert!(
            toplevel.is_some() || !start.join("docs/images").is_dir(),
            "docs/images is in this checkout but git did not identify it as the oracle repository"
        );
        toplevel
    }

    fn committed_blob(root: &std::path::Path, name: &str) -> Vec<u8> {
        let object = format!("HEAD:docs/images/{name}.svg");
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["cat-file", "blob", &object])
            .output()
            .unwrap_or_else(|err| panic!("{name}: {err}"));
        assert!(output.status.success(), "{name}: git cat-file failed: {}", String::from_utf8_lossy(&output.stderr));
        output.stdout
    }

    #[test]
    fn committed_svgs_match_byte_for_byte() {
        // The oracle is the git blob. Skip when this crate is not inside the repository that contains docs/images.
        // cargo package extracts under target/, and that directory is still inside this repository, so the comparison runs there too.
        let Some(root) = oracle_root() else {
            return;
        };
        let mut mismatches = Vec::new();
        let pictures = super::examples::pictures();
        for (name, build) in &pictures {
            let mut diagram = build().unwrap_or_else(|err| panic!("{name}: {err}"));
            let report = diagram.render().unwrap_or_else(|err| panic!("{name}: {err}"));
            assert!(report.problems.is_empty(), "{name}: {:?}", report.problems);
            let committed = committed_blob(&root, name);
            let got = report.svg.into_bytes();
            if got != committed {
                let n = got.len().min(committed.len());
                let mut at = n;
                for (i, (a, b)) in got.iter().zip(committed.iter()).enumerate() {
                    if a != b {
                        at = i;
                        break;
                    }
                }
                let start = at.saturating_sub(60);
                let end = (at + 60).min(n);
                mismatches.push(format!(
                    "{name} len {} vs {} at {at}: {:?} | {:?}",
                    got.len(),
                    committed.len(),
                    String::from_utf8_lossy(&got[start..end]),
                    String::from_utf8_lossy(&committed[start..end])
                ));
            }
        }
        assert_eq!(pictures.len(), 17, "every committed picture has to be in this comparison");
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }

    #[test]
    fn lf_and_crlf_sources_render_the_same_svg() {
        let lf = "flowchart LR\n  A[Start] --> B[End]\n";
        let crlf = lf.replace('\n', "\r\n");
        let mut left = super::from_mermaid(lf, Some("Flow"), None).expect("lf");
        let mut right = super::from_mermaid(&crlf, Some("Flow"), None).expect("crlf");
        let left_svg = left.render().expect("lf svg").svg;
        let right_svg = right.render().expect("crlf svg").svg;
        assert_eq!(left_svg, right_svg);
        assert!(!left_svg.contains('\r'), "svg newlines are LF on every host");
    }

    #[test]
    fn inter_outlines_match_fonttools() {
        let hi = outline("Hi", 16.0, 0.0, 12.0, "regular");
        assert_eq!(
            hi,
            "M1.41 12V0.36H2.89V5.39H9V0.36H10.48V12H9V6.7H2.89V12ZM13.12 12V3.27H14.53V12ZM13.84 1.84Q13.44 1.84 13.15 1.56Q12.86 1.29 12.86 0.91Q12.86 0.52 13.15 0.25Q13.44 -0.02 13.84 -0.02Q14.24 -0.02 14.53 0.25Q14.82 0.52 14.82 0.91Q14.82 1.29 14.53 1.56Q14.24 1.84 13.84 1.84Z"
        );
        let av = outline("AV", 14.0, 3.5, 20.0, "regular");
        assert_eq!(
            av,
            "M3.86 20 7.55 9.81H9.06L12.8 20H11.44L10.42 17.16H6.22L5.24 20ZM6.62 16.02H10.01L9.28 13.96Q9.09 13.44 8.86 12.7Q8.63 11.96 8.3 10.85Q7.98 11.98 7.74 12.73Q7.5 13.49 7.33 13.96ZM16.3 20 12.56 9.81H13.92L16.08 15.86Q16.25 16.33 16.49 17.09Q16.73 17.85 17.06 18.95Q17.38 17.83 17.62 17.08Q17.86 16.33 18.03 15.86L20.12 9.81H21.51L17.8 20Z"
        );
    }
}
