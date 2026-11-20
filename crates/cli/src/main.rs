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
  pvc rerecord [setup] --scene d1b3 [changes]
                                 throw away the kept recording on one bead, so
                                 the next video records it afresh
  pvc save <name> [setup] [changes]
                                 keep a setup under a name, to use again
  pvc delete <name>              remove a saved setup
  pvc export [setup] [changes]   print the whole setup, every setting filled in
  pvc options                    every setting and its choices
  pvc voices                     the voices in your ElevenLabs account
  pvc voices \"Name\"              how that voice is set to speak at ElevenLabs
  pvc models                     the ElevenLabs models, and what each charges
  pvc dictionaries               the pronunciation dictionaries in your account
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
  --voice \"Bella + George\"               several voices take turns
  --model eleven_flash_v2_5              short for --set voice.model=eleven_flash_v2_5
  --takes 5                              short for --set voice.takes=5: five different
                                         recordings of each prayer take turns
  --no-voice                             short for --set voice.on=false
  --screen tv-1080p                      short for --set screen.preset=tv-1080p
  --to FOLDER                            where the files go

Other:
  --approve-credits N    agree to spend up to N ElevenLabs credits, on new speech
                         and on music made by ElevenLabs together
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
            "--takes" => a.sets.push(("voice.takes".into(), need(&mut it, "--takes")?)),
            "--model" => a.sets.push(("voice.model".into(), json!(need(&mut it, "--model")?).to_string())),
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

/// A scene can be named by its number (0 is the opening title) or by a bead, like d3b10.
fn find_scene(plan: &pvc_engine::plan::Plan, wanted: &str) -> Result<usize> {
    match wanted.parse::<usize>() {
        Ok(n) => Ok(n.min(plan.scenes.len() - 1)),
        Err(_) => plan
            .scenes
            .iter()
            .position(|s| s.bead.as_deref() == Some(wanted) || s.id == wanted)
            .ok_or_else(|| Error::other(format!("--scene takes a number from 0 to {} or a bead like d3b10. Got: {wanted}", plan.scenes.len() - 1))),
    }
}

