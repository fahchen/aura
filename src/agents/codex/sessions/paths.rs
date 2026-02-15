use chrono::{Datelike, Local, NaiveDate};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub(super) struct CodexPaths {
    pub(super) home: PathBuf,
    pub(super) sessions_root: PathBuf,
    pub(super) sessions_root_alt: PathBuf,
}

impl CodexPaths {
    pub(super) fn detect() -> Self {
        let home_raw = codex_home();
        let home = std::fs::canonicalize(&home_raw).unwrap_or_else(|_| home_raw.clone());
        let sessions_root = home.join("sessions");
        let sessions_root_alt = home_raw.join("sessions");

        Self {
            home,
            sessions_root,
            sessions_root_alt,
        }
    }
}

fn codex_home() -> PathBuf {
    if let Some(home) = std::env::var_os("CODEX_HOME") {
        return PathBuf::from(home);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
}

pub(super) fn is_jsonl(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jsonl"))
}

pub(super) fn session_id_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Common format: `rollout-<timestamp>-<uuid>.jsonl` (uuid has 5 `-` segments).
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() >= 6 {
        return parts[parts.len() - 5..].join("-");
    }

    stem.to_string()
}

pub(super) fn read_dir_recursive(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        let entries = match std::fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let p = entry.path();
            match entry.file_type() {
                Ok(t) if t.is_dir() => stack.push(p),
                Ok(t) if t.is_file() && is_jsonl(&p) => out.push(p),
                _ => {}
            }
        }
    }

    out
}

fn date_dir(root: &Path, date: NaiveDate) -> PathBuf {
    root.join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{:02}", date.day()))
}

fn max_numeric_child_dir(parent: &Path, len: usize) -> Option<PathBuf> {
    let entries = std::fs::read_dir(parent).ok()?;
    let mut best: Option<(String, PathBuf)> = None;

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.len() != len || !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        match &best {
            Some((best_name, _)) if name <= best_name.as_str() => {}
            _ => best = Some((name.to_string(), entry.path())),
        }
    }

    best.map(|(_, path)| path)
}

fn latest_day_dir(root: &Path) -> Option<PathBuf> {
    let year = max_numeric_child_dir(root, 4)?;
    let month = max_numeric_child_dir(&year, 2)?;
    max_numeric_child_dir(&month, 2)
}

fn candidate_scan_dirs(root: &Path) -> Vec<PathBuf> {
    let today = Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);

    let mut dirs = vec![
        root.to_path_buf(),
        date_dir(root, today),
        date_dir(root, yesterday),
    ];
    if let Some(latest) = latest_day_dir(root) {
        dirs.push(latest);
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

async fn read_dir_jsonl(dir: &Path) -> Vec<PathBuf> {
    let Ok(mut rd) = tokio::fs::read_dir(dir).await else {
        return Vec::new();
    };

    let mut out = Vec::new();
    while let Ok(Some(entry)) = rd.next_entry().await {
        let path = entry.path();
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };
        if file_type.is_file() && is_jsonl(&path) {
            out.push(path);
        }
    }

    out
}

pub(super) async fn modified_within(path: &Path, window: Duration) -> bool {
    let Ok(meta) = tokio::fs::metadata(path).await else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::ZERO);
    age <= window
}

pub(super) async fn scan_recent_rollouts(root: &Path, window: Duration) -> Vec<PathBuf> {
    let mut out = Vec::new();

    for dir in candidate_scan_dirs(root) {
        for path in read_dir_jsonl(&dir).await {
            if modified_within(&path, window).await {
                out.push(path);
            }
        }
    }

    out
}
