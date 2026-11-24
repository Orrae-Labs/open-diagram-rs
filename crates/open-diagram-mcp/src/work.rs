#![expect(
    clippy::redundant_pub_crate,
    reason = "sibling modules use these items; pub would fail unreachable_pub because they are not the external API"
)]

use std::path::{Path, PathBuf};

use open_diagram_rs::{Diagram, catalog, from_mermaid, from_spec_with, py_g, svg_to_png};

use crate::guide::guide_text;

const MAX_SOURCE: usize = 200_000;

pub(crate) struct Drawn {
    pub(crate) text: String,
    pub(crate) png: Option<Vec<u8>>,
}

pub(crate) fn build_diagram(
    source: &str,
    format: &str,
    icons: &[PathBuf],
) -> Result<Diagram, String> {
    let chars = source.chars().count();
    if chars > MAX_SOURCE {
        return Err(format!(
            "the diagram text is {chars} characters; the limit is {MAX_SOURCE}"
        ));
    }
    let text = source.trim();
    let format = if format == "auto" {
        if text.starts_with('{') {
            "json"
        } else {
            "mermaid"
        }
    } else {
        format
    };
    match format {
        "json" => {
            let spec =
                serde_json::from_str(text).map_err(|err| format!("not valid JSON: {err}"))?;
            from_spec_with(&spec, Some(icons)).map_err(|err| err.to_string())
        }
        "mermaid" => from_mermaid(text, None, None, None).map_err(|err| err.to_string()),
        _ => Err("format must be 'json', 'mermaid' or 'auto'".to_owned()),
    }
}

pub(crate) fn problems_text(problems: &[String]) -> String {
    let mut lines = vec![format!("{} layout problem(s)", problems.len())];
    lines.extend(problems.iter().map(|problem| format!("- {problem}")));
    if !problems.is_empty() {
        lines.push("Fix every problem above and run check again.".to_owned());
    }
    lines.join("\n")
}

pub(crate) fn check_text(source: &str, format: &str, icons: &[PathBuf]) -> Result<String, String> {
    let mut diagram = build_diagram(source, format, icons)?;
    let report = diagram.render().map_err(|err| err.to_string())?;
    Ok(problems_text(&report.problems))
}

pub(crate) fn describe_text(
    source: &str,
    format: &str,
    icons: &[PathBuf],
) -> Result<String, String> {
    Ok(build_diagram(source, format, icons)?.describe())
}

pub(crate) fn template_text(name: &str) -> Result<String, String> {
    if name.is_empty() {
        let mut lines =
            vec!["Templates (call template with a name to get one as JSON):".to_owned()];
        for (file, purpose) in catalog::table(None)? {
            let stem = Path::new(&file)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(file.as_str());
            lines.push(format!("- {stem}: {purpose}"));
        }
        return Ok(lines.join("\n"));
    }
    catalog::read(name)
}

pub(crate) fn icons_guide(icons: &[PathBuf]) -> String {
    let mut names = Vec::new();
    for folder in icons {
        names.extend(catalog::icon_names(folder));
    }
    guide_text(&names)
}

pub(crate) fn draw_files(
    source: &str,
    format: &str,
    name: &str,
    out_dir: &Path,
    picture: bool,
    icons: &[PathBuf],
) -> Result<Drawn, String> {
    let mut diagram = build_diagram(source, format, icons)?;
    let report = diagram.render().map_err(|err| err.to_string())?;
    let described = diagram.describe();
    std::fs::create_dir_all(out_dir).map_err(|err| format!("{}: {err}", out_dir.display()))?;
    let safe = safe_name(name);
    let svg_path = out_dir.join(format!("{safe}.svg"));
    if svg_path.parent() != Some(out_dir) {
        return Err(format!(
            "{}: the file name left the output folder",
            svg_path.display()
        ));
    }
    std::fs::write(&svg_path, report.svg.as_bytes())
        .map_err(|err| format!("{}: {err}", svg_path.display()))?;
    let mut files = vec![svg_path.clone()];
    let png_path = out_dir.join(format!("{safe}.png"));
    let png_error = svg_to_png(&svg_path, &png_path, report.width, report.height, 2.0, None).err();
    if png_error.is_none() {
        files.push(png_path.clone());
    }
    let mut lines = vec![
        problems_text(&report.problems),
        String::new(),
        "Files:".to_owned(),
    ];
    lines.extend(files.iter().map(|path| format!("- {}", path.display())));
    if let Some(err) = &png_error {
        lines.push(format!("PNG skipped: {err}. The SVG is written."));
    }
    // `render` records a size when it drew text. `save` prints "none" when it did not.
    let smallest = report.min_text.map_or_else(|| "none".to_owned(), py_g);
    lines.push(format!(
        "Size: {} x {} px; smallest text {smallest} px",
        py_g(report.width),
        py_g(report.height)
    ));
    lines.push(String::new());
    lines.push("What the picture says:".to_owned());
    lines.push(described);
    let png = if picture && png_error.is_none() {
        Some(std::fs::read(&png_path).map_err(|err| format!("{}: {err}", png_path.display()))?)
    } else {
        None
    };
    Ok(Drawn {
        text: lines.join("\n"),
        png,
    })
}

