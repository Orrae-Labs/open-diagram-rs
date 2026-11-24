use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde_json::{json, Value};

use super::{Diagram, Enclosed, EnclosureOpts, LinkSpec};
use crate::blocks::{Block, Graph, Panel, Resource, Tile, Tiles};
use crate::error::SaveError;
use crate::theme::{shade, Theme};
use crate::{from_mermaid, from_spec, from_spec_with, load_spec, text_width, wrap, LayoutError};

fn bundled(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn workspace(path: &str) -> Option<PathBuf> {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path);
    file.exists().then_some(file)
}

fn panel(title: &str) -> Block {
    let tiles = Tiles::new(vec![Tile::new("x")], None, None).unwrap_or_else(|err| panic!("{err}"));
    Panel::new(title).section(tiles.into()).into()
}

fn resource(label: &str) -> Resource {
    Resource::new(label, None, None, 64.0, 180.0, None, None).unwrap_or_else(|err| panic!("{err}"))
}

fn opts<'a>(border: &'a str, pad: f64, style: &'a str, tone: &'a str) -> EnclosureOpts<'a> {
    EnclosureOpts { sub: None, pad, border, fill: None, stroke: None, style, tone }
}

fn failure(result: Result<Diagram, impl std::fmt::Display>, label: &str) -> String {
    match result {
        Ok(_) => panic!("{label}: expected an error"),
        Err(err) => err.to_string(),
    }
}

fn assert_clean(problems: &[String]) {
    assert_eq!(problems, &[] as &[String]);
}

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("open-diagram-{name}-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_nanos())));
    std::fs::create_dir_all(&path).unwrap_or_else(|err| panic!("{err}"));
    path
}

#[test]
fn text_width_grows_and_kerning_narrows_av() {
    assert!(text_width("ab", 12.0, "regular") < text_width("abc", 12.0, "regular"));
    assert!(text_width("AV", 100.0, "regular") < text_width("A", 100.0, "regular") + text_width("V", 100.0, "regular"));
}

#[test]
fn wrap_respects_width_and_newlines() {
    for line in wrap("one two three four five six seven", 12.0, Some(70.0), "regular") {
        assert!(text_width(&line, 12.0, "regular") <= 70.0 || !line.contains(' '), "{line}");
    }
    assert_eq!(wrap("first\nsecond", 12.0, None, "regular"), vec!["first".to_owned(), "second".to_owned()]);
}

