//! A very small remote control for Chrome or Edge.
//!
//! The browser runs with no window. We load one page, ask it to draw a scene,
//! and take a picture of it. That is all this needs to do, so it speaks the
//! browser's own protocol directly and leans on no browser library.

use crate::error::{Error, Result};
use crate::tools;
use base64::Engine as _;
use serde_json::{json, Value};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

pub struct Browser {
    child: Option<Child>,
    profile: Option<PathBuf>,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    session: Option<String>,
    next_id: u64,
}

fn connect(url: &str) -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    // A still of a big screen is several megabytes in one message. Lift the usual limits.
    let config = WebSocketConfig::default().max_message_size(None).max_frame_size(None);
    let (socket, _) = tungstenite::client::connect_with_config(url, Some(config), 0).map_err(|e| Error::tool(format!("Could not talk to the browser: {e}")))?;
    if let MaybeTlsStream::Plain(stream) = socket.get_ref() {
        stream.set_read_timeout(Some(Duration::from_secs(90))).ok();
    }
    Ok(socket)
}

impl Browser {
    /// Start a browser with no window and a throwaway profile inside `work`.
    pub fn launch(exe: &Path, work: &Path) -> Result<Self> {
        let profile = work.join(format!("browser-{}-{}", std::process::id(), Instant::now().elapsed().as_nanos() ^ rand_bits()));
        std::fs::create_dir_all(&profile)?;
        let port_file = profile.join("DevToolsActivePort");
        let mut child = tools::command(exe)
            .arg("--headless=new")
            .arg("--remote-debugging-port=0")
            .arg(format!("--user-data-dir={}", profile.display()))
            .args([
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-extensions",
                "--disable-sync",
                "--disable-background-networking",
                "--hide-scrollbars",
                "--mute-audio",
                "--force-color-profile=srgb",
                "--allow-file-access-from-files",
                "--force-device-scale-factor=1",
                "about:blank",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| Error::tool(format!("Could not start the browser at {}: {e}", exe.display())))?;

        // The browser writes the port it chose into its profile folder.
        let started = Instant::now();
        let (port, path) = loop {
            if let Ok(text) = std::fs::read_to_string(&port_file) {
                let mut lines = text.lines();
                if let (Some(port), Some(path)) = (lines.next().and_then(|p| p.trim().parse::<u16>().ok()), lines.next()) {
                    break (port, path.trim().to_string());
                }
            }
            if let Ok(Some(status)) = child.try_wait() {
                let _ = std::fs::remove_dir_all(&profile);
                return Err(Error::tool(format!("The browser closed as soon as it started ({status}). Try the other one of Chrome and Edge in Settings.")));
            }
            if started.elapsed() > Duration::from_secs(30) {
                let _ = child.kill();
                let _ = std::fs::remove_dir_all(&profile);
                return Err(Error::tool("The browser did not start within 30 seconds."));
            }
            std::thread::sleep(Duration::from_millis(50));
        };

        let socket = match connect(&format!("ws://127.0.0.1:{port}{path}")) {
            Ok(s) => s,
            Err(e) => {
                let _ = child.kill();
                let _ = std::fs::remove_dir_all(&profile);
                return Err(e);
            }
        };
        let mut browser = Browser { child: Some(child), profile: Some(profile), socket, session: None, next_id: 0 };
        let target = browser.call("Target.createTarget", json!({ "url": "about:blank" }))?;
        let target_id = target["targetId"].as_str().ok_or_else(|| Error::tool("The browser did not open a page."))?.to_string();
        let attached = browser.call("Target.attachToTarget", json!({ "targetId": target_id, "flatten": true }))?;
        browser.session = Some(attached["sessionId"].as_str().ok_or_else(|| Error::tool("The browser did not attach to its page."))?.to_string());
        browser.call("Page.enable", json!({}))?;
        Ok(browser)
    }

    /// Attach to a page of a browser (or app window) that is already running with
    /// remote debugging on. Used to check the app itself while building it.
    pub fn attach(port: u16, title_contains: Option<&str>) -> Result<Self> {
        let list: Value = reqwest::blocking::get(format!("http://127.0.0.1:{port}/json/list"))?.json()?;
        let pages = list.as_array().cloned().unwrap_or_default();
        let page = pages
            .iter()
            .filter(|p| p["type"] == "page")
            .find(|p| title_contains.is_none_or(|t| p["title"].as_str().unwrap_or("").contains(t) || p["url"].as_str().unwrap_or("").contains(t)))
            .ok_or_else(|| Error::tool(format!("No page found on port {port}.")))?;
        let url = page["webSocketDebuggerUrl"].as_str().ok_or_else(|| Error::tool("That page cannot be attached to."))?;
        let socket = connect(url)?;
        Ok(Browser { child: None, profile: None, socket, session: None, next_id: 0 })
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next_id += 1;
        let id = self.next_id;
        let mut msg = json!({ "id": id, "method": method, "params": params });
        if let Some(s) = &self.session {
            msg["sessionId"] = json!(s);
        }
        let bad = |e: tungstenite::Error| Error::tool(format!("Lost touch with the browser during {method}: {e}"));
        self.socket.send(Message::text(msg.to_string())).map_err(bad)?;
        loop {
            let reply = self.socket.read().map_err(bad)?;
            let Message::Text(text) = reply else { continue };
            let Ok(v) = serde_json::from_str::<Value>(text.as_str()) else { continue };
            if v["id"].as_u64() != Some(id) {
                continue; // an event, or an answer to something older
            }
            if let Some(err) = v.get("error") {
                return Err(Error::tool(format!("The browser refused {method}: {}", err["message"].as_str().unwrap_or("unknown reason"))));
            }
            return Ok(v["result"].clone());
        }
    }

    /// Set the size of the page, in real pixels.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.call("Emulation.setDeviceMetricsOverride", json!({ "width": width, "height": height, "deviceScaleFactor": 1, "mobile": false }))?;
        Ok(())
    }

