use indexmap::IndexMap;
use serde_json::{Value, json};

use crate::{Block, Diagram, Graph, LinkSpec, Panel, Theme, Tile, Tiles, from_mermaid, from_spec};

fn panel(title: &str, label: &str) -> Result<Panel, crate::DiagramError> {
    Ok(Panel::new(title).section(Tiles::new(vec![Tile::new(label)], None, None)?.into()))
}

fn nodes(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, label)| ((*key).to_owned(), (*label).to_owned()))
        .collect()
}

fn spots(pairs: &[(&str, f64, f64)]) -> IndexMap<String, (f64, f64)> {
    pairs
        .iter()
        .map(|(key, x, y)| ((*key).to_owned(), (*x, *y)))
        .collect()
}

fn problems_of(diagram: &mut Diagram) -> Result<String, crate::DiagramError> {
    Ok(diagram.render()?.problems.join("\n"))
}

fn keep(log: &mut Vec<String>, missing: &mut Vec<String>, text: &str, needles: &[&str]) {
    println!("{text}");
    log.push(text.to_owned());
    for needle in needles {
        if !text.contains(needle) {
            missing.push(format!("missing `{needle}` in: {text}"));
        }
    }
}

fn expect_sentences(
    result: Result<String, impl std::fmt::Display>,
    context: &str,
    needles: &[&str],
) {
    let mut missing = Vec::new();
    let mut log = Vec::new();
    match result {
        Ok(text) => keep(&mut log, &mut missing, &text, needles),
        Err(err) => missing.push(format!("{context}: {err}")),
    }
    assert!(
        missing.is_empty(),
        "{}\n---\n{}",
        missing.join("\n"),
        log.join("\n---\n")
    );
}

#[test]
fn overlap_gap_cross_and_width_sentences() {
    expect_sentences(
        overlap_gap_cross_and_width(),
        "layout checks",
        &[
            "'Alpha' and 'Beta' overlap",
            "only 5 px apart",
            "crosses 'Beta'",
            "needs ",
            " px wide",
            "smallest text",
        ],
    );
}

#[test]
fn group_facing_and_title_sentences() {
    expect_sentences(
        group_facing_and_title(),
        "group checks",
        &[
            "group 'Box' overlaps 'Outside'",
            "facing away",
            "sits on the title of group 'Group title'",
        ],
    );
}

#[test]
fn graph_problem_sentences() {
    expect_sentences(
        graph_messages(),
        "graph checks",
        &[
            "graph: the label ",
            "touches the node",
            "graph: the line ",
            "runs through",
        ],
    );
}

#[test]
fn describe_wording_sentences() {
    expect_sentences(
        describe_wording(),
        "describe",
        &[
            "->",
            "labeled \"new\"",
            "(being built)",
            "[step 1, planned]",
            "(connection to build)",
        ],
    );
}

#[test]
fn quickstart_describe_sentences() {
    expect_sentences(
        crate::examples::quickstart().map(|diagram| diagram.describe()),
        "quickstart describe",
        &[
            "How a search request is answered",
            "Web app: Search box; Results page; Filters",
            "Caches results (being built)",
            "Search index [planned]",
            "Search API -> Search index",
            "(connection to build)",
            "labeled \"new\"",
        ],
    );
}

#[test]
fn spec_error_sentences() {
    expect_sentences(
        spec_errors(),
        "spec errors",
        &[
            "columns[0][0].tiles[0]",
            "columns[1][0].tiles[2]",
            "'colour'",
            "no panel with id 'c'",
            "needs numbers for x and y",
            "only work in the 'panels' layout",
            "exactly one layout",
            "; got columns, ring",
        ],
    );
}

#[test]
fn mermaid_error_sentences() {
    expect_sentences(mermaid_errors(), "mermaid errors", &["line 2", "line 7"]);
}

