//! Making a video, start to finish.
//!
//! `look` is free and fast: it plans the video and says what it would cost.
//! `make` does the work. It will not spend credits unless it is told, in
//! numbers, how many credits of new speech have been agreed to, and it stops
//! rather than go over that number.

use crate::audio::{self, Timeline};
use crate::elevenlabs::{Client, Credits};
use crate::error::{Error, Result};
use crate::frames::{self, Shot};
use crate::paths::{file_safe, Paths};
use crate::plan::{self, Context, Plan};
use crate::secrets::{self, KeySource};
use crate::setup::{Mysteries, Setup};
use crate::speech::{self, Estimate, Forgotten, Heard, PaceNote};
use crate::tools;
use crate::video;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Look {
    pub plan: Plan,
    pub estimate: Estimate,
    /// For each scene of the plan, the speech heard on it: which voice, which
    /// take, and whether it is already recorded.
    pub speech: Vec<Vec<Heard>>,
    /// Where the ElevenLabs key was found, if there is one.
    pub key: Option<KeySource>,
    /// The files `make` would write.
    pub files: Vec<String>,
}

/// The base name for the files, without an ending.
fn base_name(setup: &Setup, plan: &Plan) -> String {
    if let Some(n) = setup.output.file_name.as_deref().filter(|n| !n.trim().is_empty()) {
        return file_safe(n);
    }
    // "Today" changes with the day, so name the file after the mysteries it turned
    // out to be. Monday's video should not overwrite Sunday's.
    if setup.prayers.mysteries == Mysteries::Today {
        let own = setup.name.trim();
        let set = format!("{} Mysteries", plan.set_name);
        return file_safe(&if own.is_empty() || own == "Today's Mysteries" { set } else { format!("{set} - {own}") });
    }
    file_safe(&plan.name)
}

fn out_files(paths: &Paths, setup: &Setup, plan: &Plan) -> Vec<(&'static str, PathBuf)> {
    let folder = setup.output.folder.as_deref().filter(|f| !f.trim().is_empty()).map(PathBuf::from).unwrap_or_else(|| paths.output_folder());
    let base = base_name(setup, plan);
    let mut v = Vec::new();
    let o = &setup.output;
    if o.video {
        v.push(("video", folder.join(format!("{base}.mp4"))));
    }
    if o.audio {
        v.push(("sound", folder.join(format!("{base}.m4a"))));
    }
    if o.captions {
        v.push(("captions", folder.join(format!("{base}.srt"))));
    }
    if o.guide {
        v.push(("guide", folder.join(format!("{base} - guide.html"))));
    }
    v
}

fn needs_sound(setup: &Setup) -> bool {
    setup.output.video || setup.output.audio || setup.output.captions
}

/// Plan the video and work out what it would cost. Spends nothing.
pub fn look(paths: &Paths, setup: &Setup, ctx: &Context) -> Result<Look> {
    let mut plan = plan::build(setup, ctx)?;
    let (estimate, speech) = if needs_sound(setup) {
        let script = speech::script(&plan, &ctx.search);
        plan.warnings.extend(speech::notes(paths, &script));
        (speech::estimate(paths, &script), speech::heard(paths, &script))
    } else {
        (Estimate::default(), vec![Vec::new(); plan.scenes.len()])
    };
    let files = out_files(paths, setup, &plan).into_iter().map(|(_, p)| p.to_string_lossy().into_owned()).collect();
    Ok(Look { plan, estimate, speech, key: secrets::source(), files })
}

/// Throw away the kept recordings heard on one scene, so they are recorded
/// afresh the next time the video is made. For a recording that came out wrong.
/// Spends nothing itself.
pub fn record_again(paths: &Paths, setup: &Setup, ctx: &Context, scene: usize) -> Result<Forgotten> {
    let plan = plan::build(setup, ctx)?;
    if scene >= plan.scenes.len() {
        return Err(Error::setup(format!("There is no scene {scene}. This video has scenes 0 to {}.", plan.scenes.len() - 1)));
    }
    speech::forget(paths, &speech::script(&plan, &ctx.search), scene)
}

