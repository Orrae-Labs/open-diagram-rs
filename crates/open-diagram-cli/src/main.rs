use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use indexmap::IndexMap;
use open_diagram::Diagram;

const SOUNDTRACK: &str =
    "The Gathering Field: https://open.spotify.com/artist/0BfQ8COB7K647uB6a0kMZn";

#[derive(Parser)]
#[command(name = "open-diagram", version = env!("CARGO_PKG_VERSION"), about = "Draw and check diagrams that check their own layout.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write FILE's diagram as SVG and PNG; exit 1 if the layout has problems.
    Draw {
        file: String,
        /// Output path without extension (default: next to FILE).
        #[arg(short, long)]
        out: Option<String>,
        /// Write the SVG only.
        #[arg(long)]
        no_png: bool,
        /// PNG scale (default 2).
        #[arg(long, default_value_t = 2.0)]
        scale: f64,
        /// JSON file of name -> state, applied before drawing.
        #[arg(long)]
        statuses: Option<String>,
        /// Title to use (Mermaid files default to their front-matter title).
        #[arg(long)]
        title: Option<String>,
        /// Another folder of icons for JSON icon names.
        #[arg(long, action = clap::ArgAction::Append)]
        icons: Vec<String>,
    },
    /// Build each FILE in memory and list layout problems; exit 1 if any.
    Check {
        files: Vec<String>,
        /// JSON file of name -> state, applied before checking.
        #[arg(long)]
        statuses: Option<String>,
        /// Another folder of icons for JSON icon names.
        #[arg(long, action = clap::ArgAction::Append)]
        icons: Vec<String>,
    },
    /// Print a plain-text description of FILE's diagram.
    Describe {
        file: String,
        /// JSON file of name -> state, applied first.
        #[arg(long)]
        statuses: Option<String>,
        #[arg(long)]
        title: Option<String>,
        /// Another folder of icons for JSON icon names.
        #[arg(long, action = clap::ArgAction::Append)]
        icons: Vec<String>,
    },
    /// Convert an SVG to PNG with Chrome, Chromium or Edge.
    Png {
        svg: String,
        #[arg(long, default_value_t = 2.0)]
        scale: f64,
    },
    /// List the templates, or write one to start from.
    New {
        /// Template name; leave out to list them.
        template: Option<String>,
        /// File to write (default: the template's name, here).
        #[arg(short, long)]
        out: Option<String>,
        /// Replace the file if it already exists.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--soundtrack") {
        println!("{SOUNDTRACK}");
        return ExitCode::SUCCESS;
    }
    if args
        .first()
        .is_some_and(|arg| arg.to_ascii_lowercase().ends_with(".svg"))
    {
        args.insert(0, "png".to_owned());
    }
    let mut full = vec!["open-diagram".to_owned()];
    full.append(&mut args);
    match execute(&full) {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn execute(args: &[String]) -> Result<u8, String> {
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Png { svg, scale } => png(&svg, scale),
        Commands::New {
            template,
            out,
            force,
        } => fresh(template.as_deref(), out.as_deref(), force),
        Commands::Describe {
            file,
            statuses,
            title,
            icons,
        } => {
            let diagram = load(&file, statuses.as_deref(), title.as_deref(), &icons)?;
            println!("{}", diagram.describe());
            Ok(0)
        }
        Commands::Draw {
            file,
            out,
            no_png,
            scale,
            statuses,
            title,
            icons,
        } => draw(
            &file,
            out.as_deref(),
            no_png,
            scale,
            statuses.as_deref(),
            title.as_deref(),
            &icons,
        ),
        Commands::Check {
            files,
            statuses,
            icons,
        } => check(&files, statuses.as_deref(), &icons),
    }
}

fn draw(
    file: &str,
    out: Option<&str>,
    no_png: bool,
    scale: f64,
    statuses: Option<&str>,
    title: Option<&str>,
    icons: &[String],
) -> Result<u8, String> {
    check_scale(scale)?;
    let mut diagram = load(file, statuses, title, icons)?;
    let base = out.map_or_else(|| split_stem(file).to_owned(), str::to_owned);
    let report = diagram
        .save(&base, !no_png, scale, false)
        .map_err(|err| err.to_string())?;
    for problem in &report.problems {
        println!("  - {problem}");
    }
    Ok(u8::from(!report.problems.is_empty()))
}

fn check(files: &[String], statuses: Option<&str>, icons: &[String]) -> Result<u8, String> {
    let mut failed = false;
    for path in files {
        let mut diagram = load(path, statuses, None, icons)?;
        let report = diagram.render().map_err(|err| err.to_string())?;
        println!("{path}: {} layout problem(s)", report.problems.len());
        for problem in &report.problems {
            println!("  - {problem}");
        }
        failed |= !report.problems.is_empty();
    }
    Ok(u8::from(failed))
}

fn png(svg: &str, scale: f64) -> Result<u8, String> {
    check_scale(scale)?;
    let text = std::fs::read_to_string(svg).map_err(|err| format!("{svg}: {err}"))?;
    let head: String = text.chars().take(2000).collect();
    let Some(width) = attr_number(&head, "width") else {
        eprintln!("{svg}: the SVG needs width and height attributes");
        return Ok(2);
    };
    let Some(height) = attr_number(&head, "height") else {
        eprintln!("{svg}: the SVG needs width and height attributes");
        return Ok(2);
    };
    let out = if svg.len() >= 4
        && svg
            .get(svg.len() - 4..)
            .is_some_and(|tail| tail.eq_ignore_ascii_case(".svg"))
    {
        format!("{}.png", &svg[..svg.len() - 4])
    } else {
        format!("{svg}.png")
    };
    if let Err(err) =
        open_diagram::svg_to_png(Path::new(svg), Path::new(&out), width, height, scale, None)
    {
        eprintln!("PNG not written: {err}");
        Ok(2)
    } else {
        println!("wrote {out}");
        Ok(0)
    }
}