fn overlap_gap_cross_and_width() -> Result<String, crate::DiagramError> {
    let mut out = Vec::new();

    let mut overlap = Diagram::new("t");
    overlap.add(panel("Alpha", "x")?, 0.0, 0.0)?;
    overlap.add(panel("Beta", "y")?, 20.0, 20.0)?;
    out.push(problems_of(&mut overlap)?);

    let mut gap = Diagram::new("t");
    let alpha = gap.add(panel("Alpha", "x")?, 0.0, 0.0)?;
    gap.right_of(&alpha, panel("Beta", "y")?, Some(5.0), "top", None, None)?;
    out.push(problems_of(&mut gap)?);

    let mut crossing = Diagram::new("t");
    let left = crossing.add(panel("Alpha", "x")?, 0.0, 0.0)?;
    let mid = crossing.right_of(&left, panel("Beta", "y")?, None, "center", None, None)?;
    let right = crossing.right_of(&mid, panel("Gamma", "z")?, None, "center", None, None)?;
    crossing.link(&left, &right, LinkSpec::default())?;
    let crossed = problems_of(&mut crossing)?;
    out.push(crossed);

    let mut allowed = Diagram::new("t");
    let left = allowed.add(panel("Alpha", "x")?, 0.0, 0.0)?;
    let mid = allowed.right_of(&left, panel("Beta", "y")?, None, "center", None, None)?;
    let right = allowed.right_of(&mid, panel("Gamma", "z")?, None, "center", None, None)?;
    allowed.link(&left, &right, LinkSpec::default().cross_ok(true))?;
    let clean = problems_of(&mut allowed)?;
    if !clean.is_empty() {
        return Err(crate::DiagramError::new(format!(
            "cross_ok still reported: {clean}"
        )));
    }

    let mut narrow = Diagram::new("t");
    narrow.add_named(panel("Wide", "x")?, 0.0, 0.0, Some(10.0), None)?;
    out.push(problems_of(&mut narrow)?);

    let theme = Theme {
        min_text: 30.0,
        ..Theme::default()
    };
    let mut tiny = Diagram::new("t").theme(theme);
    tiny.add(panel("Alpha", "x")?, 0.0, 0.0)?;
    out.push(problems_of(&mut tiny)?);

    Ok(out.join("\n"))
}

fn group_facing_and_title() -> Result<String, crate::DiagramError> {
    let mut out = Vec::new();

    let mut grouped = Diagram::new("t");
    let inside = grouped.add(panel("Inside", "x")?, 0.0, 0.0)?;
    grouped.add(panel("Outside", "y")?, 40.0, 10.0)?;
    grouped.group("Box", std::slice::from_ref(&inside), None, Some(80.0));
    out.push(problems_of(&mut grouped)?);

    let mut away = Diagram::new("t");
    let left = away.add(panel("Left", "x")?, 0.0, 0.0)?;
    let right = away.right_of(&left, panel("Right", "y")?, None, "top", None, None)?;
    away.link(&left, &right, LinkSpec::default().start("left").end("left"))?;
    out.push(problems_of(&mut away)?);

    let mut titled = Diagram::new("t");
    let start = titled.add(panel("A", "x")?, 0.0, 60.0)?;
    let end = titled.add(panel("B", "y")?, 420.0, 100.0)?;
    titled.group("Group title", std::slice::from_ref(&end), None, None);
    titled.link(
        &start,
        &end,
        LinkSpec::default()
            .via_y(78.0)
            .label("a long label that reaches the group title"),
    )?;
    out.push(problems_of(&mut titled)?);

    Ok(out.join("\n"))
}

