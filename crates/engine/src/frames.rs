//! Stills. One picture for every moment of the video that looks different.
//!
//! The drawing itself is done by `renderer/frame.js` in a browser with no
//! window. This file builds the page that holds it, and takes the pictures.

use crate::cdp::Browser;
use crate::error::{Error, Result};
use crate::plan::Plan;
use crate::tools;
use base64::Engine as _;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const FRAME_JS: &str = include_str!("../../../renderer/frame.js");
const FRAME_CSS: &str = include_str!("../../../renderer/frame.css");
const CARDO_400: &[u8] = include_bytes!("../../../renderer/fonts/cardo-latin-400-normal.woff2");
const CARDO_700: &[u8] = include_bytes!("../../../renderer/fonts/cardo-latin-700-normal.woff2");
const FIRA_400: &[u8] = include_bytes!("../../../renderer/fonts/fira-sans-latin-400-normal.woff2");
const FIRA_500: &[u8] = include_bytes!("../../../renderer/fonts/fira-sans-latin-500-normal.woff2");

/// One still to take: a scene, and the line lit up in it when following along.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shot {
    pub scene: usize,
    pub line: Option<usize>,
}

impl Shot {
    /// The file name of this still, without the ending.
    pub fn name(&self, plan: &Plan) -> String {
        match self.line {
            Some(l) => format!("{}-line{l:02}", plan.scenes[self.scene].id),
            None => plan.scenes[self.scene].id.clone(),
        }
    }
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// JSON that is safe to place inside a `<script>` tag.
fn script_json(value: &impl serde::Serialize) -> Result<String> {
    Ok(serde_json::to_string(value)?.replace('<', "\\u003c").replace('\u{2028}', "\\u2028").replace('\u{2029}', "\\u2029"))
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "image/png",
    }
}

/// A picture file as text a page can carry inside itself.
pub fn data_url(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|e| Error::other(format!("Could not open the picture {}: {e}", path.display())))?;
    Ok(format!("data:{};base64,{}", mime(path), b64(&bytes)))
}

fn head(title: &str) -> String {
    let fonts = serde_json::json!({
        "cardo400": format!("data:font/woff2;base64,{}", b64(CARDO_400)),
        "cardo700": format!("data:font/woff2;base64,{}", b64(CARDO_700)),
        "fira400": format!("data:font/woff2;base64,{}", b64(FIRA_400)),
        "fira500": format!("data:font/woff2;base64,{}", b64(FIRA_500)),
    });
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>\n<style>html,body{{margin:0;background:#000;}}\n{FRAME_CSS}</style>\n</head>\n<body>\n<script>\n{FRAME_JS}\n</script>\n<script>PVC.installFonts(PVC.fontCss({fonts}));</script>\n",
        title.replace('&', "&amp;").replace('<', "&lt;")
    )
}

/// The page the stills are taken from. Pictures are read straight from disk.
pub fn shots_page(plan: &Plan) -> Result<String> {
    let mut html = head(&plan.name);
    html.push_str(&format!(
        "<div id=\"host\"></div>\n<script>\nwindow.PLAN = {};\nwindow.FRAME = PVC.create(document.getElementById('host'), PLAN, {{ picture: function (f) {{ return 'file:///' + encodeURI(f).replace(/#/g, '%23').replace(/\\?/g, '%3F'); }} }});\nwindow.READY = FRAME.ready.then(function () {{ return FRAME.preload(); }}).then(function () {{ return true; }});\n</script>\n</body>\n</html>\n",
        script_json(plan)?
    ));
    Ok(html)
}

/// The tap-through guide: one file a person can open anywhere, pictures and all.
pub fn guide_page(plan: &Plan) -> Result<String> {
    let mut pictures: BTreeMap<String, String> = BTreeMap::new();
    for scene in &plan.scenes {
        if let Some(p) = &scene.picture {
            if !pictures.contains_key(&p.file) {
                pictures.insert(p.file.clone(), data_url(Path::new(&p.file))?);
            }
        }
    }
    // The page should not say where the pictures sat on the maker's computer.
    let mut plan = plan.clone();
    let mut short: BTreeMap<String, String> = BTreeMap::new();
    for (i, (file, url)) in pictures.into_iter().enumerate() {
        let key = format!("picture-{}", i + 1);
        for scene in plan.scenes.iter_mut() {
            if let Some(p) = scene.picture.as_mut().filter(|p| p.file == file) {
                p.file = key.clone();
            }
        }
        short.insert(key, url);
    }
    let mut html = head(&plan.title);
    html.push_str(&format!(
        "<script>\nwindow.PLAN = {};\nwindow.PICTURES = {};\nPVC.guide(PLAN, {{ picture: function (f) {{ return PICTURES[f] || ''; }} }});\n</script>\n</body>\n</html>\n",
        script_json(&plan)?,
        script_json(&short)?
    ));
    Ok(html)
}

/// What came back from drawing one still.
#[derive(Clone, Debug)]
pub struct Taken {
    pub shot: Shot,
    pub file: PathBuf,
    /// True when the words did not fit even at their smallest.
    pub overflow: bool,
}

/// Take the stills. `on_done(i, n)` is called after each one; return false from
/// it to stop early.
pub fn shoot(browser_exe: &Path, work: &Path, plan: &Plan, shots: &[Shot], out_dir: &Path, mut on_done: impl FnMut(usize, usize) -> bool) -> Result<Vec<Taken>> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::create_dir_all(work)?;
    let page = work.join("frame.html");
    std::fs::write(&page, shots_page(plan)?)?;

    let mut browser = Browser::launch(browser_exe, work)?;
    browser.open(&tools::file_url(&page), plan.stage.width, plan.stage.height)?;
    let ready = browser.eval("window.READY")?;
    if ready != true {
        return Err(Error::tool("The page with the beads did not get ready."));
    }
    // Make sure the fonts the first video used really loaded. A missing font
    // would quietly change every line of text.
    let fonts = browser.eval("[...document.fonts].filter(f => f.status === 'loaded').map(f => f.family.replace(/\"/g, '')).join('|')")?;
    let fonts = fonts.as_str().unwrap_or("");
    if !fonts.contains("Cardo") || !fonts.contains("Fira Sans") {
        return Err(Error::tool(format!("The fonts did not load in the browser (got: {fonts}).")));
    }

    let mut taken = Vec::with_capacity(shots.len());
    for (i, shot) in shots.iter().enumerate() {
        let line = shot.line.map(|l| l.to_string()).unwrap_or_else(|| "null".into());
        let fit = browser.eval(&format!("FRAME.show({}, {{ line: {line} }})", shot.scene))?;
        let file = out_dir.join(format!("{}.png", shot.name(plan)));
        std::fs::write(&file, browser.screenshot()?)?;
        taken.push(Taken { shot: *shot, file, overflow: fit["overflow"] == true });
        if !on_done(i + 1, shots.len()) {
            return Err(Error::Cancelled);
        }
    }
    Ok(taken)
}
