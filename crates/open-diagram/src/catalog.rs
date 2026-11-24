use std::path::{Path, PathBuf};

use crate::num::py_repr;

include!(concat!(env!("OUT_DIR"), "/template_assets.rs"));

const TEMPLATE_TYPES: &[&str] = &[".json", ".mmd"];
const ICON_TYPES: &[&str] = &[".svg", ".png", ".jpg", ".jpeg"];

#[must_use]
pub fn template_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

#[must_use]
pub fn icon_dir() -> PathBuf {
    template_dir().join("assets")
}

/// Bytes embedded from `templates/` at compile time. `rel` uses forward slashes, such as `assets/aws/s3.svg`.
#[must_use]
pub fn embedded(rel: &str) -> Option<&'static [u8]> {
    ASSETS
        .iter()
        .find(|entry| entry.0 == rel)
        .map(|entry| entry.1)
}

/// Embedded icon bytes when `path` is under [`icon_dir`] and the file is one of the compiled assets.
#[must_use]
pub fn embedded_icon(path: &str) -> Option<&'static [u8]> {
    let rel = Path::new(path).strip_prefix(icon_dir()).ok()?;
    let rel = rel.to_string_lossy().replace('\\', "/");
    if rel.is_empty()
        || rel
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    embedded(&format!("assets/{rel}"))
}

/// # Errors
/// Returns an error when the template table cannot be read.
pub fn table(readme: Option<&Path>) -> Result<Vec<(String, String)>, String> {
    let text = readme_text(readme)?;
    let mut rows = Vec::new();
    let mut header: Option<Vec<String>> = None;
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            if header.is_some() && !rows.is_empty() {
                break;
            }
            continue;
        }
        let cells: Vec<String> = line
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_owned())
            .collect();
        if header.is_none() {
            if cells
                .first()
                .is_some_and(|cell| cell.eq_ignore_ascii_case("template"))
            {
                header = Some(cells.iter().map(|cell| cell.to_ascii_lowercase()).collect());
            }
            continue;
        }
        if line.chars().all(|ch| matches!(ch, '|' | '-' | ':' | ' ')) {
            continue;
        }
        let heads = header.as_ref().map_or(&[][..], Vec::as_slice);
        let name = link_name(cells.first().map_or("", String::as_str));
        let purpose = heads
            .iter()
            .position(|cell| cell == "purpose")
            .and_then(|index| cells.get(index))
            .cloned()
            .unwrap_or_default();
        rows.push((name, purpose));
    }
    Ok(rows)
}

fn readme_text(readme: Option<&Path>) -> Result<String, String> {
    if let Some(path) = readme {
        return std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()));
    }
    let path = template_dir().join("README.md");
    let bytes = if path.is_file() {
        std::fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?
    } else {
        embedded("README.md")
            .map(<[u8]>::to_vec)
            .ok_or_else(|| "missing template README.md".to_owned())?
    };
    String::from_utf8(bytes).map_err(|err| format!("README.md: {err}"))
}

fn link_name(cell: &str) -> String {
    let Some(start) = cell.find("](") else {
        return cell.trim_matches('`').to_owned();
    };
    let rest = &cell[start + 2..];
    let end = rest.find(')').unwrap_or(rest.len());
    Path::new(&rest[..end]).file_name().map_or_else(
        || rest[..end].to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// # Errors
/// Returns an error when the template table cannot be read.
pub fn names() -> Result<Vec<String>, String> {
    Ok(table(None)?
        .into_iter()
        .filter_map(|(file, _)| {
            let path = Path::new(&file);
            let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            let dotted = format!(".{ext}");
            TEMPLATE_TYPES.contains(&dotted.as_str()).then(|| {
                path.file_stem()
                    .map_or_else(|| file.clone(), |stem| stem.to_string_lossy().into_owned())
            })
        })
        .collect())
}

fn template_rel(name: &str) -> Result<String, String> {
    let base = Path::new(name)
        .file_name()
        .and_then(|file| file.to_str())
        .unwrap_or("");
    let stem = Path::new(base)
        .file_stem()
        .and_then(|file| file.to_str())
        .unwrap_or("");
    let known = names()?;
    for ext in TEMPLATE_TYPES {
        let rel = format!("{stem}{ext}");
        let listed = known.iter().any(|item| item == stem);
        let present = template_dir().join(&rel).is_file() || embedded(&rel).is_some();
        if listed && present {
            return Ok(rel);
        }
    }
    Err(format!(
        "no template named {}; templates: {}",
        py_repr(name),
        known.join(", ")
    ))
}

/// # Errors
/// Returns an error naming the templates when `name` is not one of them.
pub fn path(name: &str) -> Result<PathBuf, String> {
    Ok(template_dir().join(template_rel(name)?))
}

/// # Errors
/// Returns an error when the template cannot be read.
pub fn read(name: &str) -> Result<String, String> {
    let rel = template_rel(name)?;
    let path = template_dir().join(&rel);
    let bytes = if path.is_file() {
        std::fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?
    } else {
        embedded(&rel)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| format!("missing template asset {rel}"))?
    };
    String::from_utf8(bytes).map_err(|err| format!("{rel}: {err}"))
}

#[must_use]
pub fn icon_names(folder: &Path) -> Vec<String> {
    if folder.is_dir() {
        return scan_icon_dir(folder);
    }
    if folder == icon_dir() {
        return embedded_icon_names();
    }
    Vec::new()
}

fn scan_icon_dir(folder: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![folder.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut children = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                children.push(path);
                continue;
            }
            let Some(name) = icon_name(folder, &path) else {
                continue;
            };
            found.push(name);
        }
        children.sort();
        stack.extend(children.into_iter().rev());
    }
    found.sort();
    found
}