#[test]
fn overlap_gap_crossing_lane_and_graph_label_are_named() {
    let mut crowded = Diagram::new("t");
    crowded.add(panel("Alpha"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    crowded.add(panel("Beta"), 20.0, 20.0).unwrap_or_else(|err| panic!("{err}"));
    let problems = crowded.render().unwrap_or_else(|err| panic!("{err}")).problems;
    assert!(problems.iter().any(|problem| problem.contains("'Alpha' and 'Beta' overlap")), "{problems:?}");

    let mut gap = Diagram::new("t");
    let alpha = gap.add(panel("Alpha"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    gap.right_of(&alpha, panel("Beta"), Some(5.0), "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    let problems = gap.render().unwrap_or_else(|err| panic!("{err}")).problems;
    assert!(problems.iter().any(|problem| problem.contains("only 5 px apart")), "{problems:?}");

    let mut clean = Diagram::new("t");
    let alpha = clean.add(panel("Alpha"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let beta = clean.right_of(&alpha, panel("Beta"), None, "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    clean.link(&alpha, &beta, LinkSpec::default()).unwrap_or_else(|err| panic!("{err}"));
    assert_clean(&clean.render().unwrap_or_else(|err| panic!("{err}")).problems);

    let mut crossing = Diagram::new("t");
    let alpha = crossing.add(panel("Alpha"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let beta = crossing.right_of(&alpha, panel("Beta"), None, "center", None, None).unwrap_or_else(|err| panic!("{err}"));
    let gamma = crossing.right_of(&beta, panel("Gamma"), None, "center", None, None).unwrap_or_else(|err| panic!("{err}"));
    crossing.link(&alpha, &gamma, LinkSpec::default()).unwrap_or_else(|err| panic!("{err}"));
    let problems = crossing.render().unwrap_or_else(|err| panic!("{err}")).problems;
    assert!(problems.iter().any(|problem| problem.contains("crosses 'Beta'")), "{problems:?}");
    crossing.links[0].cross_ok = true;
    assert_clean(&crossing.render().unwrap_or_else(|err| panic!("{err}")).problems);

    let mut lane = Diagram::new("t");
    let alpha = lane.add(panel("Alpha"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let beta = lane.right_of(&alpha, panel("Beta"), None, "center", None, None).unwrap_or_else(|err| panic!("{err}"));
    let gamma = lane.right_of(&beta, panel("Gamma"), None, "center", None, None).unwrap_or_else(|err| panic!("{err}"));
    lane.link(&alpha, &gamma, LinkSpec::default().start("top").end("top").via_y(alpha.y - 40.0)).unwrap_or_else(|err| panic!("{err}"));
    assert_clean(&lane.render().unwrap_or_else(|err| panic!("{err}")).problems);

    let mut nodes = IndexMap::new();
    nodes.insert("a".to_owned(), "A".to_owned());
    nodes.insert("b".to_owned(), "B".to_owned());
    let mut at = IndexMap::new();
    at.insert("a".to_owned(), (0.0, 0.0));
    at.insert("b".to_owned(), (0.6, 0.0));
    let graph = Graph::new(nodes, vec![("a".to_owned(), "b".to_owned(), Some("a very long label for a short line".to_owned()))], at).unwrap_or_else(|err| panic!("{err}"));
    let mut map = Diagram::new("t");
    map.add(Panel::new("Map").section(graph.into()), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let problems = map.render().unwrap_or_else(|err| panic!("{err}")).problems;
    assert!(problems.iter().any(|problem| problem.contains("touches the node")), "{problems:?}");
}

#[test]
fn unknown_status_names_live() {
    let mut diagram = Diagram::new("t");
    let tile = Tile::new("x").status("nope");
    let body = Tiles::new(vec![tile], None, None).unwrap_or_else(|err| panic!("{err}"));
    diagram.add(Panel::new("Alpha").section(body.into()), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let err = diagram.render().expect_err("unknown status");
    assert!(err.to_string().contains("live"), "{err}");
}

#[test]
fn save_writes_the_svg_before_reporting_overlap() {
    let mut diagram = Diagram::new("t");
    diagram.add(panel("Alpha"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    diagram.add(panel("Beta"), 10.0, 10.0).unwrap_or_else(|err| panic!("{err}"));
    let dir = scratch("save");
    let path = dir.join("out");
    let err = diagram.save(path.to_str().unwrap_or("out"), false, 2.0, true).expect_err("layout");
    let SaveError::Layout(err) = err else {
        panic!("expected LayoutError");
    };
    assert!(path.with_extension("svg").is_file());
    assert!(err.to_string().contains("overlap"), "{err}");
    let _: LayoutError = err;
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn theme_colors_header_contrast_and_flat_measurement() {
    let mut theme = Theme::default();
    theme.tones.insert("default".to_owned(), "#123456".to_owned());
    theme.badge_fill = Some("#ABCDEF".to_owned());
    let mut diagram = Diagram::new("t").theme(theme);
    diagram.add(Panel::new("Alpha").section(Tiles::new(vec![Tile::new("x")], None, None).unwrap_or_else(|err| panic!("{err}")).into()).step(1), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert!(report.svg.contains("#123456"));
    assert!(report.svg.contains("#ABCDEF"));

    let theme = Theme::default();
    assert_eq!(theme.header_colors("#2B3036").unwrap_or_else(|err| panic!("{err}")), (theme.header_text_on_dark.clone(), theme.header_sub_on_dark.clone()));
    assert_eq!(theme.header_colors("#E2E5E9").unwrap_or_else(|err| panic!("{err}")), (theme.header_text.clone(), theme.header_sub));

    let on = Theme::default();
    let off = Theme { lid: 0.0, depth: 0.0, ..Theme::default() };
    let tile = Tile::new("x");
    assert_eq!(tile.measure(&on).0 - tile.measure(&off).0, on.depth);
    let panel = Panel::new("A").section(Tiles::new(vec![Tile::new("x")], None, None).unwrap_or_else(|err| panic!("{err}")).into());
    let on_h = panel.measure(&on).unwrap_or_else(|err| panic!("{err}")).1;
    let off_h = panel.measure(&off).unwrap_or_else(|err| panic!("{err}")).1;
    assert_eq!(on_h - off_h, on.lid + on.depth);
}

#[test]
fn describe_and_statuses_follow_the_quickstart() {
    let mut diagram = crate::examples::quickstart().unwrap_or_else(|err| panic!("{err}"));
    let text = diagram.describe();
    for part in [
        "How a search request is answered",
        "Web app: Search box; Results page; Filters",
        "Caches results (being built)",
        "Search index [planned]",
        "Search API -> Search index",
        "(connection to build)",
        "labeled \"new\"",
    ] {
        assert!(text.contains(part), "{part} missing from {text}");
    }
    let svg = diagram.render().unwrap_or_else(|err| panic!("{err}")).svg;
    assert!(svg.contains("<desc>How a search request is answered"));

    let mut statuses = IndexMap::new();
    statuses.insert("Caches results".to_owned(), "live".to_owned());
    statuses.insert("Product text".to_owned(), "retiring".to_owned());
    diagram.set_statuses(&statuses).unwrap_or_else(|err| panic!("{err}"));
    let text = diagram.describe();
    assert!(text.contains("Checks access; Caches results. Owner"), "{text}");
    assert!(text.contains("Product text (retiring)"), "{text}");

    let mut missing = IndexMap::new();
    missing.insert("Nope".to_owned(), "live".to_owned());
    let err = diagram.set_statuses(&missing).expect_err("missing name");
    assert!(err.to_string().contains("'Nope'"), "{err}");
    let mut shipped = IndexMap::new();
    shipped.insert("Filters".to_owned(), "shipped".to_owned());
    let err = diagram.set_statuses(&shipped).expect_err("missing state");
    assert!(err.to_string().contains("'shipped'"), "{err}");
}

#[test]
fn spec_and_mermaid_errors_name_the_place() {
    let spec = json!({
        "title": "t",
        "columns": [[{"id": "a", "title": "Alpha", "tiles": ["x", {"label": "y", "status": "planned"}]}], [{"id": "b", "title": "Beta", "tiles": ["z"]}]],
        "links": [{"from": "a", "to": "b", "kind": "planned"}, {"from": "b", "to": "a", "lane": "bottom"}]
    });
    let mut diagram = from_spec(&spec).unwrap_or_else(|err| panic!("{err}"));
    assert_clean(&diagram.render().unwrap_or_else(|err| panic!("{err}")).problems);
    assert_eq!(diagram.lines().len(), 2);
    assert!(diagram.describe().contains("y (planned)"));

    let bad = json!({"title": "t", "columns": [[{"id": "a", "title": "Alpha", "tiles": [{"label": "x", "colour": "red"}]}]]});
    let err = failure(from_spec(&bad), "colour");
    assert!(err.contains("columns[0][0].tiles[0]"), "{err}");
    assert!(err.contains("'colour'"), "{err}");
    let bad = json!({"title": "t", "columns": [[{"id": "a", "title": "Alpha", "tiles": ["x"]}]], "links": [{"from": "a", "to": "c"}]});
    let err = failure(from_spec(&bad), "missing id");
    assert!(err.contains("no panel with id 'c'"), "{err}");

    let email = load_spec(bundled("examples/email_delivery.json")).unwrap_or_else(|err| panic!("{err}"));
    assert_clean(&email.render_owned().problems);

    let text = "flowchart LR\n  a[Start] --> b{Decide}\n  b -->|yes| c[Do it]:::building\n  b -- no --> a\n  c -.-> d([Done])\n  a --> d\n  subgraph work [The work]\n    c\n  end\n";
    let flow = from_mermaid(text, Some("Flow"), None).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(flow.title, "Flow");
    assert_eq!(flow.lines().len(), 5);
    assert!(flow.lines().into_iter().any(|line| line.a == "Do it" && line.b == "Done" && line.kind == "planned"));
    let labels: Vec<_> = flow.lines().into_iter().filter_map(|line| line.label).collect();
    assert!(labels.contains(&"yes".to_owned()) && labels.contains(&"no".to_owned()));
    assert_eq!(flow.groups.iter().map(|group| group.label.as_str()).collect::<Vec<_>>(), vec!["The work"]);
    assert!(flow.describe().contains("Do it (being built)"));
    for direction in ["LR", "TD"] {
        let mut turned = from_mermaid(&text.replace("LR", direction), None, None).unwrap_or_else(|err| panic!("{err}"));
        assert!(turned.render().unwrap_or_else(|err| panic!("{err}")).problems.is_empty(), "{direction}");
    }
    let signup = load_spec_mermaid();
    assert_clean(&signup.problems);
    let words = from_mermaid("flowchart LR\n  click[Click it] --> style[Style it]:::building\n  style -.-> class[Done]\n", None, None).unwrap_or_else(|err| panic!("{err}"));
    let pairs: Vec<_> = words.lines().into_iter().map(|line| (line.a, line.b, line.kind)).collect();
    assert_eq!(pairs, vec![("Click it".to_owned(), "Style it".to_owned(), "flow".to_owned()), ("Style it".to_owned(), "Done".to_owned(), "planned".to_owned())]);
    assert!(words.describe().contains("Style it (being built)"));
    let mail = "flowchart TD\n a[Ask] --> b{Found?}\n b -->|yes| c[Send link]\n b -->|no| d[Same message]\n c --> e[Click]\n e -.->|expired| a\n subgraph mail [Email service]\n  c\n end\n";
    let mut mail = from_mermaid(mail, None, None).unwrap_or_else(|err| panic!("{err}"));
    assert_clean(&mail.render().unwrap_or_else(|err| panic!("{err}")).problems);
    let err = failure(from_mermaid("flowchart LR\n  a[ok] --> ???", None, None), "line");
    assert!(err.contains("line 2"), "{err}");
}

fn load_spec_mermaid() -> crate::Report {
    let mut diagram = crate::load_mermaid(bundled("examples/signup_flow.mmd"), None).unwrap_or_else(|err| panic!("{err}"));
    diagram.render().unwrap_or_else(|err| panic!("{err}"))
}

trait RenderOwned {
    fn render_owned(self) -> crate::Report;
}

impl RenderOwned for Diagram {
    fn render_owned(mut self) -> crate::Report {
        self.render().unwrap_or_else(|err| panic!("{err}"))
    }
}

#[test]
fn group_titles_rings_and_facing_sides() {
    let mut diagram = Diagram::new("t");
    let a = diagram.add(panel("A"), 0.0, 60.0).unwrap_or_else(|err| panic!("{err}"));
    let b = diagram.add(panel("B"), 420.0, 100.0).unwrap_or_else(|err| panic!("{err}"));
    diagram.group("Group title", std::slice::from_ref(&b), None, None);
    diagram.link(&a, &b, LinkSpec::default().start("right").end("left").via_y(78.0).label("a long label that reaches the group title")).unwrap_or_else(|err| panic!("{err}"));
    let problems = diagram.render().unwrap_or_else(|err| panic!("{err}")).problems;
    assert!(problems.iter().any(|problem| problem.contains("sits on the title of group 'Group title'")), "{problems:?}");

    let mut ring = Diagram::new("t");
    let blocks: Vec<Block> = (0..5).map(|i| {
        let tiles = Tiles::new(vec![Tile::new("x")], None, None).unwrap_or_else(|err| panic!("{err}"));
        Panel::new(format!("Step {i}")).section(tiles.into()).into()
    }).collect();
    let center = vec![panel("Goal")];
    let (around, middle) = ring.ring(blocks, 0.0, 0.0, None, -90.0, center).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!((around.len(), middle.len()), (5, 1));
    for pair in around.iter().zip(around.iter().cycle().skip(1)).take(around.len()) {
        ring.link(pair.0, pair.1, LinkSpec::default().start("auto").end("auto")).unwrap_or_else(|err| panic!("{err}"));
    }
    assert_clean(&ring.render().unwrap_or_else(|err| panic!("{err}")).problems);

    let mut stacked = Diagram::new("t");
    let top = stacked.add(panel("Top"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let below = stacked.below(&top, panel("Below"), None, "left", None, None).unwrap_or_else(|err| panic!("{err}"));
    stacked.link(&top, &below, LinkSpec::default().start("auto").end("auto")).unwrap_or_else(|err| panic!("{err}"));
    stacked.render().unwrap_or_else(|err| panic!("{err}"));
    let line = &stacked.lines()[0];
    assert_eq!((line.start.as_str(), line.end.as_str()), ("bottom", "top"));

    let mut away = Diagram::new("t");
    let left = away.add(panel("Left"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let right = away.right_of(&left, panel("Right"), None, "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    away.link(&left, &right, LinkSpec::default().start("left").end("left")).unwrap_or_else(|err| panic!("{err}"));
    let problems = away.render().unwrap_or_else(|err| panic!("{err}")).problems;
    assert!(problems.iter().any(|problem| problem.contains("leaves 'Left' from its left side, facing away from 'Right'")), "{problems:?}");
}

#[test]
fn json_layouts_and_public_helpers() {
    let free = json!({"title": "t", "panels": [{"id": "a", "title": "A", "tiles": ["x"], "x": 0, "y": 0}, {"id": "b", "title": "B", "tiles": ["y"], "x": 320, "y": 160}], "links": [{"from": "a", "to": "b"}]});
    assert_clean(&from_spec(&free).unwrap_or_else(|err| panic!("{err}")).render_owned().problems);
    assert_clean(&load_spec(bundled("examples/release_cycle.json")).unwrap_or_else(|err| panic!("{err}")).render_owned().problems);
    let cases = [
        (json!({"title": "t", "panels": [{"title": "A", "tiles": ["x"], "x": 0}]}), "needs numbers for x and y"),
        (json!({"title": "t", "columns": [[{"title": "A", "x": 1}]]}), "only work in the 'panels' layout"),
        (json!({"title": "t", "columns": [[{"title": "A"}]], "ring": [{"title": "B"}]}), "exactly one layout"),
        (json!({"title": "t"}), "exactly one layout"),
    ];
    for (spec, words) in cases {
        let err = failure(from_spec(&spec), words);
        assert!(err.contains(words), "{words} missing from {err}");
    }
    assert_eq!(crate::panel_from_spec(&json!({"title": "A", "tiles": ["x"]}), "panel").unwrap_or_else(|err| panic!("{err}")).title, "A");
}

#[test]
fn yellow_step_badges_turn_dark_without_changing_the_theme() {
    let mut diagram = Diagram::new("t");
    let body = Tiles::new(vec![Tile::new("x")], None, None).unwrap_or_else(|err| panic!("{err}"));
    diagram.add(Panel::new("Plain").section(body.into()).step(1), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(diagram.drawing_theme().unwrap_or_else(|err| panic!("{err}")).badge_fill, Theme::default().badge_fill);
    let body = Tiles::new(vec![Tile::new("y")], None, None).unwrap_or_else(|err| panic!("{err}"));
    diagram.add(Panel::new("Decision").section(body.into()).tone("accent"), 300.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(diagram.drawing_theme().unwrap_or_else(|err| panic!("{err}")).badge_fill.as_deref(), Theme::default().tones.get("dark").map(String::as_str));
    assert_eq!(diagram.theme.badge_fill, Theme::default().badge_fill);
}

#[test]
fn resources_embed_icons_and_reject_bad_inputs() {
    let dir = scratch("icon");
    let icon = dir.join("icon.svg");
    std::fs::write(&icon, "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 64 64\"><circle cx=\"32\" cy=\"32\" r=\"28\"/></svg>").unwrap_or_else(|err| panic!("{err}"));
    let mut diagram = Diagram::new("Resources");
    let left = diagram.add(Resource::new("Long extractor label", Some(icon.to_string_lossy().into_owned()), Some("AWS Lambda".to_owned()), 64.0, 180.0, None, None).unwrap_or_else(|err| panic!("{err}")), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let right = diagram.right_of(&left, Resource::new("Bucket", Some(icon.to_string_lossy().into_owned()), None, 64.0, 180.0, None, None).unwrap_or_else(|err| panic!("{err}")), None, "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    diagram.link(&left, &right, LinkSpec::default()).unwrap_or_else(|err| panic!("{err}"));
    let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(report.svg.matches("<image").count(), 2);
    assert!(report.svg.contains("data:image/svg+xml;base64,"));
    assert!(report.svg.contains("width=\"64\""));
    let span = left.w + 64.0;
    let mid = span / 2.0;
    assert_eq!(crate::fmt(diagram.links[0].start_pt.0), crate::fmt(left.x + mid));
    let text = diagram.describe();
    assert!(!text.contains("States shown: Live"), "{text}");
    assert!(text.contains("Long extractor label (AWS Lambda)"), "{text}");
    let missing = Resource::new("Extractor", Some("missing-cloud-icon.svg".to_owned()), None, 64.0, 180.0, None, None).expect_err("missing");
    assert!(missing.to_string().contains("Extractor") && missing.to_string().contains("icon"), "{missing}");
    for (size, width, border) in [(0.0, 180.0, None), (64.0, -1.0, None), (f64::NAN, 180.0, None)] {
        assert!(Resource::new("Extractor", None, None, size, width, border, None).is_err());
    }
    assert!(Resource::new("Extractor", None, None, 64.0, 180.0, Some("dahsed".to_owned()), None).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn resource_borders_do_not_invent_state_and_lanes_miss_labels() {
    for style in [None, Some("solid"), Some("dashed"), Some("dotted")] {
        let mut diagram = Diagram::new("Framing");
        diagram.add(Resource::new("Compute", None, None, 64.0, 180.0, style.map(str::to_owned), None).unwrap_or_else(|err| panic!("{err}")), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
        let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
        assert!(report.problems.is_empty(), "{:?}", report.problems);
        assert!(!diagram.describe().contains("States shown:"));
        let dashed = matches!(style, Some("dashed" | "dotted"));
        assert_eq!(report.svg.contains("stroke-dasharray"), dashed, "{style:?}");
    }
    let mut etl = crate::examples::aws_etl().unwrap_or_else(|err| panic!("{err}"));
    etl.render().unwrap_or_else(|err| panic!("{err}"));
    let line = etl.links.iter().find(|line| line.label.as_deref() == Some("after extraction")).expect("lane");
    let node = &etl.nodes[line.a];
    let image_bottom = node.placed.y + etl.theme.pad + 64.0;
    assert!(line.samples.iter().all(|point| !(node.placed.x < point.0 && point.0 < node.placed.right() && image_bottom < point.1 && point.1 < node.placed.bottom())));
}

#[test]
fn enclosures_measure_nest_and_keep_unrelated_overlap() {
    for lid in [0.0, 14.0] {
        let theme = Theme { lid, ..Theme::default() };
        let mut diagram = Diagram::new("Containers").theme(theme);
        let bucket = diagram.add(resource("Bucket"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
        let mut inner_opts = opts("solid", 24.0, "panel", "accent");
        inner_opts.sub = Some("A measured subtitle".to_owned());
        let inner = diagram.enclosure("An intentionally long resource section heading", &[Enclosed::Placed(bucket.clone())], inner_opts).unwrap_or_else(|err| panic!("{err}"));
        let mut outer_opts = opts("solid", 24.0, "panel", "default");
        outer_opts.fill = Some("#FFF9E8".to_owned());
        let outer = diagram.enclosure("Cloud account", &[Enclosed::Group(inner)], outer_opts).unwrap_or_else(|err| panic!("{err}"));
        let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
        assert!(report.problems.is_empty(), "{lid} {:?}", report.problems);
        let (_x, y, _w, h) = diagram.group_title_box(inner.0);
        assert_eq!(h, lid + diagram.theme.header_h_sub);
        assert!(y + h + diagram.groups[inner.0].pad <= bucket.y);
        let (ox, _oy, ow, _oh) = diagram.groups[outer.0].rect;
        let (ix, _iy, iw, _ih) = diagram.group_title_box(inner.0);
        assert!(ix + iw <= ox + ow);
        assert!(diagram.groups[outer.0].rect.1 < diagram.groups[inner.0].rect.1);
        assert!(diagram.describe().contains("A measured subtitle"));
        let accent = diagram.theme.tone("accent").unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(report.svg.contains(&shade(&accent, 0.16)), lid != 0.0);
        assert!(!diagram.describe().contains("States shown:"));
    }

    for (border, dash) in [("solid", None), ("dashed", Some("5 3.5")), ("dotted", Some("0.1 3.4"))] {
        let mut diagram = Diagram::new("Frame");
        let node = diagram.add(resource("Compute"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
        diagram.enclosure("Dark section", &[Enclosed::Placed(node)], opts(border, 24.0, "panel", "dark")).unwrap_or_else(|err| panic!("{err}"));
        let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
        assert!(report.problems.is_empty(), "{border} {:?}", report.problems);
        let at = report.svg.find("<title>Dark section</title>").unwrap_or_else(|| panic!("{}", report.svg));
        let start = report.svg[..at].rfind("<path").unwrap_or(0);
        assert!(report.svg[start..at].contains(&format!("fill=\"{}\"", diagram.theme.header_text_on_dark)), "{border}");
        match dash {
            Some(dash) => assert!(report.svg.contains(&format!("stroke-dasharray=\"{dash}\"")), "{border}"),
            None => assert!(!report.svg.contains("stroke-dasharray=\"5 3.5\"") && !report.svg.contains("stroke-dasharray=\"0.1 3.4\"")),
        }
    }
}

#[test]
fn nested_titles_fit_and_bad_members_are_rejected() {
    let mut diagram = Diagram::new("Nested");
    let left = diagram.add(resource("Extractor"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let right = diagram.right_of(&left, resource("Database"), Some(160.0), "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    let mut inner_opts = opts("dotted", 24.0, "outline", "default");
    inner_opts.fill = Some("#F1F2F3".to_owned());
    let inner = diagram.enclosure("Private subnet", &[Enclosed::Placed(left)], inner_opts).unwrap_or_else(|err| panic!("{err}"));
    let mut outer_opts = opts("dashed", 24.0, "outline", "default");
    outer_opts.fill = Some("#FAFBFC".to_owned());
    let outer = diagram.enclosure("AWS account", &[Enclosed::Group(inner), Enclosed::Placed(right)], outer_opts).unwrap_or_else(|err| panic!("{err}"));
    let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert!(report.svg.find("fill=\"#FAFBFC\"").unwrap_or(usize::MAX) < report.svg.find("fill=\"#F1F2F3\"").unwrap_or(0));
    assert!(diagram.groups[outer.0].rect.0 < diagram.groups[inner.0].rect.0);
    assert!(diagram.groups[outer.0].rect.1 < diagram.groups[inner.0].rect.1);
    assert!(report.svg.contains("stroke-dasharray=\"0.1 3.4\""));
    assert!(report.svg.contains("stroke-dasharray=\"5 3.5\""));
    assert!(diagram.describe().contains("Private subnet"));

    let mut solid = Diagram::new("Enclosure");
    let node = solid.add(resource("S3"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let group = solid.enclosure("A deliberately long descriptive cloud account boundary", &[Enclosed::Placed(node)], opts("solid", 24.0, "outline", "default")).unwrap_or_else(|err| panic!("{err}"));
    let report = solid.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    let (bx, _, bw, _) = solid.group_title_box(group.0);
    let (gx, _, gw, _) = solid.groups[group.0].rect;
    assert!(bx + bw <= gx + gw);
    assert!(!report.svg.contains("stroke-dasharray"));

    let mut validation = Diagram::new("Validation");
    let err = validation.enclosure("Account", &[], opts("dashed", 24.0, "outline", "default")).expect_err("empty");
    assert_ne!(err.to_string(), "");
    let mut other = Diagram::new("Other");
    let foreign = other.add(resource("Foreign"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    assert!(validation.enclosure("Account", &[Enclosed::Placed(foreign)], opts("dashed", 24.0, "outline", "default")).is_err());
    let node = validation.add(resource("Node"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    assert!(validation.enclosure("Account", &[Enclosed::Placed(node.clone())], opts("dahsed", 24.0, "outline", "default")).is_err());
    assert!(validation.enclosure("Account", &[Enclosed::Placed(node.clone())], opts("dashed", -1.0, "outline", "default")).is_err());
    assert!(validation.enclosure("Account", &[Enclosed::Placed(node.clone())], opts("dashed", f64::INFINITY, "outline", "default")).is_err());
    assert!(validation.enclosure("Account", &[Enclosed::Placed(node.clone())], opts("dashed", 24.0, "pannel", "default")).is_err());
    assert!(validation.enclosure("Account", &[Enclosed::Placed(node)], opts("dashed", 24.0, "outline", "missing-tone")).is_err());

    let mut overlap = Diagram::new("Overlap");
    let a = overlap.add(resource("A"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    overlap.add(resource("Unrelated"), a.right() + 18.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    overlap.enclosure("Boundary", &[Enclosed::Placed(a)], opts("dashed", 50.0, "outline", "default")).unwrap_or_else(|err| panic!("{err}"));
    let problems = overlap.render().unwrap_or_else(|err| panic!("{err}")).problems;
    assert!(problems.iter().any(|problem| problem.contains("overlaps 'Unrelated'")), "{problems:?}");
}

#[test]
fn link_dashes_are_independent_of_planned_state() {
    let mut theme = Theme::default();
    theme.link_labels.insert("control".to_owned(), "Orchestration".to_owned());
    let mut diagram = Diagram::new("Control").theme(theme);
    let a = diagram.add(resource("MWAA"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let b = diagram.right_of(&a, resource("Lambda"), None, "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    diagram.link(&a, &b, LinkSpec::default().kind("control").dash("dashed")).unwrap_or_else(|err| panic!("{err}"));
    let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert!(report.svg.contains("stroke-dasharray=\"5 3.5\""));
    let text = diagram.describe();
    assert!(text.to_ascii_lowercase().contains("orchestration"), "{text}");
    assert!(!text.to_ascii_lowercase().contains("planned"), "{text}");
    assert!(text.contains("Dashed lines: orchestration"), "{text}");
    let err = diagram.link(&a, &b, LinkSpec::default().kind("control").dash("dahsed")).expect_err("dash");
    assert!(err.to_string().contains("dash"), "{err}");

    let spec = json!({"title": "Dash", "columns": [[{"id": "a", "title": "A"}], [{"id": "b", "title": "B"}]], "links": [{"from": "a", "to": "b", "dash": "dashed"}]});
    let report = from_spec(&spec).unwrap_or_else(|err| panic!("{err}")).render_owned();
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert!(report.svg.contains("stroke-dasharray=\"5 3.5\""));
    let mut bad = spec;
    bad["links"][0]["dash"] = json!("typo");
    let err = failure(from_spec(&bad), "typo");
    assert!(err.contains("links[0]") && err.contains("dash"), "{err}");

    let mut theme = Theme::default();
    theme.link_labels.insert("flow".to_owned(), "Data".to_owned());
    let mut mixed = Diagram::new("Mixed").theme(theme);
    let a = mixed.add(resource("A"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let b = mixed.right_of(&a, resource("B"), None, "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    let c = mixed.right_of(&b, resource("C"), None, "top", None, None).unwrap_or_else(|err| panic!("{err}"));
    mixed.link(&a, &b, LinkSpec::default()).unwrap_or_else(|err| panic!("{err}"));
    mixed.link(&b, &c, LinkSpec::default().dash("dashed")).unwrap_or_else(|err| panic!("{err}"));
    let report = mixed.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert!(report.svg.contains("<title>Data (solid)</title>"));
    assert!(report.svg.contains("<title>Data (dashed)</title>"));

    let mut clear = Diagram::new("Transparent");
    let bucket = clear.add(resource("Bucket"), 0.0, 0.0).unwrap_or_else(|err| panic!("{err}"));
    let mut inner = opts("dashed", 24.0, "outline", "default");
    inner.fill = Some("#EFEFEF".to_owned());
    let inner = clear.enclosure("Storage", &[Enclosed::Placed(bucket)], inner).unwrap_or_else(|err| panic!("{err}"));
    let mut outer = opts("dashed", 24.0, "outline", "default");
    outer.fill = Some("none".to_owned());
    clear.enclosure("Account", &[Enclosed::Group(inner)], outer).unwrap_or_else(|err| panic!("{err}"));
    let report = clear.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert!(report.svg.contains("fill=\"none\" stroke="));
    assert!(report.svg.contains("fill=\"#EFEFEF\""));
}

#[test]
fn templates_match_the_standard_and_json_resources() {
    let mut etl = crate::examples::aws_etl().unwrap_or_else(|err| panic!("{err}"));
    etl.render().unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(etl.groups.len(), 7);
    assert!(etl.groups.iter().all(|group| group.style == "panel"));
    assert_ne!(etl.theme.paper, "#FFFFFF");
    for line in &etl.links {
        for index in 0..etl.groups.len() {
            let (x, y, w, h) = etl.group_title_box(index);
            let hit = line.samples.iter().any(|point| x < point.0 && point.0 < x + w && y < point.1 && point.1 < y + h);
            assert!(!hit, "{} -> {} crosses {}", etl.nodes[line.a].placed.name, etl.nodes[line.b].placed.name, etl.groups[index].label);
        }
    }
    let names: std::collections::BTreeSet<_> = etl.placed().map(|placed| placed.name.clone()).collect();
    assert_eq!(names, ["api", "athena", "catalog", "curated", "dashboards", "database", "glue", "mwaa", "raw", "saas"].into_iter().map(str::to_owned).collect());
    let edges: std::collections::BTreeSet<_> = etl.links.iter().map(|line| (etl.nodes[line.a].placed.name.clone(), etl.nodes[line.b].placed.name.clone())).collect();
    for key in ["api", "saas", "database"] {
        assert!(edges.contains(&(String::from("mwaa"), key.to_owned())));
        assert!(edges.contains(&(key.to_owned(), String::from("raw"))));
    }
    for border in ["solid", "dashed", "dotted"] {
        for group in &mut etl.groups {
            group.border = border.to_owned();
        }
        assert!(etl.render().unwrap_or_else(|err| panic!("{err}")).problems.is_empty(), "{border}");
    }

    let templates = crate::catalog::template_dir();
    let readme = templates.join("README.md");
    let rows = crate::catalog::table(Some(&readme)).unwrap_or_else(|err| panic!("{err}"));
    let files = std::fs::read_dir(&templates).unwrap_or_else(|err| panic!("{err}")).filter_map(|entry| {
        let path = entry.ok()?.path();
        let name = path.file_name()?.to_string_lossy().into_owned();
        matches!(path.extension().and_then(|ext| ext.to_str()), Some("json" | "mmd")).then_some(name)
    }).collect::<std::collections::BTreeSet<_>>();
    let listed: std::collections::BTreeSet<_> = rows.iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(listed, files);
    assert!(std::fs::read_dir(&templates).unwrap_or_else(|err| panic!("{err}")).flatten().all(|entry| entry.path().extension().and_then(|ext| ext.to_str()) != Some("py")));
    for (name, purpose) in &rows {
        assert!(name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '.'), "{name}");
        let ext = Path::new(name).extension().and_then(|ext| ext.to_str()).unwrap_or("");
        assert!(ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("mmd"), "{name}");
        assert!(!purpose.is_empty(), "{name}");
    }
    let credits = std::fs::read_to_string(templates.join("assets/README.md")).unwrap_or_else(|err| panic!("{err}"));
    for entry in std::fs::read_dir(templates.join("assets")).unwrap_or_else(|err| panic!("{err}")).flatten() {
        if entry.path().is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            assert!(credits.replace('\r', "").contains(&format!("## {name}\n")), "{name}");
        }
    }
    assert_eq!(templates.canonicalize().unwrap_or_else(|err| panic!("{err}")), bundled("templates").canonicalize().unwrap_or_else(|err| panic!("{err}")));
    for name in ["email_delivery.json", "release_cycle.json", "signup_flow.mmd"] {
        if let Some(published) = workspace(&format!("examples/{name}")) {
            let left = std::fs::read(bundled(&format!("examples/{name}"))).unwrap_or_else(|err| panic!("{err}"));
            let right = std::fs::read(published).unwrap_or_else(|err| panic!("{err}"));
            assert_eq!(left, right, "{name}");
        }
    }
    let names = crate::catalog::names().unwrap_or_else(|err| panic!("{err}"));
    assert!(names.iter().any(|name| name == "azure_web_app"));
    assert!(crate::catalog::read("aws_etl").unwrap_or_else(|err| panic!("{err}")).trim_start().starts_with('{'));
    assert!(crate::catalog::embedded("README.md").is_some());
    assert!(crate::catalog::embedded("assets/aws/s3.svg").is_some());
    assert!(crate::catalog::embedded("../Cargo.toml").is_none());
    let err = crate::catalog::path("nope").expect_err("missing template");
    assert!(err.contains("no template named 'nope'; templates: aws_etl"), "{err}");

    for entry in std::fs::read_dir(&templates).unwrap_or_else(|err| panic!("{err}")).flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let mut diagram = load_spec(&path).unwrap_or_else(|err| panic!("{err}"));
        let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
        assert!(report.problems.is_empty(), "{} {:?}", path.display(), report.problems);
        assert!(report.min_text.unwrap_or(0.0) >= 10.0);
        assert!(report.width <= 2000.0, "{}", report.width);
        assert!(report.height <= 1400.0, "{}", report.height);
        assert!(report.svg.contains("data:"));
        for node in &diagram.nodes {
            if let Block::Resource(resource) = &node.block
                && resource.icon.is_some() {
                    assert!(resource.sub.is_some(), "{} needs its service name", resource.label);
                }
        }
        if let Some(images) = workspace("docs/images") {
            let preview = images.join(format!("{}.png", path.file_stem().unwrap_or_default().to_string_lossy()));
            assert!(preview.is_file(), "{}", preview.display());
        }
    }
}

#[test]
fn json_resources_name_mistakes_and_refuse_paths_outside_icon_folders() {
    let spec = json!({
        "title": "Resources from JSON",
        "resources": [
            {"id": "fn", "label": "Worker", "sub": "AWS Lambda", "icon": "aws/lambda", "x": 0, "y": 0},
            {"id": "db", "label": "Records", "sub": "Amazon RDS", "icon": "aws/rds", "x": 300, "y": 0}
        ],
        "enclosures": [
            {"id": "data", "label": "Data subnet", "members": ["db"], "border": "dotted"},
            {"id": "net", "label": "Network", "members": ["fn", "data"], "border": "solid", "pad": 32}
        ],
        "links": [{"from": "fn", "to": "db", "label": "SQL", "dash": "dashed"}]
    });
    let mut diagram = from_spec(&spec).unwrap_or_else(|err| panic!("{err}"));
    let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(diagram.groups.iter().map(|group| group.label.as_str()).collect::<Vec<_>>(), vec!["Data subnet", "Network"]);
    assert_eq!(report.svg.matches("<image").count(), 2);
    assert!(diagram.describe().contains("Worker (AWS Lambda)"));

    let cases: Vec<(Value, &str)> = vec![
        (json!({"title": "t", "resources": spec["resources"].clone(), "enclosures": [{"label": "Net", "members": ["fn", "data"]}, {"id": "data", "label": "D", "members": ["db"]}], "links": []}), "enclosures[0].members: enclosure 'data' is listed after this one"),
        (json!({"title": "t", "resources": spec["resources"].clone(), "enclosures": [{"label": "Net", "members": ["ghost"]}], "links": []}), "no resource"),
        (json!({"title": "t", "resources": [{"id": "a", "label": "A", "x": 0, "y": 0}, {"id": "a", "label": "B", "x": 200, "y": 0}]}), "resources[1].id: 'a' is already used"),
        (json!({"title": "t", "resources": [{"label": "A", "x": 0}]}), "resources[0]: missing required key 'y'"),
        (json!({"title": "t", "resources": spec["resources"].clone(), "columns": [[{"title": "P"}]]}), "not with 'columns'"),
        (json!({"title": "t", "resources": spec["resources"].clone(), "enclosures": [{"label": "Net", "members": ["fn"], "border": "wavy"}]}), "border"),
        (json!({"title": "t", "resources": [{"label": "A", "icon": "aws/nope", "x": 0, "y": 0}]}), "no icon named 'aws/nope'"),
    ];
    for (spec, words) in cases {
        let err = failure(from_spec(&spec), words);
        assert!(err.contains(words), "{words} missing from {err}");
        if words.contains("aws/nope") {
            assert!(err.contains("aws/lambda"), "{err}");
        }
        if words == "no resource" {
            assert!(err.contains("'ghost'"), "{err}");
        }
        if words == "border" {
            assert!(err.contains("enclosures[0]"), "{err}");
        }
    }

    let dir = scratch("outside");
    let secret = dir.join("secret.svg");
    std::fs::write(&secret, "<svg xmlns=\"http://www.w3.org/2000/svg\"/>").unwrap_or_else(|err| panic!("{err}"));
    let folder = dir.join("icons");
    std::fs::create_dir_all(folder.join("own")).unwrap_or_else(|err| panic!("{err}"));
    std::fs::write(folder.join("own").join("box.svg"), "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"/>").unwrap_or_else(|err| panic!("{err}"));
    for name in [secret.to_string_lossy().into_owned(), "../secret".to_owned(), "own/../../secret".to_owned(), "C:/secret".to_owned(), "own//box".to_owned()] {
        let spec = json!({"title": "t", "resources": [{"label": "A", "icon": name, "x": 0, "y": 0}]});
        assert!(from_spec_with(&spec, Some(std::slice::from_ref(&folder))).is_err(), "{name}");
    }
    let spec = json!({"title": "t", "resources": [{"label": "A", "sub": "Mine", "icon": "own/box", "x": 0, "y": 0}]});
    assert_clean(&from_spec_with(&spec, Some(std::slice::from_ref(&folder))).unwrap_or_else(|err| panic!("{err}")).render_owned().problems);

    let beside = scratch("beside");
    std::fs::write(beside.join("mine.svg"), "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"/>").unwrap_or_else(|err| panic!("{err}"));
    let file = beside.join("d.json");
    let body = serde_json::to_string(&json!({
        "title": "Tiles",
        "columns": [[{
            "title": "Store",
            "tiles": [
                {"label": "Bucket", "icon": "aws/s3"},
                {"label": "Mine", "icon": "mine"}
            ]
        }]]
    }))
    .unwrap_or_else(|err| panic!("{err}"));
    std::fs::write(&file, body).unwrap_or_else(|err| panic!("{err}"));
    let mut diagram = load_spec(&file).unwrap_or_else(|err| panic!("{err}"));
    let report = diagram.render().unwrap_or_else(|err| panic!("{err}"));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(report.svg.matches("<image").count(), 2);
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(beside);
}

#[test]
fn missing_chrome_does_not_pretend_a_png_was_written() {
    let dir = scratch("png");
    let svg = dir.join("blank.svg");
    std::fs::write(&svg, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"></svg>").unwrap_or_else(|err| panic!("{err}"));
    let png = dir.join("blank.png");
    assert!(!crate::svg_to_png(&svg, &png, 10.0, 10.0, 2.0, Some(Path::new("missing-chrome-binary"))));
    assert!(!png.is_file());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn chrome_screenshots_an_svg_when_a_browser_exists() {
    let Some(chrome) = crate::find_chrome() else {
        return;
    };
    let dir = scratch("chrome");
    let svg = dir.join("box.svg");
    std::fs::write(&svg, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"10\"><rect width=\"20\" height=\"10\" fill=\"#000\"/></svg>").unwrap_or_else(|err| panic!("{err}"));
    let png = dir.join("box.png");
    assert!(crate::svg_to_png(&svg, &png, 20.0, 10.0, 1.0, Some(&chrome)), "chrome failed: {}", chrome.display());
    let bytes = std::fs::read(&png).unwrap_or_else(|err| panic!("{err}"));
    assert!(bytes.starts_with(b"\x89PNG"), "not a png");
    let _ = std::fs::remove_dir_all(dir);
}
