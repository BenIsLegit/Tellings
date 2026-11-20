//! `pvc` — make a prayer video from the command line.
//!
//! This is the door Claude Code and Codex use. It does exactly what the app
//! does, through the same engine. Run `pvc help` to see everything.

use pvc_engine::error::{Error, Result};
use pvc_engine::paths::Paths;
use pvc_engine::plan::{Context, SceneKind};
use pvc_engine::{cdp, elevenlabs, frames, library, make, reference, secrets, setup, speech, tools};
use serde_json::{json, Value};
use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

const HELP: &str = "\
pvc - make a follow-along prayer video

  pvc list                       the setups you can start from
  pvc show  [setup] [changes]    what the video will be, and what it will cost
  pvc make  [setup] [changes]    make it
  pvc frame [setup] --scene 12 --to still.png [changes]
                                 draw one moment of the video as a picture
  pvc save <name> [setup] [changes]
                                 keep a setup under a name, to use again
  pvc delete <name>              remove a saved setup
  pvc export [setup] [changes]   print the whole setup, every setting filled in
  pvc options                    every setting and its choices
  pvc voices                     the voices in your ElevenLabs account
  pvc credits                    credits left in your ElevenLabs account
  pvc key status|set|clear       the ElevenLabs API key on this computer
  pvc doctor                     check that everything the tool needs is here

A setup is a preset (luminous, joyful, today ...), a saved setup, or a .json
file. Leave it out and you get today's mysteries.

Changes, any number of each:
  --set voice.name=\"Imperial - Male\"     change one setting for the whole video
  --rule \"mystery=3: voice.name=Bella\"   change it for part of the video
  --mysteries joyful                     short for --set prayers.mysteries=joyful
  --voice NAME                           short for --set voice.name=NAME
  --no-voice                             short for --set voice.on=false
  --screen tv-1080p                      short for --set screen.preset=tv-1080p
  --to FOLDER                            where the files go

Other:
  --approve-credits N    agree to spend up to N ElevenLabs credits on new speech
  --date 2026-11-19      pretend it is this day (for \"today\")
  --json                 answer in JSON, for programs

Nothing spends credits unless you pass --approve-credits. `pvc show` tells you
the number first.
";

struct Args {
    command: String,
    words: Vec<String>,
    sets: Vec<(String, String)>,
    rules: Vec<String>,
    to: Option<String>,
    scene: Option<String>,
    approve: u64,
    date: Option<String>,
    json: bool,
    markdown: bool,
    refresh: bool,
}

fn parse_args(raw: Vec<String>) -> Result<Args> {
    let mut a = Args { command: String::new(), words: vec![], sets: vec![], rules: vec![], to: None, scene: None, approve: 0, date: None, json: false, markdown: false, refresh: false };
    let mut it = raw.into_iter();
    let need = |it: &mut std::vec::IntoIter<String>, flag: &str| it.next().ok_or_else(|| Error::other(format!("{flag} needs something after it. Run: pvc help")));
    let pair = |text: String, flag: &str| -> Result<(String, String)> {
        text.split_once('=').map(|(k, v)| (k.trim().to_string(), v.to_string())).ok_or_else(|| Error::other(format!("{flag} should look like {flag} voice.name=Bella. Got: {text}")))
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--set" => a.sets.push(pair(need(&mut it, "--set")?, "--set")?),
            "--rule" => a.rules.push(need(&mut it, "--rule")?),
            "--mysteries" => a.sets.push(("prayers.mysteries".into(), need(&mut it, "--mysteries")?)),
            "--voice" => a.sets.push(("voice.name".into(), json!(need(&mut it, "--voice")?).to_string())),
            "--no-voice" => a.sets.push(("voice.on".into(), "false".into())),
            "--screen" => a.sets.push(("screen.preset".into(), need(&mut it, "--screen")?)),
            "--to" | "--out" => a.to = Some(need(&mut it, "--to")?),
            "--scene" => a.scene = Some(need(&mut it, "--scene")?),
            "--approve-credits" => {
                let n = need(&mut it, "--approve-credits")?;
                a.approve = n.replace([',', '_'], "").parse().map_err(|_| Error::other(format!("--approve-credits needs a number. Got: {n}")))?;
            }
            "--date" => a.date = Some(need(&mut it, "--date")?),
            "--json" => a.json = true,
            "--markdown" => a.markdown = true,
            "--refresh" => a.refresh = true,
            "-h" | "--help" => a.command = "help".into(),
            flag if flag.starts_with("--") => {
                // --voice.name=Bella is as good as --set voice.name=Bella
                match flag[2..].split_once('=') {
                    Some((k, v)) if k.contains('.') => a.sets.push((k.to_string(), v.to_string())),
                    _ => return Err(Error::other(format!("I do not know {flag}. Run: pvc help"))),
                }
            }
            _ if a.command.is_empty() => a.command = arg,
            _ => a.words.push(arg),
        }
    }
    if a.command.is_empty() {
        a.command = "help".into();
    }
    Ok(a)
}