fn icon_name(folder: &Path, path: &Path) -> Option<String> {
    let ext = path.extension().and_then(|ext| ext.to_str())?;
    if !ICON_TYPES.contains(&format!(".{ext}").to_ascii_lowercase().as_str()) {
        return None;
    }
    let stem = path.file_stem()?;
    let parent = path.parent().unwrap_or(folder);
    let rel = parent.strip_prefix(folder).unwrap_or(parent);
    let mut name = rel.to_string_lossy().replace('\\', "/");
    if !name.is_empty() {
        name.push('/');
    }
    name.push_str(&stem.to_string_lossy());
    Some(name)
}

fn embedded_icon_names() -> Vec<String> {
    let mut found = Vec::new();
    for &(rel, _) in ASSETS {
        if !rel.starts_with("assets/") {
            continue;
        }
        let Some(name) = icon_name(Path::new("assets"), Path::new(rel)) else {
            continue;
        };
        found.push(name);
    }
    found.sort();
    found
}

#[must_use]
pub fn find_icon(name: &str, folders: &[PathBuf]) -> Option<PathBuf> {
    let clean = name.replace('\\', "/");
    let parts: Vec<&str> = clean.split('/').collect();
    if clean.is_empty()
        || clean.starts_with('/')
        || clean.contains(':')
        || parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return None;
    }
    let ext = Path::new(&clean)
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or(String::new(), |ext| format!(".{ext}").to_ascii_lowercase());
    let tries: Vec<String> = if ICON_TYPES.contains(&ext.as_str()) {
        vec![clean]
    } else {
        ICON_TYPES
            .iter()
            .map(|suffix| format!("{clean}{suffix}"))
            .collect()
    };
    for folder in folders {
        if let Some(found) = find_icon_on_disk(folder, &tries) {
            return Some(found);
        }
        if folder == &icon_dir()
            && let Some(found) = find_embedded_icon(&tries)
        {
            return Some(found);
        }
    }
    None
}

fn find_icon_on_disk(folder: &Path, tries: &[String]) -> Option<PathBuf> {
    let root = folder.canonicalize().ok()?;
    for attempt in tries {
        let joined = attempt
            .split('/')
            .fold(root.clone(), |path, part| path.join(part));
        let Ok(real) = joined.canonicalize() else {
            continue;
        };
        if real.is_file() && real.starts_with(&root) {
            return Some(real);
        }
    }
    None
}

fn find_embedded_icon(tries: &[String]) -> Option<PathBuf> {
    for attempt in tries {
        if embedded(&format!("assets/{attempt}")).is_some() {
            return Some(icon_dir().join(attempt));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{embedded, embedded_icon, find_embedded_icon, find_icon, icon_dir};

    #[test]
    fn embedded_icons_match_the_files_in_the_crate() {
        let svg = icon_dir().join("aws/s3.svg");
        assert_eq!(
            embedded_icon(&svg.to_string_lossy()).expect("s3 bytes"),
            std::fs::read(&svg).expect("s3 file")
        );
        let quicksight = find_embedded_icon(&[
            "aws/quicksight.png".to_owned(),
            "aws/quicksight.svg".to_owned(),
        ])
        .expect("quicksight");
        assert_eq!(
            embedded_icon(&quicksight.to_string_lossy()).expect("quicksight bytes"),
            std::fs::read(&quicksight).expect("quicksight file")
        );
        assert!(embedded("aws_etl.json").expect("etl").starts_with(b"{"));
        assert!(find_icon("aws/s3", &[icon_dir()]).expect("s3").is_file());
        assert!(find_icon("aws/s3", &[std::env::temp_dir()]).is_none());
        assert!(embedded_icon(r"C:\outside\aws\s3.svg").is_none());
    }
}
