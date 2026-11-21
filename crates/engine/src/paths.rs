//! Where things live on this computer.
//!
//! Nothing here is inside the repo. Saved setups, recorded speech and working
//! files sit in the app's own folder, which the app and the command-line tool
//! share, so speech paid for once is reused by both.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Also the app's identifier in `src-tauri/tauri.conf.json`. Keep the two the same.
pub const APP_ID: &str = "gg.cathedral.prayer-video-creator";

#[derive(Clone, Debug)]
pub struct Paths {
    pub data: PathBuf,
}

/// The few things that belong to this computer, not to any one video.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// Where finished videos go. Empty means Videos/Prayer Videos.
    pub output_folder: Option<String>,
    /// Chrome or Edge, if it is not where this tool looks.
    pub browser: Option<String>,
    /// FFmpeg, if it is not where this tool looks.
    pub ffmpeg: Option<String>,
}

impl Paths {
    pub fn new(data: impl Into<PathBuf>) -> Self {
        Paths { data: data.into() }
    }

    /// The usual place: `%APPDATA%\gg.cathedral.prayer-video-creator` on Windows.
    /// `PVC_DATA_DIR` moves it, which the tests use.
    pub fn standard() -> Self {
        let data = std::env::var_os("PVC_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join(APP_ID));
        Paths { data }
    }

    pub fn setups(&self) -> PathBuf {
        self.data.join("setups")
    }
    pub fn pictures(&self) -> PathBuf {
        self.data.join("pictures")
    }
    pub fn recordings(&self) -> PathBuf {
        self.data.join("recordings")
    }
    /// Speech from ElevenLabs, exactly as it arrived. Never deleted by a render.
    pub fn speech(&self) -> PathBuf {
        self.data.join("cache").join("speech")
    }
    /// Speech and recordings after trimming and speed change.
    pub fn clips(&self) -> PathBuf {
        self.data.join("cache").join("clips")
    }
    /// Music from ElevenLabs, exactly as it arrived. Paid for, so never deleted by a render.
    pub fn music(&self) -> PathBuf {
        self.data.join("cache").join("music")
    }
    pub fn voices_file(&self) -> PathBuf {
        self.data.join("cache").join("voices.json")
    }
    /// The ElevenLabs models and what each charges, as last fetched.
    pub fn models_file(&self) -> PathBuf {
        self.data.join("cache").join("models.json")
    }
    /// Stills and scene files for the video being made. Safe to delete.
    pub fn work(&self) -> PathBuf {
        self.data.join("work")
    }
    pub fn settings_file(&self) -> PathBuf {
        self.data.join("settings.json")
    }

    pub fn settings(&self) -> Settings {
        std::fs::read_to_string(self.settings_file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        std::fs::create_dir_all(&self.data)?;
        std::fs::write(self.settings_file(), serde_json::to_string_pretty(settings)?)?;
        Ok(())
    }

    /// Where finished videos go when a setup does not say.
    pub fn output_folder(&self) -> PathBuf {
        if let Some(f) = self.settings().output_folder.filter(|f| !f.trim().is_empty()) {
            return PathBuf::from(f);
        }
        dirs::video_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from(".")).join("Prayer Videos")
    }

    /// Folders searched for a picture or recording named in a setup, in order:
    /// beside the setup file, the folder the tool was started in, then the app's own folder.
    pub fn search(&self, setup_dir: Option<&Path>) -> Vec<PathBuf> {
        let mut v = Vec::new();
        if let Some(d) = setup_dir {
            v.push(d.to_path_buf());
        }
        if let Ok(cwd) = std::env::current_dir() {
            v.push(cwd);
        }
        v.push(self.data.clone());
        v
    }
}

/// A name that is safe to use as a file name on any system.
pub fn file_safe(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | ',' | '\'' | '(' | ')' | '+') {
            out.push(c);
        } else {
            out.push('-');
        }
    }
    let out = out.trim_matches(|c| c == '.' || c == ' ' || c == '-').to_string();
    if out.is_empty() {
        "Prayer video".into()
    } else {
        out
    }
}

/// A short id made from a name: "My Joyful (slow)" becomes "my-joyful-slow".
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "setup".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_become_safe_files_and_ids() {
        assert_eq!(file_safe("Luminous: the first video?"), "Luminous- the first video");
        assert_eq!(file_safe("  ...  "), "Prayer video");
        assert_eq!(
            file_safe("The Holy Rosary - Joyful Mysteries (Ann + Ben, Eleven Flash v2.5, 1920x1080)"),
            "The Holy Rosary - Joyful Mysteries (Ann + Ben, Eleven Flash v2.5, 1920x1080)",
            "a title the tool writes is a file name as it stands"
        );
        assert_eq!(slug("My Joyful (slow)"), "my-joyful-slow");
        assert_eq!(slug("¿?"), "setup");
    }
}