/// A setup with the command line's changes laid over it.
struct Chosen {
    /// The sparse setup: what to save if the person asks to keep it.
    doc: Value,
    setup: setup::Setup,
    ctx: Context,
}

fn choose(paths: &Paths, a: &Args, reference: Option<&str>) -> Result<Chosen> {
    let found = library::find(paths, reference.unwrap_or("today"))?;
    // Changes are laid over a setup that extends the one named, so saving keeps only the changes.
    let mut doc = match found.entry.kind {
        library::Kind::File => found.doc.clone(),
        _ => json!({ "extends": found.entry.id }),
    };
    for (path, value) in &a.sets {
        setup::set_path(&mut doc, path, setup::parse_value(value))?;
    }
    if let Some(folder) = &a.to {
        setup::set_path(&mut doc, "output.folder", Value::String(folder.clone()))?;
    }
    if !a.rules.is_empty() {
        let mut rules = doc["rules"].as_array().cloned().unwrap_or_default();
        for text in &a.rules {
            rules.push(serde_json::to_value(setup::parse_rule(text)?)?);
        }
        doc["rules"] = Value::Array(rules);
    }
    let resolved = library::resolve(paths, &doc)?;
    let setup = setup::from_doc(&resolved)?;
    let today = match &a.date {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| Error::other(format!("--date should look like 2026-11-19. Got: {d}")))?,
        None => chrono::Local::now().date_naive(),
    };
    Ok(Chosen { doc, setup, ctx: Context { today, search: paths.search(found.dir.as_deref()) } })
}