/// A file name with no folders in it, so every file lands inside the output folder.
pub(crate) fn safe_name(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(ch);
        } else {
            dash = true;
        }
    }
    let sliced: String = out.trim_matches('-').chars().take(80).collect();
    if sliced.is_empty() {
        "diagram".to_owned()
    } else {
        sliced
    }
}

pub(crate) fn absolute_user(path: PathBuf) -> PathBuf {
    let path = expand_user(path);
    std::path::absolute(&path).unwrap_or(path)
}

fn expand_user(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    let Some(rest) = text.strip_prefix('~') else {
        return path;
    };
    if rest.is_empty() {
        return home_dir();
    }
    if rest.starts_with('/') || rest.starts_with('\\') {
        return home_dir().join(&rest[1..]);
    }
    path
}

fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
}

pub(crate) fn default_out() -> PathBuf {
    if let Some(value) = std::env::var_os("OPEN_DIAGRAM_RS_OUT") {
        return absolute_user(PathBuf::from(value));
    }
    if let Ok(cwd) = std::env::current_dir()
        && cwd != Path::new("/")
        && cwd != Path::new("\\")
        && is_writable(&cwd)
    {
        return absolute_user(cwd.join("open-diagram-rs"));
    }
    absolute_user(home_dir().join("open-diagram-rs"))
}

fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(
        ".open-diagram-rs-mcp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos())
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .is_ok_and(|file| {
            drop(file);
            // The new file already showed the directory is writable.
            match std::fs::remove_file(probe) {
                Ok(()) | Err(_) => {}
            }
            true
        })
}

pub(crate) fn icon_folders(cli: &[PathBuf]) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = if cli.is_empty() {
        std::env::var_os("OPEN_DIAGRAM_RS_ICONS").map_or_default(|value| {
            std::env::split_paths(&value)
                .filter(|path| !path.as_os_str().is_empty())
                .map(absolute_user)
                .collect()
        })
    } else {
        cli.iter().cloned().map(absolute_user).collect()
    };
    folders.push(catalog::icon_dir());
    folders
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::guide::GUIDE;

    #[test]
    fn safe_names_have_no_folders() {
        assert_eq!(safe_name(""), "diagram");
        assert_eq!(safe_name("---"), "diagram");
        assert_eq!(safe_name("../escape"), "escape");
        assert_eq!(safe_name("foo/../../bar"), "foo-bar");
        assert_eq!(safe_name("My Diagram"), "My-Diagram");
        assert_eq!(safe_name("a--b"), "a--b");
        assert_eq!(safe_name(&"a".repeat(90)).chars().count(), 80);
        let mut mixed = "a".repeat(79);
        mixed.push('🚀');
        mixed.push('b');
        let safe = safe_name(&mixed);
        assert_eq!(safe.chars().count(), 80);
        assert!(safe.is_ascii());
        assert!(safe.ends_with('-'));
    }

    #[test]
    fn source_over_the_limit_names_the_count() {
        let source = "x".repeat(200_001);
        let Err(err) = build_diagram(&source, "auto", &[]) else {
            panic!("a source over the limit should fail");
        };
        assert_eq!(
            err,
            "the diagram text is 200001 characters; the limit is 200000"
        );
    }

    #[test]
    fn format_must_be_json_mermaid_or_auto() {
        let Err(err) = build_diagram("{}", "picture", &[]) else {
            panic!("an unknown format should fail");
        };
        assert_eq!(err, "format must be 'json', 'mermaid' or 'auto'");
    }

    #[test]
    fn auto_treats_a_brace_as_json() {
        let Err(err) = build_diagram("{}", "auto", &[catalog::icon_dir()]) else {
            panic!("empty json should fail");
        };
        assert!(err.contains("title") || err.contains("diagram"), "{err}");
        let Err(err) = build_diagram("not a flowchart", "auto", &[]) else {
            panic!("mermaid text should not be parsed as json");
        };
        assert!(!err.starts_with("not valid JSON"), "{err}");
    }

    #[test]
    fn missing_template_names_the_templates() {
        let err = template_text("not-a-template").unwrap_err();
        assert!(err.contains("no template named"), "{err}");
        let listed = template_text("").unwrap();
        assert!(listed.contains("- aws_etl:"), "{listed}");
    }

    #[test]
    fn guide_lists_builtin_icons() {
        let text = icons_guide(&[catalog::icon_dir()]);
        assert!(text.contains("Icons available:"), "{text}");
        assert!(text.contains("aws/s3"), "{text}");
        assert!(text.starts_with(GUIDE));
    }

    #[test]
    fn draw_keeps_the_svg_inside_the_output_folder() {
        let out = scratch();
        let drawn = draw_files(
            "flowchart LR\n  A[Start] --> B[End]\n",
            "mermaid",
            "../escape",
            &out,
            false,
            &[],
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let svg = out.join("escape.svg");
        assert!(svg.is_file(), "{}", drawn.text);
        assert!(drawn.text.contains(&format!("- {}", svg.display())));
        assert!(
            drawn.text.contains("What the picture says:"),
            "{}",
            drawn.text
        );
        assert!(drawn.png.is_none());
        let parent = out.parent().expect("parent");
        assert!(!parent.join("escape.svg").exists());
        remove_tree(&out);
    }

    fn scratch() -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "open-diagram-rs-mcp-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("scratch");
        path
    }

    fn remove_tree(dir: &std::path::Path) {
        match std::fs::remove_dir_all(dir) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => panic!("remove {}: {err}", dir.display()),
        }
    }
}
