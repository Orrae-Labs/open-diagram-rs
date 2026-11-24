use std::fmt::Write;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let templates = manifest.join("templates");
    println!("cargo:rerun-if-changed=templates");
    let mut files = Vec::new();
    walk(&templates, &templates, &mut files);
    files.sort();
    let mut source = String::from("pub static ASSETS: &[(&str, &[u8])] = &[\n");
    for (rel, abs) in files {
        let abs = abs.to_string_lossy().replace('\\', "/");
        writeln!(source, "    ({rel:?}, include_bytes!({abs:?})),").expect("format asset line");
        println!("cargo:rerun-if-changed={}", templates.join(&rel).display());
    }
    source.push_str("];\n");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("template_assets.rs");
    std::fs::write(out, source).expect("write template assets");
}

fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("{err}"));
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, files);
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        files.push((rel, path));
    }
}
