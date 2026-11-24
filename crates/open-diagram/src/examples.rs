use indexmap::IndexMap;

use crate::blocks::{Block, Card, Graph, Note, Panel, Row, Tile, Tiles};
use crate::diagram::{Diagram, LinkSpec, Placed};
use crate::error::DiagramError;

fn tile(label: &str) -> Tile {
    Tile::new(label)
}

fn marked(label: &str, status: &str) -> Tile {
    Tile::new(label).status(status)
}

fn tiles(
    items: Vec<Tile>,
    cols: Option<usize>,
    label: Option<&str>,
) -> Result<Block, DiagramError> {
    Ok(Tiles::new(items, cols, label.map(str::to_owned))?.into())
}

fn rows(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn names(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn spots(pairs: &[(&str, f64, f64)]) -> IndexMap<String, (f64, f64)> {
    pairs
        .iter()
        .map(|(key, x, y)| ((*key).to_owned(), (*x, *y)))
        .collect()
}

fn edges(pairs: &[(&str, &str, &str)]) -> Vec<(String, String, Option<String>)> {
    pairs
        .iter()
        .map(|(a, b, label)| ((*a).to_owned(), (*b).to_owned(), Some((*label).to_owned())))
        .collect()
}

fn steps(pairs: &[(i64, &str)]) -> Vec<(i64, String)> {
    pairs
        .iter()
        .map(|(n, label)| (*n, (*label).to_owned()))
        .collect()
}

fn flow() -> LinkSpec {
    LinkSpec::default()
}

fn span(diagram: &Diagram) -> (f64, f64) {
    let top = diagram.placed().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let bottom = diagram
        .placed()
        .map(Placed::bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    (top, bottom)
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn quickstart() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("How a search request is answered").tag("EXAMPLE");
    let app = diagram.add(
        Panel::new("Web app")
            .section(tiles(
                vec![tile("Search box"), tile("Results page"), tile("Filters")],
                None,
                None,
            )?)
            .footer("Owner: Web team"),
        0.0,
        0.0,
    )?;
    let api = diagram.right_of(
        &app,
        Panel::new("Search API")
            .section(tiles(
                vec![tile("Checks access"), marked("Caches results", "building")],
                None,
                None,
            )?)
            .footer("Owner: Platform team"),
        None,
        "center",
        None,
        None,
    )?;
    let index = diagram.right_of(
        &api,
        Panel::new("Search index")
            .section(tiles(
                vec![tile("Product text"), marked("Spelling fixes", "planned")],
                None,
                None,
            )?)
            .status("planned"),
        None,
        "center",
        None,
        None,
    )?;
    diagram.link(&app, &api, flow())?;
    diagram.link(&api, &index, flow().kind("planned").label("new"))?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn card_payment() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("What happens when you tap a card")
        .subtitle("Simplified: one in-store payment, start to finish")
        .tag("EXAMPLE")
        .steps(
            steps(&[
                (1, "Tap"),
                (2, "Ask the card's bank"),
                (3, "Approve or decline"),
                (4, "Money moves later"),
            ]),
            "Order",
        );
    let placed = diagram.columns(
        vec![
            vec![
                Panel::new("Shopper")
                    .section(tiles(
                        vec![tile("Card"), tile("Phone wallet")],
                        Some(1),
                        None,
                    )?)
                    .step(1)
                    .into(),
            ],
            vec![
                Panel::new("Shop")
                    .section(tiles(
                        vec![tile("Card reader"), tile("Checkout system")],
                        Some(1),
                        None,
                    )?)
                    .into(),
            ],
            vec![
                Panel::new("Shop's bank")
                    .section(tiles(
                        vec![tile("Payment processor"), tile("Fraud screening")],
                        Some(1),
                        None,
                    )?)
                    .subtitle("The acquirer")
                    .step(4)
                    .into(),
            ],
            vec![
                Panel::new("Card network")
                    .section(tiles(
                        vec![tile("Routes the request"), tile("Network rules")],
                        Some(1),
                        None,
                    )?)
                    .step(2)
                    .into(),
            ],
            vec![
                Panel::new("Card's bank")
                    .section(tiles(
                        vec![
                            tile("Checks the balance"),
                            tile("Fraud checks"),
                            tile("Approves or declines"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .subtitle("The issuer")
                    .step(3)
                    .into(),
            ],
        ],
        (0.0, 0.0),
        Some(64.0),
        None,
        "center",
        true,
    )?;
    let shopper = &placed[0][0];
    let shop = &placed[1][0];
    let acquirer = &placed[2][0];
    let network = &placed[3][0];
    let issuer = &placed[4][0];
    diagram.link(shopper, shop, flow().label("tap"))?;
    diagram.link(shop, acquirer, flow())?;
    diagram.link(acquirer, network, flow())?;
    diagram.link(network, issuer, flow())?;
    let (top, bottom) = span(&diagram);
    diagram.link(
        issuer,
        shop,
        flow()
            .start("top")
            .end("top")
            .via_y(top - 56.0)
            .label("The answer comes back the same way, in seconds"),
    )?;
    diagram.link(
        issuer,
        acquirer,
        flow()
            .start("bottom")
            .end("bottom")
            .via_y(bottom + 44.0)
            .label("Money moves later, in a batch"),
    )?;
    Ok(diagram)
}

fn order_map() -> Result<Graph, DiagramError> {
    Graph::new(
        names(&[
            ("customer", "Customer"),
            ("order", "Order"),
            ("payment", "Payment"),
            ("item", "Item"),
            ("delivery", "Delivery"),
            ("return", "Return"),
        ]),
        edges(&[
            ("customer", "order", "places"),
            ("order", "payment", "paid by"),
            ("order", "item", "includes"),
            ("order", "delivery", "ships as"),
            ("item", "return", "may come back as"),
        ]),
        spots(&[
            ("customer", 0.0, 0.0),
            ("order", 1.5, 0.0),
            ("payment", 3.0, 0.0),
            ("return", 0.0, 2.0),
            ("item", 1.5, 2.0),
            ("delivery", 3.0, 2.0),
        ]),
    )
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn business_map() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("Pinemoor Home: from order to doorstep")
        .subtitle("What runs today, what is being built, and what comes next")
        .tag("EXAMPLE")
        .note(Note::new(
            "Example",
            [
                "Pinemoor Home is an invented company.",
                "Every system, owner and status is made up.",
            ],
        ))
        .steps(
            steps(&[
                (1, "Fraud checks in checkout"),
                (2, "One record for each order"),
                (3, "Book returns online"),
            ]),
            "Rollout",
        );
    let order = Row::new(
        [
            Block::from(order_map()?),
            Block::from(Card::new(
                "Example: order 10482",
                rows(&[
                    ("Customer", "Online account"),
                    ("Items", "Sofa, two cushions"),
                    ("Paid by", "Card, in full"),
                    ("Delivery", "Two-person, Tuesday"),
                ]),
                None,
            )),
        ],
        None,
    );
    let placed = diagram.columns(
        vec![
            vec![
                Panel::new("Where orders come from")
                    .section(tiles(
                        vec![
                            tile("Website"),
                            tile("Mobile app"),
                            marked("Showroom tablets", "retiring"),
                            marked("Marketplace listings", "trial"),
                        ],
                        Some(2),
                        None,
                    )?)
                    .subtitle("Four ways in")
                    .footer("Owner: Digital team")
                    .into(),
            ],
            vec![
                Panel::new("Checkout")
                    .section(tiles(
                        vec![
                            tile("Basket"),
                            tile("Payments"),
                            marked("Fraud checks", "building"),
                            marked("Buy now, pay later", "planned"),
                        ],
                        Some(2),
                        None,
                    )?)
                    .step(1)
                    .footer("Owner: Payments team")
                    .into(),
                Panel::new("Order record")
                    .section(order.into())
                    .subtitle("Every order kept in one place")
                    .step(2)
                    .status("building")
                    .footer("Owner: Platform team")
                    .into(),
            ],
            vec![
                Panel::new("Fulfilment")
                    .section(tiles(
                        vec![tile("Pick and pack"), tile("Stock counts")],
                        None,
                        Some("Warehouse"),
                    )?)
                    .section(tiles(
                        vec![
                            tile("Two-person delivery"),
                            marked("Same-day vans", "trial"),
                            marked("Parcel lockers", "planned"),
                        ],
                        None,
                        Some("Delivery"),
                    )?)
                    .footer("Owner: Operations")
                    .into(),
                Panel::new("Returns")
                    .section(tiles(
                        vec![tile("Book a collection"), tile("Refund on pickup")],
                        None,
                        None,
                    )?)
                    .step(3)
                    .status("planned")
                    .footer("Owner: Not yet named")
                    .into(),
            ],
            vec![
                Panel::new("Reporting")
                    .section(tiles(
                        vec![
                            tile("Daily sales"),
                            tile("Stock cover"),
                            marked("Demand forecast", "trial"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .tone("neutral")
                    .footer("Owner: Finance")
                    .into(),
                Panel::new("Customer service")
                    .section(tiles(
                        vec![
                            tile("Order lookup"),
                            tile("Delivery changes"),
                            marked("Chat assistant", "building"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .tone("neutral")
                    .footer("Owner: Service team")
                    .into(),
            ],
        ],
        (0.0, 0.0),
        Some(88.0),
        Some(40.0),
        "center",
        true,
    )?;
    diagram.group(
        "Order platform",
        &[placed[1][0].clone(), placed[1][1].clone()],
        Some("Run by the platform team".to_owned()),
        None,
    );
    diagram.link(&placed[0][0], &placed[1][0], flow())?;
    diagram.link(
        &placed[1][0],
        &placed[1][1],
        flow().start("bottom").end("top"),
    )?;
    diagram.link(&placed[1][1], &placed[2][0], flow())?;
    diagram.link(&placed[1][1], &placed[2][1], flow().kind("planned"))?;
    diagram.link(&placed[2][0], &placed[3][0], flow())?;
    diagram.link(&placed[2][0], &placed[3][1], flow().label("updates"))?;
    diagram.link(&placed[2][1], &placed[3][0], flow().kind("planned"))?;
    let (top, bottom) = span(&diagram);
    diagram.link(
        &placed[0][0],
        &placed[3][0],
        flow()
            .start("top")
            .end("top")
            .via_y(top - 80.0)
            .label("Web traffic, sent nightly"),
    )?;
    diagram.link(
        &placed[0][0],
        &placed[3][1],
        flow()
            .start("bottom")
            .end("bottom")
            .via_y(bottom + 48.0)
            .label("Marketplace questions come straight to service"),
    )?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when a block cannot be measured.
pub fn concept_map() -> Result<Diagram, DiagramError> {
    let map = Graph::new(
        names(&[
            ("branch", "Branch"),
            ("tag", "Tag"),
            ("commit", "Commit"),
            ("parent", "Parent commit"),
            ("tree", "Tree"),
            ("blob", "Blob"),
            ("subtree", "Subfolder tree"),
        ]),
        edges(&[
            ("branch", "commit", "points to"),
            ("tag", "commit", "points to"),
            ("commit", "parent", "follows"),
            ("commit", "tree", "saves"),
            ("tree", "blob", "lists file"),
            ("tree", "subtree", "lists folder"),
        ]),
        spots(&[
            ("branch", 0.0, 0.0),
            ("commit", 1.5, 0.0),
            ("tree", 3.0, 0.0),
            ("blob", 4.5, 0.0),
            ("tag", 0.0, 2.0),
            ("parent", 1.5, 2.0),
            ("subtree", 4.5, 2.0),
        ]),
    )?;
    let example = Card::new(
        "Example: one commit",
        rows(&[
            ("Snapshot", "Tree 9c1e4a"),
            ("Parent", "Commit 52f0b7"),
            ("Message", "Fix sign-in on phones"),
            ("Branch", "main points here"),
        ]),
        Some("One commit, as git stores it:".to_owned()),
    );
    let mut diagram = Diagram::new("How git stores a project").tag("EXAMPLE");
    diagram.add(
        Panel::new("Objects and how they connect")
            .section(Row::new([Block::from(map), Block::from(example)], None).into())
            .subtitle("Each object is named by a hash of its contents"),
        0.0,
        0.0,
    )?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn how_it_works() -> Result<Diagram, DiagramError> {
    let stages: Vec<(&str, &str, Vec<Tile>)> = vec![
        (
            "Your script",
            "Written by you or an agent",
            vec![
                tile("Words in lists"),
                tile("Panels and tiles"),
                tile("Lines and groups"),
            ],
        ),
        (
            "Measure",
            "From the font file",
            vec![
                tile("Letter widths"),
                tile("Kerning pairs"),
                tile("Line wrapping"),
            ],
        ),
        (
            "Place",
            "Sizes come from Measure",
            vec![
                tile("Columns"),
                tile("Next to, below"),
                tile("Shaded groups"),
            ],
        ),
        (
            "Check",
            "Every rule, every build",
            vec![
                tile("Overlaps"),
                tile("Gaps under 16 px"),
                tile("Lines through boxes"),
                tile("Labels on boxes"),
                tile("Text under 10 px"),
            ],
        ),
        (
            "Draw",
            "Same look everywhere",
            vec![
                tile("Text as outlines"),
                tile("Curves and lanes"),
                tile("Key from what is used"),
                marked("3D icons", "planned"),
            ],
        ),
        (
            "Output",
            "Files first, then problems",
            vec![tile("SVG file"), tile("PNG file"), tile("Problem report")],
        ),
    ];
    let mut columns = Vec::new();
    for (name, sub, items) in stages {
        let tone = if name == "Check" { "accent" } else { "default" };
        columns.push(vec![
            Panel::new(name)
                .section(tiles(items, Some(1), None)?)
                .subtitle(sub)
                .tone(tone)
                .into(),
        ]);
    }
    let mut diagram =
        Diagram::new("How open-diagram turns a script into a picture").tag("HOW IT WORKS");
    let placed = diagram.columns(columns, (0.0, 0.0), Some(64.0), None, "center", true)?;
    let panels: Vec<Placed> = placed.iter().map(|col| col[0].clone()).collect();
    for pair in panels.windows(2) {
        diagram.link(&pair[0], &pair[1], flow())?;
    }
    let bottom = panels
        .iter()
        .map(Placed::bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    let (Some(last), Some(first)) = (panels.last(), panels.first()) else {
        return Err(DiagramError::new("how it works has no stages"));
    };
    diagram.link(
        last,
        first,
        flow()
            .start("bottom")
            .end("bottom")
            .via_y(bottom + 44.0)
            .label("Each problem comes back by name: fix it and run again"),
    )?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn incident_response() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("When the site goes down: from alert to lesson learned")
        .tag("EXAMPLE")
        .note(Note::new(
            "Example",
            [
                "An invented team's process.",
                "Every tool and status is made up.",
            ],
        ));
    let placed = diagram.columns(
        vec![
            vec![
                Panel::new("Detect")
                    .section(tiles(
                        vec![
                            tile("Error-rate alerts"),
                            tile("Uptime checks"),
                            marked("Unusual traffic", "trial"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .footer("Owner: Reliability team")
                    .into(),
            ],
            vec![
                Panel::new("Respond")
                    .section(tiles(
                        vec![
                            tile("Page the on-call engineer"),
                            tile("Open an incident channel"),
                            marked("Assign roles automatically", "building"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .footer("Owner: On-call rotation")
                    .into(),
            ],
            vec![
                Panel::new("Fix")
                    .section(tiles(
                        vec![
                            tile("Roll back"),
                            tile("Add servers"),
                            tile("Turn off a feature"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .into(),
                Panel::new("Tell people")
                    .section(tiles(
                        vec![
                            tile("Status page"),
                            tile("Support team"),
                            marked("Email affected users", "planned"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .tone("neutral")
                    .into(),
            ],
            vec![
                Panel::new("Learn")
                    .section(tiles(
                        vec![
                            tile("Timeline"),
                            tile("Blameless review"),
                            tile("Follow-up tasks"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .tone("accent")
                    .footer("Owner: Incident lead")
                    .into(),
            ],
        ],
        (0.0, 0.0),
        Some(136.0),
        Some(32.0),
        "center",
        true,
    )?;
    diagram.link(&placed[0][0], &placed[1][0], flow())?;
    diagram.link(&placed[1][0], &placed[2][0], flow())?;
    diagram.link(&placed[1][0], &placed[2][1], flow().label("every 30 min"))?;
    diagram.link(&placed[2][0], &placed[3][0], flow())?;
    diagram.link(&placed[2][1], &placed[3][0], flow().label("what users saw"))?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn layers() -> Result<Diagram, DiagramError> {
    let specs: Vec<(&str, Vec<Tile>, &str)> = vec![
        (
            "People",
            vec![tile("Shoppers"), tile("Store staff"), tile("Partners")],
            "accent",
        ),
        (
            "Apps",
            vec![
                tile("Website"),
                tile("Mobile app"),
                marked("Staff tablets", "retiring"),
            ],
            "default",
        ),
        (
            "Services",
            vec![
                tile("Search"),
                tile("Orders"),
                tile("Payments"),
                marked("Notifications", "building"),
            ],
            "default",
        ),
        (
            "Data",
            vec![
                tile("Main database"),
                tile("Search index"),
                marked("Event log", "planned"),
            ],
            "neutral",
        ),
    ];
    let mut blocks = Vec::new();
    for (name, items, tone) in specs {
        let cols = items.len();
        blocks.push(
            Panel::new(name)
                .section(tiles(items, Some(cols), None)?)
                .tone(tone)
                .into(),
        );
    }
    let mut diagram = Diagram::new("A web product, top to bottom").tag("EXAMPLE");
    let stacked = diagram.stack(blocks, 0.0, 0.0, Some(36.0))?;
    for pair in stacked.windows(2) {
        diagram.link(&pair[0], &pair[1], flow().start("bottom").end("top"))?;
    }
    let side = Panel::new("Sign-in and access")
        .section(tiles(
            vec![
                tile("Single sign-on"),
                marked("Passkeys", "trial"),
                tile("Audit log"),
            ],
            Some(1),
            None,
        )?)
        .footer("Owner: Security team");
    let (_, height) = side.measure(&diagram.theme)?;
    let span = stacked[1].y + stacked[2].bottom();
    let middle = span / 2.0;
    let access = diagram.add(side, stacked[0].right() + 96.0, middle - height / 2.0)?;
    diagram.link(
        &access,
        &stacked[1],
        flow().start("left").end("right").kind("planned"),
    )?;
    diagram.link(
        &access,
        &stacked[2],
        flow().start("left").end("right").kind("planned"),
    )?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn pipeline() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("From pull request to production")
        .tag("EXAMPLE")
        .steps(
            steps(&[
                (1, "Cache every build"),
                (2, "Scan every image"),
                (3, "Release to 5% first"),
            ]),
            "This quarter",
        );
    let placed = diagram.columns(
        vec![
            vec![
                Panel::new("Code")
                    .section(tiles(
                        vec![tile("Pull request"), tile("Code review")],
                        Some(1),
                        None,
                    )?)
                    .footer("Owner: Every team")
                    .into(),
            ],
            vec![
                Panel::new("Build")
                    .section(tiles(
                        vec![
                            tile("Compile"),
                            tile("Unit tests"),
                            tile("Container image"),
                            marked("Build cache", "building"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .step(1)
                    .footer("Owner: Build team")
                    .into(),
            ],
            vec![
                Panel::new("Checks")
                    .section(tiles(
                        vec![
                            tile("Integration tests"),
                            marked("Security scan", "building"),
                            marked("Load test", "planned"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .step(2)
                    .footer("Owner: Quality team")
                    .into(),
            ],
            vec![
                Panel::new("Release")
                    .section(tiles(
                        vec![
                            tile("Staging"),
                            marked("Canary, 5% of users", "trial"),
                            tile("Production"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .step(3)
                    .footer("Owner: Platform team")
                    .into(),
            ],
        ],
        (0.0, 0.0),
        Some(72.0),
        None,
        "center",
        true,
    )?;
    diagram.link(&placed[0][0], &placed[1][0], flow())?;
    diagram.link(&placed[1][0], &placed[2][0], flow())?;
    diagram.link(&placed[2][0], &placed[3][0], flow())?;
    let bottom = placed
        .iter()
        .flatten()
        .map(Placed::bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    diagram.link(
        &placed[3][0],
        &placed[0][0],
        flow()
            .start("bottom")
            .end("bottom")
            .via_y(bottom + 44.0)
            .label("If errors rise: roll back and tell the author"),
    )?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn smart_home() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("A smart home, from sensor to phone")
        .tag("EXAMPLE")
        .note(Note::new(
            "Example",
            ["An invented product.", "Every part and status is made up."],
        ));
    let placed = diagram.columns(
        vec![
            vec![
                Panel::new("Devices")
                    .section(tiles(
                        vec![
                            tile("Thermostat"),
                            tile("Door lock"),
                            tile("Lights"),
                            tile("Motion sensors"),
                        ],
                        Some(2),
                        None,
                    )?)
                    .subtitle("In every room")
                    .into(),
            ],
            vec![
                Panel::new("Home hub")
                    .section(tiles(
                        vec![
                            tile("Device pairing"),
                            tile("Local rules"),
                            marked("Works offline", "building"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .subtitle("Talks to every device")
                    .into(),
            ],
            vec![
                Panel::new("Cloud")
                    .section(tiles(
                        vec![
                            tile("Device registry"),
                            tile("Usage history"),
                            marked("Firmware updates", "building"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .footer("Owner: Platform team")
                    .into(),
            ],
            vec![
                Panel::new("Apps")
                    .section(tiles(
                        vec![
                            tile("Phone app"),
                            marked("Voice assistants", "trial"),
                            marked("Web dashboard", "retiring"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .tone("neutral")
                    .footer("Owner: Apps team")
                    .into(),
            ],
        ],
        (0.0, 0.0),
        Some(80.0),
        None,
        "center",
        true,
    )?;
    diagram.link(&placed[0][0], &placed[1][0], flow())?;
    diagram.link(&placed[1][0], &placed[2][0], flow())?;
    diagram.link(&placed[2][0], &placed[3][0], flow())?;
    let (top, bottom) = span(&diagram);
    diagram.link(
        &placed[3][0],
        &placed[1][0],
        flow()
            .start("top")
            .end("top")
            .via_y(top - 56.0)
            .label("At home, the phone talks to the hub directly"),
    )?;
    diagram.link(
        &placed[2][0],
        &placed[0][0],
        flow()
            .start("bottom")
            .end("bottom")
            .via_y(bottom + 44.0)
            .kind("planned")
            .label("New firmware sent to devices at night"),
    )?;
    Ok(diagram)
}

fn video_map() -> Result<Graph, DiagramError> {
    Graph::new(
        names(&[
            ("channel", "Channel"),
            ("video", "Video"),
            ("rendition", "Rendition"),
            ("caption", "Caption"),
            ("view", "View"),
            ("language", "Language"),
        ]),
        edges(&[
            ("channel", "video", "publishes"),
            ("video", "rendition", "encoded as"),
            ("video", "caption", "has"),
            ("video", "view", "watched in"),
            ("caption", "language", "written in"),
        ]),
        spots(&[
            ("channel", 0.0, 0.0),
            ("video", 1.5, 0.0),
            ("rendition", 3.0, 0.0),
            ("language", 0.0, 2.0),
            ("caption", 1.5, 2.0),
            ("view", 3.0, 2.0),
        ]),
    )
}

/// # Errors
/// Returns an error when a block cannot be measured or a line is not valid.
pub fn systems_map() -> Result<Diagram, DiagramError> {
    let mut diagram = Diagram::new("Quillfin Video: from upload to screen")
        .subtitle("What runs today, what is being built, and what comes next")
        .tag("EXAMPLE")
        .note(Note::new(
            "Example",
            [
                "Quillfin Video is an invented company.",
                "Every system, owner and status is made up.",
            ],
        ))
        .steps(
            steps(&[
                (1, "Encode on GPUs"),
                (2, "One record for each video"),
                (3, "Recommendations"),
            ]),
            "Rollout",
        );
    let record = Row::new(
        [
            Block::from(video_map()?),
            Block::from(Card::new(
                "Example: video 7Q4M",
                rows(&[
                    ("Length", "12 min 40 s"),
                    ("Renditions", "1080p, 720p, 480p"),
                    ("Captions", "English, Spanish"),
                    ("Status", "Ready to play"),
                ]),
                None,
            )),
        ],
        None,
    );
    let placed = diagram.columns(
        vec![
            vec![
                Panel::new("Where videos come from")
                    .section(tiles(
                        vec![
                            tile("Creator app"),
                            tile("Web uploader"),
                            tile("Partner API"),
                            marked("Live streams", "trial"),
                        ],
                        Some(2),
                        None,
                    )?)
                    .subtitle("Four ways in")
                    .footer("Owner: Creator team")
                    .into(),
            ],
            vec![
                Panel::new("Processing")
                    .section(tiles(
                        vec![
                            tile("Upload checks"),
                            tile("Transcoding"),
                            marked("GPU encoding", "building"),
                            marked("Auto captions", "planned"),
                        ],
                        Some(2),
                        None,
                    )?)
                    .step(1)
                    .footer("Owner: Media team")
                    .into(),
                Panel::new("Video record")
                    .section(record.into())
                    .subtitle("Everything known about each video")
                    .step(2)
                    .status("building")
                    .footer("Owner: Media team")
                    .into(),
            ],
            vec![
                Panel::new("Storage")
                    .section(tiles(
                        vec![tile("Original files"), tile("Renditions")],
                        None,
                        Some("Files"),
                    )?)
                    .section(tiles(
                        vec![
                            tile("Metadata database"),
                            marked("Search index", "trial"),
                            marked("Cold archive", "planned"),
                        ],
                        None,
                        Some("Records"),
                    )?)
                    .footer("Owner: Platform team")
                    .into(),
                Panel::new("Recommendations")
                    .section(tiles(
                        vec![tile("Watch history"), tile("Ranking model")],
                        None,
                        None,
                    )?)
                    .step(3)
                    .status("planned")
                    .footer("Owner: Not yet named")
                    .into(),
            ],
            vec![
                Panel::new("Player apps")
                    .section(tiles(
                        vec![tile("Web"), tile("Phones"), marked("TV", "building")],
                        Some(1),
                        None,
                    )?)
                    .tone("neutral")
                    .footer("Owner: Apps team")
                    .into(),
                Panel::new("Service health")
                    .section(tiles(
                        vec![
                            tile("Playback errors"),
                            tile("Start-up time"),
                            marked("Old dashboards", "retiring"),
                        ],
                        Some(1),
                        None,
                    )?)
                    .tone("neutral")
                    .footer("Owner: Reliability team")
                    .into(),
            ],
        ],
        (0.0, 0.0),
        Some(88.0),
        Some(40.0),
        "center",
        true,
    )?;
    diagram.group(
        "Media pipeline",
        &[placed[1][0].clone(), placed[1][1].clone()],
        Some("Run by the media team".to_owned()),
        None,
    );
    diagram.link(&placed[0][0], &placed[1][0], flow())?;
    diagram.link(
        &placed[1][0],
        &placed[1][1],
        flow().start("bottom").end("top"),
    )?;
    diagram.link(&placed[1][1], &placed[2][0], flow())?;
    diagram.link(&placed[1][1], &placed[2][1], flow().kind("planned"))?;
    diagram.link(&placed[2][0], &placed[3][0], flow())?;
    diagram.link(&placed[2][0], &placed[3][1], flow().label("logs"))?;
    diagram.link(&placed[2][1], &placed[3][0], flow().kind("planned"))?;
    let (top, bottom) = span(&diagram);
    diagram.link(
        &placed[0][0],
        &placed[3][0],
        flow()
            .start("top")
            .end("top")
            .via_y(top - 80.0)
            .label("Live streams skip processing"),
    )?;
    diagram.link(
        &placed[0][0],
        &placed[3][1],
        flow()
            .start("bottom")
            .end("bottom")
            .via_y(bottom + 48.0)
            .label("Upload failures go straight to service health"),
    )?;
    Ok(diagram)
}

/// # Errors
/// Returns an error when the template cannot be read.
pub fn aws_architecture() -> Result<Diagram, DiagramError> {
    template("aws_architecture.json")
}

/// # Errors
/// Returns an error when the template cannot be read.
pub fn aws_etl() -> Result<Diagram, DiagramError> {
    template("aws_etl.json")
}

/// # Errors
/// Returns an error when the template cannot be read.
pub fn azure_web_app() -> Result<Diagram, DiagramError> {
    template("azure_web_app.json")
}

/// # Errors
/// Returns an error when the template cannot be read.
pub fn gcp_event_pipeline() -> Result<Diagram, DiagramError> {
    template("gcp_event_pipeline.json")
}

fn template(name: &str) -> Result<Diagram, DiagramError> {
    let text = crate::catalog::read(name).map_err(DiagramError::new)?;
    spec_text(name, &text)
}

fn spec_text(name: &str, text: &str) -> Result<Diagram, DiagramError> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|err| DiagramError::new(format!("{name}: not valid JSON ({err})")))?;
    crate::from_spec(&value).map_err(|err| DiagramError::new(err.to_string()))
}

/// # Errors
/// Returns an error when the JSON description cannot be read.
pub fn email_delivery() -> Result<Diagram, DiagramError> {
    spec_text(
        "examples/email_delivery.json",
        include_str!("../examples/email_delivery.json"),
    )
}

/// # Errors
/// Returns an error when the JSON description cannot be read.
pub fn release_cycle() -> Result<Diagram, DiagramError> {
    spec_text(
        "examples/release_cycle.json",
        include_str!("../examples/release_cycle.json"),
    )
}

/// # Errors
/// Returns an error when the Mermaid flowchart cannot be read.
pub fn signup_flow() -> Result<Diagram, DiagramError> {
    crate::from_mermaid(include_str!("../examples/signup_flow.mmd"), None, None)
        .map_err(|err| DiagramError::new(err.to_string()))
}

/// A named picture builder.
pub type Picture = fn() -> Result<Diagram, DiagramError>;

/// Every picture under `docs/images/` that this crate rebuilds, named like its SVG.
#[must_use]
pub fn pictures() -> Vec<(&'static str, Picture)> {
    vec![
        ("aws_architecture", aws_architecture),
        ("aws_etl", aws_etl),
        ("azure_web_app", azure_web_app),
        ("business_map", business_map),
        ("card_payment", card_payment),
        ("concept_map", concept_map),
        ("email_delivery", email_delivery),
        ("gcp_event_pipeline", gcp_event_pipeline),
        ("how_it_works", how_it_works),
        ("incident_response", incident_response),
        ("layers", layers),
        ("pipeline", pipeline),
        ("quickstart", quickstart),
        ("release_cycle", release_cycle),
        ("signup_flow", signup_flow),
        ("smart_home", smart_home),
        ("systems_map", systems_map),
    ]
}
