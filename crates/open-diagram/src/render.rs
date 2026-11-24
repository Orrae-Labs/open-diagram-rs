use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::num::py_g;

const BROWSERS: &[&str] = &[
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
];
const NAMES: &[&str] = &[
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
    "chrome",
    "microsoft-edge",
];

#[must_use]
pub fn find_chrome() -> Option<PathBuf> {
    if let Some(env) = std::env::var_os("CHROME") {
        let path = PathBuf::from(env);
        if path.is_file() {
            return Some(path);
        }
    }
    for path in BROWSERS {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    for name in NAMES {
        if let Ok(path) = which::which(name) {
            return Some(path);
        }
    }
    None
}

struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Option<Self> {
        let path = std::env::temp_dir().join(format!(
            "open-diagram-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        ));
        std::fs::create_dir_all(&path).ok()?;
        Some(Self { path })
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) | Err(_) => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngError {
    NoBrowser,
    Failed,
}

impl std::fmt::Display for PngError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoBrowser => {
                "Chrome or Chromium not found (set the CHROME environment variable to its path)"
            }
            Self::Failed => "the browser failed to write a PNG",
        })
    }
}

impl std::error::Error for PngError {}

/// # Errors
/// Returns [`PngError::NoBrowser`] when no Chrome, Chromium, or Edge can be found.
/// Returns [`PngError::Failed`] when the browser exits unsuccessfully or writes no PNG.
/// A PNG that was already at `png_path` is left unchanged on failure.
pub fn svg_to_png(
    svg_path: &Path,
    png_path: &Path,
    width: f64,
    height: f64,
    scale: f64,
    chrome: Option<&Path>,
) -> Result<(), PngError> {
    let Some(chrome) = chrome.map(Path::to_path_buf).or_else(find_chrome) else {
        return Err(PngError::NoBrowser);
    };
    let Some(w) = whole_ceil(width) else {
        return Err(PngError::Failed);
    };
    let Some(h) = whole_ceil(height) else {
        return Err(PngError::Failed);
    };
    let Ok(png_abs) = std::path::absolute(png_path) else {
        return Err(PngError::Failed);
    };
    let Ok(svg) = std::fs::read_to_string(svg_path) else {
        return Err(PngError::Failed);
    };
    let Some(scratch) = Scratch::new() else {
        return Err(PngError::Failed);
    };
    let page = scratch.path.join("page.html");
    let html = format!(
        "<!doctype html><html><head><meta charset='utf-8'><style>html,body{{margin:0;padding:0;background:#fff;overflow:hidden}}svg{{display:block}}</style></head><body>{svg}</body></html>"
    );
    if std::fs::write(&page, html).is_err() {
        return Err(PngError::Failed);
    }
    let profile = scratch.path.join("profile");
    let mut command = Command::new(&chrome);
    command
        .args([
            "--headless=new",
            "--disable-gpu",
            "--hide-scrollbars",
            "--no-first-run",
            "--no-default-browser-check",
        ])
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg(format!("--force-device-scale-factor={}", py_g(scale)))
        .arg(format!("--window-size={w},{h}"))
        .arg(format!("--screenshot={}", png_abs.display()))
        .arg(file_uri(&page))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let prior = std::fs::read(&png_abs).ok();
    let produced = run_bounded(&mut command, Duration::from_secs(180)) && png_abs.is_file();
    if !produced {
        restore_png(&png_abs, prior);
        return Err(PngError::Failed);
    }
    crop_png(&png_abs, w, h, scale);
    Ok(())
}

fn restore_png(path: &Path, prior: Option<Vec<u8>>) {
    match prior {
        Some(bytes) => match std::fs::write(path, bytes) {
            Ok(()) | Err(_) => {}
        },
        None => match std::fs::remove_file(path) {
            Ok(()) | Err(_) => {}
        },
    }
}

fn run_bounded(command: &mut Command, limit: Duration) -> bool {
    let Ok(mut child) = command.spawn() else {
        return false;
    };
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if started.elapsed() > limit => {
                match child.kill() {
                    Ok(()) | Err(_) => {}
                }
                match child.wait() {
                    Ok(_) | Err(_) => {}
                }
                return false;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => return false,
        }
    }
}

fn whole_ceil(value: f64) -> Option<u32> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    let text = format!("{:.0}", value.ceil());
    text.parse().ok()
}

fn file_uri(path: &Path) -> String {
    let absolute = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = absolute.to_string_lossy().replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(&text);
    let (prefix, rest) = drive_split(text);
    let mut uri = String::from("file:///");
    uri.push_str(prefix);
    for byte in rest.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            uri.push(char::from(byte));
        } else {
            match write!(uri, "%{byte:02X}") {
                Ok(()) | Err(_) => {}
            }
        }
    }
    uri
}

fn drive_split(text: &str) -> (&str, &str) {
    let bytes = text.as_bytes();
    if bytes.len() >= 2
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes.get(1).is_some_and(|byte| *byte == b':')
    {
        text.split_at(2)
    } else {
        ("", text)
    }
}

fn crop_png(path: &Path, width: u32, height: u32, scale: f64) {
    let target_w = u32::try_from(crate::num::py_round(f64::from(width) * scale)).unwrap_or(width);
    let target_h = u32::try_from(crate::num::py_round(f64::from(height) * scale)).unwrap_or(height);
    let Ok(image) = image::open(path) else {
        return;
    };
    if image.width() == target_w && image.height() == target_h {
        return;
    }
    if image.width() >= target_w && image.height() >= target_h {
        match image.crop_imm(0, 0, target_w, target_h).save(path) {
            Ok(()) | Err(_) => {}
        }
    }
}
