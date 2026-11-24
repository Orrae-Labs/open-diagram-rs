use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::transport::TokioChildProcess;
use tokio::process::Command;

#[tokio::test]
async fn stdio_tools_draw_inside_the_output_folder() {
    let out = scratch().expect("scratch");
    let mut command = Command::new(env!("CARGO_BIN_EXE_open-diagram-rs-mcp"));
    command.arg("--out").arg(&out).kill_on_drop(true);
    let client =
        ().serve(TokioChildProcess::new(command).expect("spawn"))
            .await
            .expect("stdio handshake");

    let tools = client.list_all_tools().await.expect("list tools");
    assert_tools(&tools);

    let info = client.peer_info().expect("server info");
    let server = info.server_info.as_ref().expect("implementation");
    assert_eq!(server.name, "open-diagram-rs");
    assert!(
        info.instructions
            .as_deref()
            .is_some_and(|text| text.contains("layout problem")),
        "{info:?}"
    );

    let guide = client
        .call_tool(CallToolRequestParams::new("format_guide"))
        .await
        .expect("format_guide");
    let guide_text = text_of(&guide);
    assert!(guide_text.contains("Icons available:"), "{guide_text}");
    assert!(!guide.is_error.unwrap_or(false));

    let checked = client
        .call_tool(args(
            "check",
            serde_json::json!({
                "source": "flowchart LR\n  A[Start] --> B[End]\n"
            }),
        ))
        .await
        .expect("check");
    let checked_text = text_of(&checked);
    assert!(
        checked_text.starts_with("0 layout problem") || checked_text.contains("layout problem"),
        "{checked_text}"
    );
    assert!(!checked.is_error.unwrap_or(false), "{checked_text}");

    let missing = client
        .call_tool(args(
            "template",
            serde_json::json!({"name": "not-a-template"}),
        ))
        .await
        .expect("template");
    assert!(missing.is_error.unwrap_or(false), "{}", text_of(&missing));
    assert!(
        text_of(&missing).contains("no template named"),
        "{}",
        text_of(&missing)
    );

    let drawn = client
        .call_tool(args(
            "draw",
            serde_json::json!({
                "source": "flowchart LR\n  A[Start] --> B[End]\n",
                "format": "mermaid",
                "name": "../escape",
                "picture": true
            }),
        ))
        .await
        .expect("draw");
    let drawn_text = text_of(&drawn);
    let svg = out.join("escape.svg");
    assert!(svg.is_file(), "{drawn_text}");
    assert!(drawn_text.contains(&format!("- {}", svg.display())));
    assert!(!out.parent().expect("parent").join("escape.svg").exists());
    assert_picture(&drawn, &drawn_text);

    let skills = client.list_all_skills().await.expect("skills");
    let uris: Vec<_> = skills.iter().map(|skill| skill.uri.clone()).collect();
    assert!(
        uris.iter()
            .any(|uri| uri == "skill://open-diagram-rs/SKILL.md"),
        "{uris:?}"
    );
    assert!(
        uris.iter().any(|uri| uri == "skill://cloud-icons/SKILL.md"),
        "{uris:?}"
    );

    client.cancel().await.expect("cancel");
    std::fs::remove_dir_all(&out).unwrap_or_else(|err| panic!("remove {}: {err}", out.display()));
}

#[expect(
    clippy::unwrap_used,
    reason = "this helper runs only from the stdio integration test"
)]
#[expect(
    clippy::expect_used,
    reason = "this helper runs only from the stdio integration test"
)]
fn assert_tools(tools: &[rmcp::model::Tool]) {
    let mut names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        ["check", "describe", "draw", "format_guide", "template"]
    );
    let check = tools.iter().find(|tool| tool.name == "check").unwrap();
    let notes = check.annotations.as_ref().expect("check annotations");
    assert_eq!(notes.read_only_hint, Some(true));
    assert_eq!(notes.open_world_hint, Some(false));
    let draw = tools.iter().find(|tool| tool.name == "draw").unwrap();
    let notes = draw.annotations.as_ref().expect("draw annotations");
    assert_eq!(notes.read_only_hint, Some(false));
    assert_eq!(notes.destructive_hint, Some(false));
    assert_eq!(notes.open_world_hint, Some(false));
}

#[expect(
    clippy::expect_used,
    reason = "this helper runs only from the stdio integration test"
)]
fn assert_picture(drawn: &rmcp::model::CallToolResult, drawn_text: &str) {
    if drawn_text.contains("PNG skipped") {
        assert!(drawn.content.iter().all(|block| block.as_image().is_none()));
        return;
    }
    let image = drawn
        .content
        .iter()
        .find_map(|block| block.as_image())
        .expect("png image");
    assert_eq!(image.mime_type, "image/png");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(image.data.as_bytes())
        .expect("base64");
    assert!(bytes.starts_with(b"\x89PNG"), "png magic");
}

fn args(name: &str, value: serde_json::Value) -> CallToolRequestParams {
    let arguments = match value {
        serde_json::Value::Object(arguments) => arguments,
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_)
        | serde_json::Value::Array(_) => serde_json::Map::new(),
    };
    CallToolRequestParams::new(name.to_owned()).with_arguments(arguments)
}

fn text_of(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|block| block.as_text().map(|text| text.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn scratch() -> std::io::Result<PathBuf> {
    static N: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "open-diagram-rs-mcp-stdio-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