    /// Open a page and wait until it has finished loading.
    pub fn open(&mut self, url: &str, width: u32, height: u32) -> Result<()> {
        self.resize(width, height)?;
        self.call("Page.navigate", json!({ "url": url }))?;
        let started = Instant::now();
        loop {
            let state = self.eval("document.readyState").unwrap_or(Value::Null);
            if state == "complete" {
                return Ok(());
            }
            if started.elapsed() > Duration::from_secs(60) {
                return Err(Error::tool("The page with the beads did not finish loading."));
            }
            std::thread::sleep(Duration::from_millis(40));
        }
    }

    /// Run a line of script in the page and bring back its answer. Waits for promises.
    pub fn eval(&mut self, expression: &str) -> Result<Value> {
        let r = self.call("Runtime.evaluate", json!({ "expression": expression, "awaitPromise": true, "returnByValue": true }))?;
        if let Some(ex) = r.get("exceptionDetails") {
            let what = ex["exception"]["description"].as_str().or_else(|| ex["text"].as_str()).unwrap_or("unknown error");
            return Err(Error::tool(format!("The page hit an error: {what}")));
        }
        Ok(r["result"]["value"].clone())
    }

    /// A PNG of the page as it is now.
    pub fn screenshot(&mut self) -> Result<Vec<u8>> {
        let r = self.call("Page.captureScreenshot", json!({ "format": "png", "captureBeyondViewport": false }))?;
        let data = r["data"].as_str().ok_or_else(|| Error::tool("The browser did not return a picture."))?;
        base64::engine::general_purpose::STANDARD.decode(data).map_err(|e| Error::tool(format!("The picture from the browser was damaged: {e}")))
    }
}

fn rand_bits() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
}

impl Drop for Browser {
    fn drop(&mut self) {
        if self.child.is_some() {
            // Closing is asked of the browser itself, not of the page.
            self.session = None;
            if let MaybeTlsStream::Plain(stream) = self.socket.get_ref() {
                stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            }
            let _ = self.call("Browser.close", json!({}));
        }
        if let Some(mut child) = self.child.take() {
            let started = Instant::now();
            while started.elapsed() < Duration::from_secs(3) {
                if matches!(child.try_wait(), Ok(Some(_))) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(profile) = self.profile.take() {
            // The browser can hold its files for a moment after it exits.
            for _ in 0..10 {
                if std::fs::remove_dir_all(&profile).is_ok() || !profile.exists() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(150));
            }
        }
    }
}