fn graph_messages() -> Result<String, crate::DiagramError> {
    let mut out = Vec::new();

    let touching = Graph::new(
        nodes(&[("a", "A"), ("b", "B")]),
        vec![(
            "a".to_owned(),
            "b".to_owned(),
            Some("a very long label for a short line".to_owned()),
        )],
        spots(&[("a", 0.0, 0.0), ("b", 0.6, 0.0)]),
    )?;
    let mut diagram = Diagram::new("t");
    diagram.add(Panel::new("Map").section(Block::from(touching)), 0.0, 0.0)?;
    out.push(problems_of(&mut diagram)?);

    let through = Graph::new(
        nodes(&[("a", "A"), ("m", "Mid"), ("c", "C")]),
        vec![("a".to_owned(), "c".to_owned(), None)],
        spots(&[("a", 0.0, 0.0), ("m", 1.0, 0.0), ("c", 2.0, 0.0)]),
    )?;
    let mut diagram = Diagram::new("t");
    diagram.add(Panel::new("Map").section(Block::from(through)), 0.0, 0.0)?;
    out.push(problems_of(&mut diagram)?);

    Ok(out.join("\n"))
}

fn describe_wording() -> Result<String, crate::DiagramError> {
    let mut diagram = Diagram::new("How a search request is answered");
    let search = diagram.add(
        Panel::new("Search").step(1).status("planned").section(
            Tiles::new(
                vec![Tile::new("Caches results").status("building")],
                None,
                None,
            )?
            .into(),
        ),
        0.0,
        0.0,
    )?;
    let index = diagram.right_of(&search, panel("Index", "rows")?, None, "top", None, None)?;
    diagram.link(
        &search,
        &index,
        LinkSpec::default().kind("planned").label("new"),
    )?;
    let once = diagram.describe();
    let twice = diagram.describe();
    if once != twice {
        return Err(crate::DiagramError::new("describe() changed between calls"));
    }
    Ok(once)
}

fn spec_errors() -> Result<String, crate::SpecError> {
    let mut out = Vec::new();
    let colour = json!({
        "title": "t",
        "columns": [[{"id": "a", "title": "Alpha", "tiles": [{"label": "x", "colour": "red"}]}]]
    });
    out.push(expect_spec(&colour)?);
    let nested = json!({
        "title": "t",
        "columns": [
            [{"title": "A", "tiles": ["ok"]}],
            [{"title": "B", "tiles": ["one", "two", {"label": "three", "colour": "red"}]}]
        ]
    });
    out.push(expect_spec(&nested)?);
    let missing = json!({
        "title": "t",
        "columns": [[{"id": "a", "title": "Alpha", "tiles": ["x"]}]],
        "links": [{"from": "a", "to": "c"}]
    });
    out.push(expect_spec(&missing)?);
    out.push(expect_spec(
        &json!({"title": "t", "panels": [{"title": "A", "tiles": ["x"], "x": 0}]}),
    )?);
    out.push(expect_spec(
        &json!({"title": "t", "columns": [[{"title": "A", "x": 1}]]}),
    )?);
    out.push(expect_spec(
        &json!({"title": "t", "columns": [[{"title": "A"}]], "ring": [{"title": "B"}]}),
    )?);
    out.push(expect_spec(&json!({"title": "t"}))?);
    Ok(out.join("\n"))
}

fn expect_spec(value: &Value) -> Result<String, crate::SpecError> {
    match from_spec(value) {
        Ok(_) => Err(crate::SpecError::new(format!(
            "expected a spec error for {value}"
        ))),
        Err(err) => Ok(err.to_string()),
    }
}

fn mermaid_errors() -> Result<String, crate::MermaidError> {
    let line_2 = match from_mermaid("flowchart LR\n  a[ok] --> ???", None, None, None) {
        Ok(_) => {
            return Err(crate::MermaidError::new(
                "expected a mermaid error on line 2",
            ));
        }
        Err(err) => err.to_string(),
    };
    let line_7 = match from_mermaid(
        "flowchart LR\n  a[ok] --> b[ok]\n  b --> c[ok]\n  c --> d[ok]\n  d --> e[ok]\n  e --> f[ok]\n  f --> ???",
        None,
        None,
        None,
    ) {
        Ok(_) => {
            return Err(crate::MermaidError::new(
                "expected a mermaid error on line 7",
            ));
        }
        Err(err) => err.to_string(),
    };
    Ok(format!("{line_2}\n{line_7}"))
}
