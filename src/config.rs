//! Persistence for user preferences (config) and runtime state.
//!
//! - **Config** (`config.json`): theme preference, saved to the platform config directory.
//! - **State** (`state.json`): indicator position, saved to the platform data directory.
//!
//! On macOS both resolve to `~/Library/Application Support/aura/`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// User preferences (persisted to config.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Theme name: "system", "liquid-dark", "liquid-light", "solid-dark", "solid-light"
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme(),
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Runtime state (persisted to state.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Indicator X position (logical pixels from left edge).
    #[serde(default)]
    pub indicator_x: Option<f64>,
    /// Indicator Y position (logical pixels from top edge).
    #[serde(default)]
    pub indicator_y: Option<f64>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            indicator_x: None,
            indicator_y: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Directory helpers
// ---------------------------------------------------------------------------

/// Aura config directory (e.g. `~/Library/Application Support/aura/`).
fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("aura"))
}

/// Aura data directory (e.g. `~/Library/Application Support/aura/`).
fn data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("aura"))
}

// ---------------------------------------------------------------------------
// Public API — Config
// ---------------------------------------------------------------------------

/// Load config from disk, returning defaults if the file is missing or invalid.
pub fn load_config() -> Config {
    let Some(path) = config_dir().map(|d| d.join("config.json")) else {
        return Config::default();
    };
    load_config_from(&path)
}

/// Save config to disk.
pub fn save_config(config: &Config) -> Result<(), std::io::Error> {
    let dir = config_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "config dir not found")
    })?;
    save_config_to(config, &dir.join("config.json"))
}

// ---------------------------------------------------------------------------
// Public API — State
// ---------------------------------------------------------------------------

/// Load state from disk, returning defaults if the file is missing or invalid.
pub fn load_state() -> State {
    let Some(path) = data_dir().map(|d| d.join("state.json")) else {
        return State::default();
    };
    load_state_from(&path)
}

/// Save state to disk.
pub fn save_state(state: &State) -> Result<(), std::io::Error> {
    let dir = data_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "data dir not found")
    })?;
    save_state_to(state, &dir.join("state.json"))
}

// ---------------------------------------------------------------------------
// Path-parameterised helpers (used by public API and tests)
// ---------------------------------------------------------------------------

fn load_config_from(path: &Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

fn save_config_to(config: &Config, path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    atomic_write(path, json.as_bytes())
}

fn load_state_from(path: &Path) -> State {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => State::default(),
    }
}

fn save_state_to(state: &State, path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    atomic_write(path, json.as_bytes())
}

/// Write bytes to a file atomically: write to a temp file in the same
/// directory, then rename over the target. Prevents partial JSON on crash.
fn atomic_write(path: &Path, data: &[u8]) -> Result<(), std::io::Error> {
    use std::io::Write;

    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no parent")
    })?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(data)?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn config_default_values() {
        let config = Config::default();
        assert_eq!(config.theme, "system");
    }

    #[test]
    fn state_default_values() {
        let state = State::default();
        assert!(state.indicator_x.is_none());
        assert!(state.indicator_y.is_none());
    }

    #[test]
    fn config_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("aura_test_config");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");

        let config = Config {
            theme: "liquid-dark".to_string(),
        };
        save_config_to(&config, &path).unwrap();
        let loaded = load_config_from(&path);
        assert_eq!(loaded.theme, "liquid-dark");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("aura_test_state");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");

        let state = State {
            indicator_x: Some(100.0),
            indicator_y: Some(200.0),
        };
        save_state_to(&state, &path).unwrap();
        let loaded = load_state_from(&path);
        assert_eq!(loaded.indicator_x, Some(100.0));
        assert_eq!(loaded.indicator_y, Some(200.0));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let path = Path::new("/tmp/aura_nonexistent/config.json");
        let config = load_config_from(path);
        assert_eq!(config.theme, "system");
    }

    #[test]
    fn load_invalid_json_returns_default() {
        let dir = std::env::temp_dir().join("aura_test_invalid");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");

        fs::write(&path, "not valid json!!!").unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.theme, "system");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_extra_fields_ignored() {
        let dir = std::env::temp_dir().join("aura_test_extra");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");

        fs::write(&path, r#"{"theme":"solid-dark","unknown_field":42}"#).unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.theme, "solid-dark");

        let _ = fs::remove_dir_all(&dir);
    }
}