fn clock(seconds: f32) -> String {
    let s = seconds.round() as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

fn commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn print_json(v: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn cost_lines(est: &speech::Estimate, key: Option<secrets::KeySource>) -> Vec<String> {
    let mut out = Vec::new();
    if est.letters_to_record == 0 {
        if est.letters_kept > 0 {
            out.push("Credits: none. Every piece of speech is already recorded and kept.".into());
        } else {
            out.push("Credits: none. This video has no generated voice.".into());
        }
    } else {
        out.push(format!("Credits: about {} to record {} new piece{} of speech.", commas(est.letters_to_record), est.pieces_to_record, if est.pieces_to_record == 1 { "" } else { "s" }));
        for v in &est.by_voice {
            out.push(format!("  {}: {} letters in {} pieces", v.voice, commas(v.letters), v.pieces));
        }
        if est.letters_kept > 0 {
            out.push(format!("  ({} letters are already recorded and will be reused.)", commas(est.letters_kept)));
        }
        if key.is_none() {
            out.push("  No ElevenLabs API key is saved yet. Add it in the app under Settings, or run: pvc key set".into());
        }
    }
    for m in &est.missing_recordings {
        out.push(format!("Recording not found: {m}"));
    }
    out
}

fn run(a: Args) -> Result<i32> {
    let paths = Paths::standard();
    let first = a.words.first().map(String::as_str);
    match a.command.as_str() {
        "help" => print!("{HELP}"),
        "version" | "--version" | "-V" => println!("pvc {}", env!("CARGO_PKG_VERSION")),

        "list" => {
            let (presets, saved) = (library::presets(), library::saved(&paths));
            if a.json {
                return print_json(&json!({ "presets": presets, "saved": saved })).map(|_| 0);
            }
            println!("Presets (they come with the tool)");
            for e in &presets {
                println!("  {:<18} {}", e.id, e.description);
            }
            println!("\nYour saved setups");
            if saved.is_empty() {
                println!("  None yet. Save one with: pvc save \"My rosary\" luminous --voice \"Some Voice\"");
            }
            for e in &saved {
                println!("  {:<18} {}{}", e.id, e.name, e.extends.as_ref().map(|x| format!("  (starts from {x})")).unwrap_or_default());
            }
            println!("\nMake one with: pvc make luminous");
        }

        "show" | "estimate" | "plan" => {
            let c = choose(&paths, &a, first)?;
            let look = make::look(&paths, &c.setup, &c.ctx)?;
            if a.json {
                return print_json(&look).map(|_| 0);
            }
            let p = &look.plan;
            let (w, h) = (p.stage.width, p.stage.height);
            println!("{}", p.name);
            println!("  Mysteries: {}", p.set_name);
            println!("  Prayers:   {}", p.prayer_count);
            println!("  Length:    about {} (a guess until the voice is recorded)", clock(p.seconds));
            println!("  Screen:    {w} x {h}");
            let voices: std::collections::BTreeSet<String> = p.scenes.iter().filter(|s| s.kind == SceneKind::Prayer && s.voice.on).map(|s| s.voice.name.clone()).collect();
            println!("  Voice:     {}", if voices.is_empty() { "none".to_string() } else { voices.into_iter().collect::<Vec<_>>().join(", ") });
            let pictures = p.scenes.iter().filter_map(|s| s.picture.as_ref().map(|x| x.file.clone())).collect::<std::collections::BTreeSet<_>>().len();
            println!("  Pictures:  {pictures}");
            for line in cost_lines(&look.estimate, look.key) {
                println!("  {line}");
            }
            for w in &p.warnings {
                println!("  Note: {w}");
            }
            println!("  Would write:");
            for f in &look.files {
                println!("    {f}");
            }
            if look.estimate.letters_to_record > 0 {
                println!("\nTo make it and agree to the credits: pvc make ... --approve-credits {}", look.estimate.letters_to_record);
            }
        }

        "make" | "render" => {
            let c = choose(&paths, &a, first)?;
            let cancel = AtomicBool::new(false);
            let quiet = a.json;
            let mut last = String::new();
            let mut on_progress = |p: make::Progress| {
                if quiet {
                    return;
                }
                // One line for every fifth of each stage is plenty.
                let step = (p.total / 5).max(1);
                let tag = format!("{:?}{}", p.stage, p.done / step);
                if tag != last && (p.done % step == 0 || p.done == p.total) {
                    last = tag;
                    eprintln!("[{:>3.0}%] {}", p.fraction * 100.0, p.message);
                }
            };
            let req = make::Request { paths: &paths, setup: &c.setup, ctx: &c.ctx, approved_letters: a.approve, voice_service: std::env::var("PVC_ELEVENLABS_URL").ok() };
            let made = match make::make(&req, &mut on_progress, &cancel) {
                Ok(m) => m,
                Err(Error::NeedsApproval(msg)) => {
                    let look = make::look(&paths, &c.setup, &c.ctx)?;
                    if a.json {
                        print_json(&json!({ "needsApproval": true, "message": msg, "credits": look.estimate.letters_to_record, "estimate": look.estimate }))?;
                    } else {
                        eprintln!("{msg}");
                        for line in cost_lines(&look.estimate, look.key) {
                            eprintln!("  {line}");
                        }
                        eprintln!("\nAsk the person first. If they say yes, run the same command again with:\n  --approve-credits {}", look.estimate.letters_to_record);
                    }
                    return Ok(2);
                }
                Err(e) => return Err(e),
            };
            if a.json {
                return print_json(&made).map(|_| 0);
            }
            println!("\nMade: {}  ({} long, took {})", made.name, clock(made.seconds), clock(made.took));
            for f in &made.files {
                println!("  {:<9} {}", f.kind, f.path);
            }
            if made.letters_recorded > 0 {
                println!("  Recorded {} letters of new speech (about that many credits).", commas(made.letters_recorded));
            }
            for n in &made.pace {
                println!("  Voice \"{}\" speaks {:.1} letters a second on its own; sped up {:.2}x to match the first video's pace.", n.voice, n.natural, n.speedup);
            }
            if let Some(s) = made.hail_mary_seconds {
                println!("  One Hail Mary takes {s:.1} seconds. (In the first video it took about 9.2.)");
            }
            for w in &made.warnings {
                println!("  Note: {w}");
            }
        }

        "frame" | "still" => {
            let c = choose(&paths, &a, first)?;
            let plan = pvc_engine::plan::build(&c.setup, &c.ctx)?;
            // A scene can be a number (0 is the opening title) or a bead, like d3b10.
            let wanted = a.scene.clone().unwrap_or_else(|| "1".into());
            let scene = match wanted.parse::<usize>() {
                Ok(n) => n.min(plan.scenes.len() - 1),
                Err(_) => plan
                    .scenes
                    .iter()
                    .position(|s| s.bead.as_deref() == Some(wanted.as_str()) || s.id == wanted)
                    .ok_or_else(|| Error::other(format!("--scene takes a number from 0 to {} or a bead like d3b10. Got: {wanted}", plan.scenes.len() - 1)))?,
            };
            let target = PathBuf::from(a.to.clone().unwrap_or_else(|| "still.png".into()));
            let work = paths.work().join(format!("frame-{}", std::process::id()));
            let browser = tools::find_browser(&paths.settings())?;
            let shot = frames::Shot { scene, line: if plan.scenes[scene].words.follow { Some(0) } else { None } };
            let result = frames::shoot(&browser, &work, &plan, &[shot], &work.join("stills"), |_, _| true);
            let taken = match result {
                Ok(t) => t,
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&work);
                    return Err(e);
                }
            };
            if let Some(dir) = target.parent().filter(|d| !d.as_os_str().is_empty()) {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::copy(&taken[0].file, &target)?;
            let _ = std::fs::remove_dir_all(&work);
            let s = &plan.scenes[scene];
            if a.json {
                return print_json(&json!({ "file": target, "scene": scene, "title": s.title, "bead": s.bead, "wordsFit": !taken[0].overflow, "warnings": plan.warnings })).map(|_| 0);
            }
            println!("Drew scene {scene} ({}{}) to {}", s.title, s.bead.as_ref().map(|b| format!(", bead {b}")).unwrap_or_default(), target.display());
            if taken[0].overflow {
                println!("Note: the words do not fully fit on this screen.");
            }
            for w in &plan.warnings {
                println!("Note: {w}");
            }
        }

        "save" => {
            let name = first.ok_or_else(|| Error::other("Give the setup a name: pvc save \"My rosary\" luminous --voice \"Some Voice\""))?;
            let c = choose(&paths, &a, a.words.get(1).map(String::as_str))?;
            let entry = library::save(&paths, name, &c.doc)?;
            if a.json {
                return print_json(&entry).map(|_| 0);
            }
            println!("Saved \"{}\". Use it with: pvc make {}", entry.name, entry.id);
            println!("It is kept at {}", paths.setups().join(format!("{}.json", entry.id)).display());
        }

        "delete" | "remove" => {
            let name = first.ok_or_else(|| Error::other("Say which saved setup: pvc delete my-rosary"))?;
            library::delete(&paths, name)?;
            println!("Deleted {name}.");
        }

        "export" => {
            let c = choose(&paths, &a, first)?;
            print_json(&c.setup)?;
        }

        "options" | "settings" => print!("{}", if a.markdown { reference::markdown() } else { reference::text() }),

        "voices" => {
            let Some((key, _)) = secrets::key()? else {
                return Err(Error::voice("No ElevenLabs API key is saved yet. Add it in the app under Settings, or run: pvc key set"));
            };
            let list = speech::voices(&paths, &elevenlabs::Client::new(&key)?, a.refresh)?;
            if a.json {
                return print_json(&list).map(|_| 0);
            }
            for v in &list {
                println!("  {:<34} {}", v.name, v.category.as_deref().unwrap_or(""));
            }
            println!("\n{} voices. Use one with: --voice \"{}\"", list.len(), list.first().map(|v| v.name.as_str()).unwrap_or("Name"));
        }

        "credits" => match make::credits()? {
            Some(c) if a.json => print_json(&json!({ "used": c.used, "limit": c.limit, "left": c.left() }))?,
            Some(c) => println!("{} credits left of {} this period.", commas(c.left() as u64), commas(c.limit.max(0) as u64)),
            None => return Err(Error::voice("No ElevenLabs API key is saved yet. Add it in the app under Settings, or run: pvc key set")),
        },

        "key" => match first.unwrap_or("status") {
            "status" => match secrets::source() {
                Some(secrets::KeySource::Environment) => println!("An ElevenLabs API key is set in the {} environment variable.", secrets::ENV_VAR),
                Some(secrets::KeySource::PasswordStore) => println!("An ElevenLabs API key is saved in this computer's password store."),
                None => println!("No ElevenLabs API key is saved. Add it in the app under Settings, or run: pvc key set"),
            },
            "set" => {
                // The key is read from the keyboard or a pipe, never from the command
                // line, so it does not end up in a history file or a transcript.
                let stdin = std::io::stdin();
                if stdin.is_terminal() {
                    eprint!("Paste your ElevenLabs API key and press Enter: ");
                    std::io::stderr().flush().ok();
                }
                let mut text = String::new();
                stdin.lock().read_to_string(&mut text).ok();
                let key = text.lines().next().unwrap_or("").trim();
                secrets::set_key(key)?;
                println!("Saved. The key is kept in this computer's password store, not in any file.");
            }
            "clear" | "remove" => {
                secrets::clear_key()?;
                println!("Removed the saved key.");
            }
            other => return Err(Error::other(format!("pvc key takes status, set or clear. Got: {other}"))),
        },

        "doctor" => {
            let settings = paths.settings();
            let ffmpeg = tools::find_ffmpeg(&settings);
            let browser = tools::find_browser(&settings);
            let key = secrets::source();
            if a.json {
                return print_json(&json!({
                    "ffmpeg": ffmpeg.as_ref().ok(), "browser": browser.as_ref().ok(), "key": key,
                    "data": paths.data, "videos": paths.output_folder(), "keptSpeechBytes": speech::kept_bytes(&paths),
                }))
                .map(|_| 0);
            }
            let line = |name: &str, r: &Result<PathBuf>| match r {
                Ok(p) => println!("  ok       {name}: {}", p.display()),
                Err(e) => println!("  MISSING  {name}: {e}"),
            };
            line("FFmpeg", &ffmpeg);
            line("Browser", &browser);
            match key {
                Some(_) => println!("  ok       ElevenLabs key: saved"),
                None => println!("  --       ElevenLabs key: none (only needed for a generated voice)"),
            }
            println!("  Videos go to:      {}", paths.output_folder().display());
            println!("  Pictures folder:   {}", paths.pictures().display());
            println!("  Setups and speech: {}", paths.data.display());
            println!("  Kept speech:       {:.1} MB", speech::kept_bytes(&paths) as f64 / 1_048_576.0);
            if ffmpeg.is_err() || browser.is_err() {
                return Ok(1);
            }
        }

        // For building the app: take a picture of the running app window.
        // Start the app with WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9347 first.
        // It only ever attaches to this app's own window, by its title. Other
        // programs on the computer can have a debugging port open too.
        "dev-shot" => {
            let port: u16 = first.and_then(|p| p.parse().ok()).unwrap_or(9347);
            let mut page = cdp::Browser::attach(port, Some("Prayer Video Creator"))?;
            if let Some(script) = a.words.get(1) {
                let script = if std::path::Path::new(script).is_file() { std::fs::read_to_string(script)? } else { script.clone() };
                println!("{}", page.eval(&script)?);
            }
            if let Some(target) = &a.to {
                std::fs::write(target, page.screenshot()?)?;
                println!("wrote {target}");
            }
        }

        other => {
            eprintln!("I do not know the command \"{other}\".\n");
            print!("{HELP}");
            return Ok(1);
        }
    }
    Ok(0)
}

fn main() {
    let code = match parse_args(std::env::args().skip(1).collect()).and_then(run) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    };
    std::process::exit(code);
}