/// "the Hail Mary by Imperial - Male (take 2 of 3)"
fn heard_line(h: &speech::Heard, title: &str) -> String {
    let what = if h.part == speech::PartKind::Announce { "the mystery's name".to_string() } else { title.to_string() };
    let take = if h.takes > 1 { format!(" (take {} of {})", h.take, h.takes) } else { String::new() };
    format!("{what} by {}{take}", h.voice)
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

const NO_KEY: &str = "No ElevenLabs API key is saved yet. Add it in the app under Settings, or run: pvc key set";

/// A way to ask ElevenLabs things, if a key is saved. Asking is free. Only `pvc make` spends.
fn client() -> Result<Option<elevenlabs::Client>> {
    match secrets::key()? {
        Some((key, _)) => Ok(Some(elevenlabs::Client::from_env(&key)?)),
        None => Ok(None),
    }
}

fn need_client() -> Result<elevenlabs::Client> {
    client()?.ok_or_else(|| Error::voice(NO_KEY))
}

/// "1 credit a letter", "half a credit a letter"
fn rate_words(rate: f32) -> String {
    if (rate - 1.0).abs() < 0.001 {
        "1 credit a letter".into()
    } else if (rate - 0.5).abs() < 0.001 {
        "half a credit a letter".into()
    } else {
        format!("{} credits a letter", (rate * 100.0).round() / 100.0)
    }
}

/// What making the video would cost. The first line always starts "Credits:"
/// and names the one number to agree to: the voice and the music together.
fn cost_lines(look: &make::Look) -> Vec<String> {
    let (est, music) = (&look.estimate, &look.music);
    let mut out = Vec::new();
    let voice = format!("about {} to record {} new piece{} of speech.", commas(est.credits_to_record), est.pieces_to_record, if est.pieces_to_record == 1 { "" } else { "s" });
    // With music to pay for as well, the voice's lines sit one step in, under the number for both.
    let both = music.credits > 0;
    let step = if both { "    " } else { "  " };
    if look.credits == 0 {
        if est.letters_kept > 0 {
            out.push("Credits: none. Every piece of speech is already recorded and kept.".into());
        } else {
            out.push("Credits: none. This video has no generated voice.".into());
        }
    } else if both && est.letters_to_record > 0 {
        out.push(format!("Credits: about {} in all, for the voice and the music together.", commas(look.credits)));
        out.push(format!("  The voice: {voice}"));
    } else if both {
        out.push(format!("Credits: about {}, for the music.", commas(look.credits)));
    } else {
        out.push(format!("Credits: {voice}"));
    }
    if est.letters_to_record > 0 {
        for v in &est.by_voice {
            let credits = if v.credits == v.letters { String::new() } else { format!(", {} credits", commas(v.credits)) };
            out.push(format!("{step}{}: {} letters in {} piece{}{credits}", v.voice, commas(v.letters), v.pieces, if v.pieces == 1 { "" } else { "s" }));
        }
        if est.rates_guessed {
            out.push(format!("{step}(A guess. What this model charges is not known yet. The making stops before it goes over the number agreed to.)"));
        }
        if est.letters_kept > 0 {
            out.push(format!("{step}({} letters are already recorded and will be reused.)", commas(est.letters_kept)));
        }
    }
    if both {
        out.push(format!("  The music: about {} for {} made by ElevenLabs.", commas(music.credits), make::minutes_words(music.minutes)));
        out.push("    A rough figure. ElevenLabs does not give the price before it makes music. Afterwards the tool says what it came to.".into());
    } else if music.wanted && music.kept {
        out.push("  The music is already made and kept.".into());
    }
    if look.credits > 0 && look.key.is_none() {
        out.push("  No ElevenLabs API key is saved yet. Add it in the app under Settings, or run: pvc key set".into());
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
            let mut look = make::look(&paths, &c.setup, &c.ctx)?;
            // A cost is only as good as what is known of the model's price. Asking
            // ElevenLabs is free, and the answer is kept for a day.
            if look.estimate.letters_to_record > 0 {
                if let Some(client) = client()? {
                    if speech::models(&paths, &client, false).is_ok() {
                        look = make::look(&paths, &c.setup, &c.ctx)?;
                    }
                }
            }
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
            let spoken = || p.scenes.iter().filter(|s| s.kind == SceneKind::Prayer && s.voice.on);
            let voices: std::collections::BTreeSet<String> = spoken().map(|s| s.speaker.clone().unwrap_or_else(|| s.voice.name.clone())).collect();
            println!("  Voice:     {}", if voices.is_empty() { "none".to_string() } else { voices.into_iter().collect::<Vec<_>>().join(", ") });
            let rates = speech::Rates::kept(&paths);
            let models: std::collections::BTreeSet<String> = spoken().filter(|s| s.voice.recording.is_none()).map(|s| s.voice.model.clone()).collect();
            if !models.is_empty() {
                let named: Vec<String> = models.iter().map(|id| rates.model(id).map(|m| format!("{} ({id})", m.name)).unwrap_or_else(|| id.clone())).collect();
                println!("  Model:     {}", named.join(", "));
            }
            // Say so when prayers are recorded more than once, because that is what costs more.
            let most_takes = look.speech.iter().flatten().map(|h| h.takes).max().unwrap_or(1);
            if spoken().any(|s| s.voice.fresh_each_time) {
                println!("  Variety:   a new recording every time a prayer is said");
            } else if most_takes > 1 {
                println!("  Variety:   up to {most_takes} different recordings of a prayer take turns");
            }
            let pictures = p.scenes.iter().filter_map(|s| s.picture.as_ref().map(|x| x.file.clone())).collect::<std::collections::BTreeSet<_>>().len();
            println!("  Pictures:  {pictures}");
            if look.music.wanted {
                println!("  Music:     {} made by ElevenLabs, looped", make::minutes_words(look.music.minutes));
            } else if let Some(file) = c.setup.music.file.as_deref().filter(|f| !f.trim().is_empty()) {
                println!("  Music:     {file}");
            }
            for line in cost_lines(&look) {
                println!("  {line}");
            }
            for w in &p.warnings {
                println!("  Note: {w}");
            }
            println!("  Would write:");
            for f in &look.files {
                println!("    {f}");
            }
            if look.credits > 0 {
                println!("\nTo make it and agree to the credits: pvc make ... --approve-credits {}", look.credits);
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
            let req = make::Request { paths: &paths, setup: &c.setup, ctx: &c.ctx, approved_credits: a.approve, voice_service: None };
            let made = match make::make(&req, &mut on_progress, &cancel) {
                Ok(m) => m,
                Err(Error::NeedsApproval(msg)) => {
                    let look = make::look(&paths, &c.setup, &c.ctx)?;
                    if a.json {
                        print_json(&json!({ "needsApproval": true, "message": msg, "credits": look.credits, "estimate": look.estimate, "music": look.music }))?;
                    } else {
                        eprintln!("{msg}");
                        for line in cost_lines(&look) {
                            eprintln!("  {line}");
                        }
                        eprintln!("\nAsk the person first. If they say yes, run the same command again with:\n  --approve-credits {}", look.credits);
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
                let on_speech = made.credits_spent - made.music.as_ref().and_then(|m| m.charged).unwrap_or(0);
                println!("  Recorded {} letters of new speech. Spent {} credits.", commas(made.letters_recorded), commas(on_speech));
            }
            if let Some(said) = &made.music_said {
                println!("  {said}");
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
            let scene = find_scene(&plan, a.scene.as_deref().unwrap_or("1"))?;
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

        "rerecord" | "record-again" => {
            let c = choose(&paths, &a, first)?;
            let wanted = a.scene.as_deref().ok_or_else(|| Error::other("Say which bead: pvc rerecord luminous --scene d1b3"))?;
            let look = make::look(&paths, &c.setup, &c.ctx)?;
            let scene = find_scene(&look.plan, wanted)?;
            let title = look.plan.scenes[scene].title.clone();
            let kept: Vec<&speech::Heard> = look.speech[scene].iter().filter(|h| !h.own_recording && h.kept).collect();
            let gone = make::record_again(&paths, &c.setup, &c.ctx, scene)?;
            if a.json {
                return print_json(&json!({ "scene": scene, "title": title, "pieces": gone.pieces, "letters": gone.letters, "credits": gone.credits })).map(|_| 0);
            }
            if gone.pieces == 0 {
                println!("Nothing to throw away: no kept recording is heard on scene {scene} ({title}).");
            } else {
                for h in &kept {
                    println!("Threw away {}. It was heard {} time{} in this video.", heard_line(h, &title), h.uses, if h.uses == 1 { "" } else { "s" });
                }
                println!("Nothing has been spent. The next video that uses it will record it again: about {} credits.", commas(gone.credits));
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

        // One voice: how it is set up at ElevenLabs, which is what a setup gets where it says nothing.
        "voices" if first.is_some() => {
            let name = first.unwrap_or_default();
            let own = speech::own_settings(&paths, &need_client()?, name)?;
            if a.json {
                return print_json(&own).map(|_| 0);
            }
            let number = |v: Option<f32>| v.map(|v| ((v * 100.0).round() / 100.0).to_string()).unwrap_or_else(|| "not set".into());
            println!("\"{name}\" is set up at ElevenLabs like this. A setup that says nothing gets these.");
            println!("  voice.stability      {}", number(own.stability));
            println!("  voice.similarity     {}", number(own.similarity));
            println!("  voice.style          {}", number(own.style));
            println!("  voice.speakerBoost   {}", own.speaker_boost.map(|b| b.to_string()).unwrap_or_else(|| "not set".into()));
            println!("  voice.modelSpeed     {}", number(own.speed));
        }

        "voices" => {
            let list = speech::voices(&paths, &need_client()?, a.refresh)?;
            if a.json {
                return print_json(&list).map(|_| 0);
            }
            for v in &list {
                println!("  {:<34} {}", v.name, v.category.as_deref().unwrap_or(""));
            }
            println!("\n{} voices. Use one with: --voice \"{}\"", list.len(), list.first().map(|v| v.name.as_str()).unwrap_or("Name"));
        }

        "models" => {
            let list = speech::models(&paths, &need_client()?, a.refresh)?;
            if a.json {
                return print_json(&list).map(|_| 0);
            }
            println!("The ElevenLabs models that turn text into speech\n");
            for m in &list {
                let most = if m.max_letters > 0 { format!("up to {} letters a prayer", commas(m.max_letters)) } else { String::new() };
                println!("  {:<28} {:<26} {:<24} {most}", m.id, m.name, rate_words(m.credits_per_letter));
                if !m.description.is_empty() {
                    println!("      {}", m.description);
                }
                let mut does_not: Vec<&str> = Vec::new();
                if !m.can_use_style {
                    does_not.push("voice.style");
                }
                if !m.can_use_speaker_boost {
                    does_not.push("voice.speakerBoost");
                }
                let direction = if m.takes_direction { "Takes voice.direction. " } else { "" };
                let unused = if does_not.is_empty() { String::new() } else { format!("Does not use {}. ", does_not.join(" or ")) };
                println!("      {direction}{unused}{} language{}.", m.languages.len(), if m.languages.len() == 1 { "" } else { "s" });
            }
            let other = list.iter().map(|m| m.id.as_str()).find(|id| *id != setup::USUAL_MODEL).unwrap_or("ID");
            println!("\nThe usual model is {}. Pick another with: --model {other}", setup::USUAL_MODEL);
            println!("Or for part of the video: --rule \"prayer=hailmary: voice.model={other}\"");
            println!("A prayer recorded with one model is recorded again, and paid for again, with another.");
            println!("The first video was recorded with {0}. --model {0} brings back its sound and its kept recordings.", setup::FIRST_MODEL);
        }

        "dictionaries" => {
            let list = need_client()?.dictionaries()?;
            if a.json {
                return print_json(&list).map(|_| 0);
            }
            if list.is_empty() {
                println!("There are no pronunciation dictionaries in your ElevenLabs account.");
                println!("Make one at elevenlabs.io when the voice keeps saying a word wrong. Then: --set voice.dictionary=\"Its Name\"");
            } else {
                for d in &list {
                    println!("  {:<34} {} word{}", d.name, d.rules, if d.rules == 1 { "" } else { "s" });
                }
                println!("\nUse one with: --set voice.dictionary=\"{}\"", list[0].name);
            }
        }

        "credits" => match make::credits()? {
            Some(c) if a.json => print_json(&json!({ "used": c.used, "limit": c.limit, "left": c.left(), "tier": c.tier, "resets": c.resets }))?,
            Some(c) => {
                println!("{} credits left of {} this period.", commas(c.left() as u64), commas(c.limit.max(0) as u64));
                if let Some(day) = c.resets.and_then(|t| chrono::DateTime::from_timestamp(t, 0)) {
                    println!("The count starts again on {}.", day.with_timezone(&chrono::Local).format("%-d %B %Y"));
                }
                if let Some(tier) = &c.tier {
                    println!("Plan: {tier}.");
                }
            }
            None => return Err(Error::voice(NO_KEY)),
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
