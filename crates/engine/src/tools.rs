//! The two outside programs this tool leans on: FFmpeg, which makes the sound
//! and the video, and Chrome or Edge, which draws the stills.

use crate::error::{Error, Result};
use crate::paths::Settings;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Start a program without flashing a black console window on Windows.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.stdin(Stdio::null());
    cmd
}

/// Run a program to the end. On failure, the error carries the end of what it said.
pub fn run(cmd: &mut Command, what: &str) -> Result<Output> {
    let out = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output().map_err(|e| Error::tool(format!("Could not start {what}: {e}")))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let tail: String = err.lines().rev().take(6).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
        return Err(Error::tool(format!("{what} failed.\n{tail}")));
    }
    Ok(out)
}

fn works(program: &Path, arg: &str) -> bool {
    command(program).arg(arg).stdout(Stdio::null()).stderr(Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
}

pub fn find_ffmpeg(settings: &Settings) -> Result<PathBuf> {
    let mut tries: Vec<PathBuf> = Vec::new();
    if let Some(p) = settings.ffmpeg.as_deref().filter(|p| !p.trim().is_empty()) {
        tries.push(PathBuf::from(p));
    }
    if let Some(p) = std::env::var_os("PVC_FFMPEG") {
        tries.push(PathBuf::from(p));
    }
    tries.push(PathBuf::from("ffmpeg"));
    if cfg!(windows) {
        tries.push(PathBuf::from(r"C:\ffmpeg\ffmpeg.exe"));
        tries.push(PathBuf::from(r"C:\ffmpeg\bin\ffmpeg.exe"));
        tries.push(PathBuf::from(r"C:\Program Files\ffmpeg\bin\ffmpeg.exe"));
    } else {
        tries.push(PathBuf::from("/opt/homebrew/bin/ffmpeg"));
        tries.push(PathBuf::from("/usr/local/bin/ffmpeg"));
    }
    tries.into_iter().find(|p| works(p, "-version")).ok_or_else(|| {
        Error::tool(
            "FFmpeg was not found. It is the free program that makes the sound and the video. \
             Install it (on Windows: winget install Gyan.FFmpeg), then try again. \
             If it is installed somewhere unusual, set its path in Settings.",
        )
    })
}

pub fn find_browser(settings: &Settings) -> Result<PathBuf> {
    let mut tries: Vec<PathBuf> = Vec::new();
    if let Some(p) = settings.browser.as_deref().filter(|p| !p.trim().is_empty()) {
        tries.push(PathBuf::from(p));
    }
    if let Some(p) = std::env::var_os("PVC_BROWSER") {
        tries.push(PathBuf::from(p));
    }
    if cfg!(windows) {
        for base in ["ProgramFiles", "ProgramFiles(x86)", "LocalAppData"] {
            if let Some(dir) = std::env::var_os(base) {
                let dir = PathBuf::from(dir);
                tries.push(dir.join(r"Google\Chrome\Application\chrome.exe"));
                tries.push(dir.join(r"Microsoft\Edge\Application\msedge.exe"));
            }
        }
    } else if cfg!(target_os = "macos") {
        tries.push(PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"));
        tries.push(PathBuf::from("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"));
        tries.push(PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"));
    } else {
        for name in ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "microsoft-edge"] {
            for dir in ["/usr/bin", "/usr/local/bin", "/snap/bin"] {
                tries.push(PathBuf::from(dir).join(name));
            }
        }
    }
    tries.into_iter().find(|p| p.is_file()).ok_or_else(|| {
        Error::tool(
            "Chrome or Edge was not found. One of them is needed to draw the pictures of each bead. \
             Install Google Chrome, or set the path to your browser in Settings.",
        )
    })
}

/// A file path as a web address a browser will open.
pub fn file_url(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file://");
    if !s.starts_with('/') {
        out.push('/');
    }
    for c in s.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '#' => out.push_str("%23"),
            '%' => out.push_str("%25"),
            '?' => out.push_str("%3F"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_become_file_addresses() {
        assert_eq!(file_url(Path::new(r"C:\My Pictures\a#1.png")), "file:///C:/My%20Pictures/a%231.png");
        assert_eq!(file_url(Path::new("/home/ben/a.png")), "file:///home/ben/a.png");
    }
}