fn fresh(template: Option<&str>, out: Option<&str>, force: bool) -> Result<u8, String> {
    let Some(template) = template else {
        let rows = open_diagram::catalog::table(None)?;
        let stems: Vec<(String, String)> = rows
            .into_iter()
            .map(|(file, purpose)| (stem_of(&file), purpose))
            .collect();
        let width = stems.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        for (name, purpose) in &stems {
            println!("{name:<width$}  {purpose}");
        }
        println!("\nopen-diagram new NAME writes NAME.json here to start from.");
        return Ok(0);
    };
    let source = open_diagram::catalog::path(template)?;
    let target = out.map_or_else(
        || {
            source.file_name().map_or_else(
                || format!("{template}.json"),
                |name| name.to_string_lossy().into_owned(),
            )
        },
        str::to_owned,
    );
    if Path::new(&target).exists() && !force {
        eprintln!(
            "error: {target} already exists; choose another name with -o, or add --force to replace it"
        );
        return Ok(2);
    }
    let bytes = open_diagram::catalog::read(template)?;
    if let Some(parent) = Path::new(&target).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|err| format!("{target}: {err}"))?;
    }
    std::fs::write(&target, bytes).map_err(|err| format!("{target}: {err}"))?;
    println!("wrote {target}; change its labels, then: open-diagram check {target}");
    Ok(0)
}

fn load(
    path: &str,
    statuses: Option<&str>,
    title: Option<&str>,
    icons: &[String],
) -> Result<Diagram, String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let extra: Vec<PathBuf> = icons.iter().map(PathBuf::from).collect();
    let mut diagram = match ext.as_str() {
        "json" => open_diagram::load_spec_with(path, &extra).map_err(|err| err.to_string())?,
        "mmd" | "mermaid" => {
            open_diagram::load_mermaid(path, title).map_err(|err| err.to_string())?
        }
        "py" => {
            return Err(format!(
                "{path}: Python scripts are not executed; use .json, .mmd or .mermaid"
            ));
        }
        _ => {
            return Err(format!(
                "{path}: unknown file type; use .json, .mmd or .mermaid"
            ));
        }
    };
    if let Some(title) = title
        && ext != "mmd"
        && ext != "mermaid"
    {
        title.clone_into(&mut diagram.title);
    }
    if let Some(statuses) = statuses {
        diagram
            .set_statuses(&read_statuses(statuses)?)
            .map_err(|err| err.to_string())?;
    }
    Ok(diagram)
}

fn read_statuses(path: &str) -> Result<IndexMap<String, String>, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("{path}: {err}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| format!("{path}: {err}"))?;
    let map = value
        .as_object()
        .ok_or_else(|| format!("{path}: expected a JSON object"))?;
    let mut statuses = IndexMap::new();
    for (name, state) in map {
        let state = state
            .as_str()
            .ok_or_else(|| format!("{path}: state for '{name}' must be a string"))?;
        statuses.insert(name.clone(), state.to_owned());
    }
    Ok(statuses)
}

fn stem_of(file: &str) -> String {
    Path::new(file).file_stem().map_or_else(
        || file.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

fn split_stem(path: &str) -> &str {
    let name_at = path.rfind(['/', '\\']).map_or(0, |index| index + 1);
    let Some(dot) = path[name_at..].rfind('.') else {
        return path;
    };
    if dot == 0 {
        path
    } else {
        &path[..name_at + dot]
    }
}

fn attr_number(head: &str, name: &str) -> Option<f64> {
    let key = format!("{name}=\"");
    let start = head.find(&key)? + key.len();
    let end = head[start..].find('"')? + start;
    head[start..end].parse().ok()
}

fn check_scale(scale: f64) -> Result<(), String> {
    if scale.is_finite() && scale > 0.0 {
        Ok(())
    } else {
        Err("scale must be a positive number".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{SOUNDTRACK, execute};

    static DIR_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn soundtrack_names_the_gathering_field() {
        assert!(SOUNDTRACK.contains("The Gathering Field"));
        assert!(SOUNDTRACK.contains("open.spotify.com"));
    }

    #[test]
    fn new_writes_a_template_and_refuses_to_replace_it() {
        let _guard = DIR_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let dir = std::env::temp_dir().join(format!("open-diagram-new-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("{err}"));
        let previous = std::env::current_dir().unwrap_or_else(|err| panic!("{err}"));
        std::env::set_current_dir(&dir).unwrap_or_else(|err| panic!("{err}"));
        let listed = execute(&["open-diagram".to_owned(), "new".to_owned()]);
        let written = execute(&[
            "open-diagram".to_owned(),
            "new".to_owned(),
            "azure_web_app".to_owned(),
        ]);
        let again = execute(&[
            "open-diagram".to_owned(),
            "new".to_owned(),
            "azure_web_app".to_owned(),
        ]);
        let checked = execute(&[
            "open-diagram".to_owned(),
            "check".to_owned(),
            "azure_web_app.json".to_owned(),
        ]);
        std::env::set_current_dir(&previous).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(listed.unwrap_or_else(|err| panic!("{err}")), 0);
        assert_eq!(written.unwrap_or_else(|err| panic!("{err}")), 0);
        assert_eq!(again.unwrap_or_else(|err| panic!("{err}")), 2);
        assert_eq!(checked.unwrap_or_else(|err| panic!("{err}")), 0);
        assert!(dir.join("azure_web_app.json").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