/// Credits left in the ElevenLabs account. Free to ask.
pub fn credits() -> Result<Option<Credits>> {
    match secrets::key()? {
        Some((key, _)) => Ok(Some(Client::from_env(&key)?.credits()?)),
        None => Ok(None),
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Voice,
    Sound,
    Stills,
    Video,
    Finishing,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub stage: Stage,
    /// A plain sentence: "Drawing bead 12 of 77".
    pub message: String,
    pub done: usize,
    pub total: usize,
    /// The whole job, 0 to 1.
    pub fraction: f32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MadeFile {
    /// "video", "sound", "captions" or "guide".
    pub kind: String,
    pub path: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Made {
    pub name: String,
    pub files: Vec<MadeFile>,
    /// Length of the video, in seconds.
    pub seconds: f32,
    /// Letters of new speech recorded this time.
    pub letters_recorded: u64,
    /// Credits that recording spent, by ElevenLabs' own count wherever it gave one.
    pub credits_spent: u64,
    pub pace: Vec<PaceNote>,
    /// How long one Hail Mary takes, the easiest way to judge the pace. 9.2 in the first video.
    pub hail_mary_seconds: Option<f32>,
    pub warnings: Vec<String>,
    /// How long the making took, in seconds.
    pub took: f32,
}

pub struct Request<'a> {
    pub paths: &'a Paths,
    pub setup: &'a Setup,
    pub ctx: &'a Context,
    /// How many credits of new speech the person has agreed to spend.
    pub approved_credits: u64,
    /// Where ElevenLabs is. Only tests change this.
    pub voice_service: Option<String>,
}

fn weight(stage: Stage, done: usize, total: usize, recording: bool) -> f32 {
    let (from, to) = match (stage, recording) {
        (Stage::Voice, _) => (0.0, 0.15),
        (Stage::Sound, true) => (0.15, 0.22),
        (Stage::Sound, false) => (0.0, 0.08),
        (Stage::Stills, true) => (0.22, 0.45),
        (Stage::Stills, false) => (0.08, 0.38),
        (Stage::Video, true) => (0.45, 0.95),
        (Stage::Video, false) => (0.38, 0.95),
        (Stage::Finishing, _) => (0.95, 1.0),
    };
    from + (to - from) * if total == 0 { 1.0 } else { done as f32 / total as f32 }
}

struct WorkDir(PathBuf);

impl Drop for WorkDir {
    fn drop(&mut self) {
        if std::env::var_os("PVC_KEEP_WORK").is_none() {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

pub fn make(req: &Request, on_progress: &mut dyn FnMut(Progress), cancel: &AtomicBool) -> Result<Made> {
    let started = Instant::now();
    let (paths, setup, ctx) = (req.paths, req.setup, req.ctx);
    let plan = plan::build(setup, ctx)?;
    let files = out_files(paths, setup, &plan);
    if files.is_empty() {
        return Err(Error::setup("This setup makes nothing: video, sound, captions and guide are all off. Turn one on under output."));
    }
    let mut warnings = plan.warnings.clone();
    let stopped = || cancel.load(Ordering::SeqCst);
    let settings = paths.settings();
    let work = WorkDir(paths.work().join(format!("job-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0))));
    std::fs::create_dir_all(&work.0)?;
    for (_, file) in &files {
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)?;
        }
    }
    let mut made_files: Vec<MadeFile> = Vec::new();
    let mut recorded = speech::Recorded::default();
    let mut pace: Vec<PaceNote> = Vec::new();
    let mut hail_mary_seconds = None;
    let mut seconds = plan.seconds;

    if needs_sound(setup) {
        let ffmpeg = tools::find_ffmpeg(&settings)?;
        // Find the browser before any credits are spent, so a missing program costs nothing.
        let browser = if setup.output.video { Some(tools::find_browser(&settings)?) } else { None };

        // 1. The voice.
        let script = speech::script(&plan, &ctx.search);
        if !script.missing.is_empty() {
            return Err(Error::setup(format!("These recordings could not be found: {}.", script.missing.join(", "))));
        }
        warnings.extend(speech::notes(paths, &script));
        let recording = speech::estimate(paths, &script).letters_to_record > 0;
        if recording {
            let Some((key, _)) = secrets::key()? else {
                return Err(Error::voice(
                    "This video needs a voice from ElevenLabs, and no API key has been saved yet. Add your key in the app under Settings. \
                     Or make it with no voice: turn the voice off.",
                ));
            };
            let client = match &req.voice_service {
                Some(base) => Client::with_base(&key, base)?,
                None => Client::from_env(&key)?,
            };
            // What each model charges, straight from ElevenLabs, so the number
            // agreed to is the number it will come to. Asking is free. If the
            // list cannot be had, the last one known is used.
            if let Err(e) = speech::models(paths, &client, false) {
                log::warn!("could not fetch the list of models, so the cost is worked out from what is already known: {e}");
            }
            let estimate = speech::estimate(paths, &script);
            if req.approved_credits < estimate.credits_to_record {
                return Err(Error::NeedsApproval(format!(
                    "Making this video would record {} letters of new speech with ElevenLabs. That spends about {} credits. Nothing has been spent yet.",
                    estimate.letters_to_record, estimate.credits_to_record
                )));
            }
            on_progress(Progress { stage: Stage::Voice, message: format!("Recording the voice: 0 of {}", estimate.pieces_to_record), done: 0, total: estimate.pieces_to_record, fraction: 0.0 });
            recorded = speech::record(paths, &client, &script, Some(req.approved_credits), |done, total| {
                on_progress(Progress { stage: Stage::Voice, message: format!("Recording the voice: {done} of {total}"), done, total, fraction: weight(Stage::Voice, done, total, true) });
                !stopped()
            })?;
        }

        // 2. The sound track.
        let (clips, notes) = speech::prepare(paths, &ffmpeg, &script, |done, total| {
            on_progress(Progress { stage: Stage::Sound, message: format!("Setting the pace of the voice: {done} of {total}"), done, total, fraction: weight(Stage::Sound, done, total, recording) });
            !stopped()
        })?;
        pace = notes;
        hail_mary_seconds = script.clips.iter().position(|c| c.lines.first().map(String::as_str) == Some("Hail Mary, full of grace,") && c.lines.len() == 8).map(|i| clips[i].seconds());
        let timeline: Timeline = audio::assemble(&plan, &script, &clips);
        seconds = timeline.seconds();
        let music = match setup.music.file.as_deref().filter(|f| !f.trim().is_empty()) {
            Some(name) => match plan::find_file(name, &ctx.search, &speech::SOUND_ENDINGS) {
                Some(file) => Some((file, setup.music.volume)),
                None => return Err(Error::setup(format!("The music file could not be found: {name}."))),
            },
            None => None,
        };
        let sound = audio::master(&ffmpeg, &timeline, &work.0, music.as_ref().map(|(f, v)| (f.as_path(), *v)))?;
        let chapters = if setup.output.chapters {
            let file = work.0.join("chapters.txt");
            std::fs::write(&file, video::chapters(&plan, &timeline))?;
            Some(file)
        } else {
            None
        };
        if stopped() {
            return Err(Error::Cancelled);
        }

        for (kind, file) in &files {
            match *kind {
                "sound" => {
                    video::sound_only(&ffmpeg, &sound, chapters.as_deref(), file)?;
                    made_files.push(MadeFile { kind: kind.to_string(), path: file.to_string_lossy().into_owned() });
                }
                "captions" => {
                    std::fs::write(file, video::captions(&plan, &timeline))?;
                    made_files.push(MadeFile { kind: kind.to_string(), path: file.to_string_lossy().into_owned() });
                }
                "video" => {
                    // 3. The stills.
                    let spans = video::spans(&plan, &timeline);
                    let shots: Vec<Shot> = video::shots(&spans);
                    let taken = frames::shoot(browser.as_deref().expect("found above"), &work.0, &plan, &shots, &work.0.join("stills"), |done, total| {
                        on_progress(Progress { stage: Stage::Stills, message: format!("Drawing the beads: {done} of {total}"), done, total, fraction: weight(Stage::Stills, done, total, recording) });
                        !stopped()
                    })?;
                    let tight: Vec<String> = taken.iter().filter(|t| t.overflow && t.shot.line.is_none()).map(|t| plan.scenes[t.shot.scene].title.clone()).collect();
                    if !tight.is_empty() {
                        let mut names = tight.clone();
                        names.dedup();
                        warnings.push(format!("The words do not fully fit on screen for: {}. Make the words smaller, or pick a bigger screen.", names.join(", ")));
                    }
                    let stills: BTreeMap<Shot, PathBuf> = taken.into_iter().map(|t| (t.shot, t.file)).collect();

                    // 4. The video.
                    let picture = video::encode(&ffmpeg, &plan, &spans, &stills, &work.0, setup.output.quality, |done, total| {
                        on_progress(Progress { stage: Stage::Video, message: format!("Making the video: {done} of {total}"), done, total, fraction: weight(Stage::Video, done, total, recording) });
                        !stopped()
                    })?;
                    on_progress(Progress { stage: Stage::Finishing, message: "Putting the sound on the video".into(), done: 0, total: 1, fraction: 0.96 });
                    video::mux(&ffmpeg, &picture, &sound, chapters.as_deref(), file)?;
                    made_files.push(MadeFile { kind: kind.to_string(), path: file.to_string_lossy().into_owned() });
                }
                _ => {}
            }
        }
    }

    for (kind, file) in &files {
        if *kind == "guide" {
            on_progress(Progress { stage: Stage::Finishing, message: "Writing the tap-through guide".into(), done: 0, total: 1, fraction: 0.98 });
            std::fs::write(file, frames::guide_page(&plan)?)?;
            made_files.push(MadeFile { kind: kind.to_string(), path: file.to_string_lossy().into_owned() });
        }
    }
    on_progress(Progress { stage: Stage::Finishing, message: "Done".into(), done: 1, total: 1, fraction: 1.0 });
    log::info!("made \"{}\": {} file(s), {:.0} s long, {} letters recorded for {} credits", plan.name, made_files.len(), seconds, recorded.letters, recorded.credits);
    Ok(Made {
        name: plan.name.clone(),
        files: made_files,
        seconds,
        letters_recorded: recorded.letters,
        credits_spent: recorded.credits,
        pace,
        hail_mary_seconds,
        warnings,
        took: started.elapsed().as_secs_f32(),
    })
}
