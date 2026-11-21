//! Making a video, start to finish.
//!
//! `look` is free and fast: it plans the video and says what it would cost.
//! `make` does the work. It will not spend credits unless it is told, in
//! numbers, how many credits have been agreed to, and it stops rather than go
//! over that number.
//!
//! There is one number: the voice and the music together (`Look::credits`).
//! Everything that spends is in `spend`, and nowhere else.

use crate::audio::{self, Timeline};
use crate::elevenlabs::{Client, Credits};
use crate::error::{Error, Result};
use crate::frames::{self, Shot};
use crate::music::{self, MadeMusic, Piece};
use crate::paths::{file_safe, Paths};
use crate::plan::{self, Context, Plan};
use crate::secrets::{self, KeySource};
use crate::setup::Setup;
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
    /// The voice: what there is to record and what that would cost.
    pub estimate: Estimate,
    /// Music ElevenLabs would be asked to make, and roughly what that would cost.
    pub music: music::Cost,
    /// The one number a person agrees to: the voice and the music together.
    /// Zero means making this video spends nothing.
    pub credits: u64,
    /// For each scene of the plan, the speech heard on it: which voice, which
    /// take, and whether it is already recorded.
    pub speech: Vec<Vec<Heard>>,
    /// Where the ElevenLabs key was found, if there is one.
    pub key: Option<KeySource>,
    /// The files `make` would write.
    pub files: Vec<String>,
}

/// The longest a file's name may be, without its ending. Long enough to say
/// everything, short enough for any folder.
const LONGEST_NAME: usize = 150;

/// The base name for the files, without an ending: the video's title, made
/// safe for a file. The title says the mysteries, the part, the voice, the
/// model and the screen, so one video does not overwrite a different one.
fn base_name(plan: &Plan) -> String {
    let name = file_safe(&plan.title);
    if name.chars().count() <= LONGEST_NAME {
        return name;
    }
    let cut: String = name.chars().take(LONGEST_NAME).collect();
    cut.trim_end_matches(|c| matches!(c, ' ' | '-' | '.' | ',' | '+' | '(')).to_string()
}

fn out_files(paths: &Paths, setup: &Setup, plan: &Plan) -> Vec<(&'static str, PathBuf)> {
    let folder = setup.output.folder.as_deref().filter(|f| !f.trim().is_empty()).map(PathBuf::from).unwrap_or_else(|| paths.output_folder());
    let base = base_name(plan);
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

/// The one number to agree to: what the voice would cost and what the music would.
fn in_all(estimate: &Estimate, music: &music::Cost) -> u64 {
    estimate.credits_to_record + music.credits
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
    plan.warnings.extend(music::notes(setup));
    let music = music::cost(paths, setup);
    let files = out_files(paths, setup, &plan).into_iter().map(|(_, p)| p.to_string_lossy().into_owned()).collect();
    Ok(Look { plan, credits: in_all(&estimate, &music), estimate, music, speech, key: secrets::source(), files })
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
    /// What was made, said in full. The title inside the files.
    pub title: String,
    pub files: Vec<MadeFile>,
    /// Length of the video, in seconds.
    pub seconds: f32,
    /// Letters of new speech recorded this time.
    pub letters_recorded: u64,
    /// Credits known to have been spent: the voice by ElevenLabs' own count
    /// wherever it gave one, and the music where its cost could be told.
    pub credits_spent: u64,
    /// The music made this time, and what is known of its cost. None when no
    /// music was made, which includes music already made and kept.
    pub music: Option<MadeMusic>,
    /// The same in a sentence, for showing to the person.
    pub music_said: Option<String>,
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
    /// How many credits the person has agreed to spend, on the voice and the music together.
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

/// What a making spent with ElevenLabs.
#[derive(Debug, Default)]
struct Spent {
    recorded: speech::Recorded,
    music: Option<MadeMusic>,
}

/// Saying what a video would spend, when nobody has agreed to it yet.
fn asking(estimate: &Estimate, piece: Option<&Piece>) -> String {
    let voice = estimate.credits_to_record;
    match piece {
        None => format!(
            "Making this video would record {} letters of new speech with ElevenLabs. That spends about {voice} credits. Nothing has been spent yet.",
            estimate.letters_to_record
        ),
        Some(p) if estimate.letters_to_record == 0 => format!(
            "Making this video would have ElevenLabs make {} of music. That spends about {} credits. It is a rough figure: ElevenLabs does not give the price before it makes music. Nothing has been spent yet.",
            minutes_words(p.minutes()),
            p.estimate()
        ),
        Some(p) => format!(
            "Making this video would record {} letters of new speech and make {} of music with ElevenLabs. That spends about {} credits in all: about {voice} for the voice and about {} for the music. \
             The music's part is a rough figure: ElevenLabs does not give the price before it makes music. Nothing has been spent yet.",
            estimate.letters_to_record,
            minutes_words(p.minutes()),
            voice + p.estimate(),
            p.estimate()
        ),
    }
}

/// "2 minutes", "1 minute", "half a minute", "1.5 minutes"
pub fn minutes_words(minutes: f32) -> String {
    let m = (minutes * 100.0).round() / 100.0;
    if (m - 1.0).abs() < 0.005 {
        "1 minute".into()
    } else if (m - 0.5).abs() < 0.005 {
        "half a minute".into()
    } else {
        format!("{m} minutes")
    }
}

/// Everything that costs credits happens here, and nowhere else: the voice
/// first, then the music. `approved` is the one number the person agreed to.
///
/// - Nothing is spent unless `approved` covers the voice and the music together.
/// - The voice stops before any piece that would go over `approved`.
/// - The music is one request that cannot be stopped half way, so it is not
///   started if what the voice really cost plus the music's rough cost would
///   go over. What was recorded is kept either way.
///
/// `piece` is the music still to be made, if any. Kept music is not passed in.
fn spend(paths: &Paths, client: &Client, script: &speech::Script, piece: Option<&Piece>, approved: u64, on_progress: &mut dyn FnMut(Progress), stopped: &dyn Fn() -> bool) -> Result<Spent> {
    let mut spent = Spent::default();
    let recording = speech::estimate(paths, script).letters_to_record > 0;
    if recording {
        // What each model charges, straight from ElevenLabs, so the number
        // agreed to is the number it will come to. Asking is free. If the
        // list cannot be had, the last one known is used.
        if let Err(e) = speech::models(paths, client, false) {
            log::warn!("could not fetch the list of models, so the cost is worked out from what is already known: {e}");
        }
    }
    let estimate = speech::estimate(paths, script);
    let music_cost = piece.map(Piece::estimate).unwrap_or(0);
    if approved < estimate.credits_to_record + music_cost {
        return Err(Error::NeedsApproval(asking(&estimate, piece)));
    }
    if recording {
        on_progress(Progress { stage: Stage::Voice, message: format!("Recording the voice: 0 of {}", estimate.pieces_to_record), done: 0, total: estimate.pieces_to_record, fraction: 0.0 });
        spent.recorded = speech::record(paths, client, script, Some(approved), |done, total| {
            on_progress(Progress { stage: Stage::Voice, message: format!("Recording the voice: {done} of {total}"), done, total, fraction: weight(Stage::Voice, done, total, true) });
            !stopped()
        })?;
    }
    if let Some(piece) = piece {
        if spent.recorded.credits + music_cost > approved {
            return Err(Error::NeedsApproval(format!(
                "Stopped before making the music, so as not to go over the {approved} credits agreed to. The voice cost {} credits, more than was expected, and the music would be about {music_cost} more. The speech is recorded and kept.",
                spent.recorded.credits
            )));
        }
        if stopped() {
            return Err(Error::Cancelled);
        }
        on_progress(Progress { stage: Stage::Voice, message: format!("Making {} of music. This can take a few minutes.", minutes_words(piece.minutes())), done: 0, total: 1, fraction: weight(Stage::Voice, 1, 1, true) });
        spent.music = Some(music::make(paths, client, piece)?);
    }
    Ok(spent)
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
    let mut spent = Spent::default();
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
        warnings.extend(music::notes(setup));
        // The music this video plays, if the setup describes some, and whether it still has to be made.
        let piece = Piece::wanted(setup);
        let piece_to_make = piece.as_ref().filter(|p| !p.is_kept(paths));
        // A sound file the person supplies is looked for now, so a wrong name costs nothing.
        let supplied = match setup.music.file.as_deref().filter(|f| !f.trim().is_empty()) {
            Some(name) => match plan::find_file(name, &ctx.search, &speech::SOUND_ENDINGS) {
                Some(file) => Some(file),
                None => return Err(Error::setup(format!("The music file could not be found: {name}."))),
            },
            None => None,
        };
        let new_speech = speech::estimate(paths, &script).letters_to_record > 0;
        let recording = new_speech || piece_to_make.is_some();
        if recording {
            let Some((key, _)) = secrets::key()? else {
                return Err(Error::voice(if new_speech {
                    "This video needs a voice from ElevenLabs, and no API key has been saved yet. Add your key in the app under Settings. \
                     Or make it with no voice: turn the voice off."
                } else {
                    "This video needs music made by ElevenLabs, and no API key has been saved yet. Add your key in the app under Settings. \
                     Or make it without: take the words out of the background sound."
                }));
            };
            let client = match &req.voice_service {
                Some(base) => Client::with_base(&key, base)?,
                None => Client::from_env(&key)?,
            };
            spent = spend(paths, &client, &script, piece_to_make, req.approved_credits, on_progress, &stopped)?;
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
        let music = match supplied {
            Some(file) => Some((file, setup.music.volume)),
            // Music from ElevenLabs: kept from before, or made a moment ago. Its end is joined to its start so it goes round gently.
            None => piece.as_ref().filter(|p| p.is_kept(paths)).map(|p| (music::loopable(&ffmpeg, &p.file(paths), &work.0), setup.music.volume)),
        };
        let sound = audio::master(&ffmpeg, &timeline, &work.0, music.as_ref().map(|(f, v)| (f.as_path(), *v)))?;
        // What each file says about itself: its title always, chapter marks if wanted.
        let tags = work.0.join("tags.txt");
        std::fs::write(&tags, video::tags(&plan, &timeline, setup.output.chapters))?;
        if stopped() {
            return Err(Error::Cancelled);
        }

        for (kind, file) in &files {
            match *kind {
                "sound" => {
                    video::sound_only(&ffmpeg, &sound, &tags, file)?;
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
                    video::mux(&ffmpeg, &picture, &sound, &tags, file)?;
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
    let recorded = spent.recorded;
    let music_charged = spent.music.as_ref().and_then(|m| m.charged).unwrap_or(0);
    log::info!("made \"{}\": {} file(s), {:.0} s long, {} letters recorded for {} credits", plan.title, made_files.len(), seconds, recorded.letters, recorded.credits);
    Ok(Made {
        name: plan.name.clone(),
        title: plan.title.clone(),
        files: made_files,
        seconds,
        letters_recorded: recorded.letters,
        credits_spent: recorded.credits + music_charged,
        music_said: spent.music.as_ref().map(MadeMusic::said),
        music: spent.music,
        pace,
        hail_mary_seconds,
        warnings,
        took: started.elapsed().as_secs_f32(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::from_doc;
    use serde_json::json;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    fn temp(tag: &str) -> Paths {
        let dir = std::env::temp_dir().join(format!("pvc-make-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Paths::new(dir)
    }

    /// One decade, with what `more` adds: three prayers, 644 letters of speech.
    fn decade(more: serde_json::Value) -> (Setup, speech::Script) {
        let mut doc = json!({ "prayers": { "opening": false, "closing": false, "firstDecade": 1, "lastDecade": 1, "fatima": false }, "voice": { "announce": false } });
        crate::setup::merge(&mut doc, &more);
        let setup = from_doc(&doc).unwrap();
        let ctx = Context { today: chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(), search: vec![] };
        let script = speech::script(&plan::build(&setup, &ctx).unwrap(), &[]);
        (setup, script)
    }

    const ORGAN: &str = "slow, quiet organ in an empty chapel";

    /// A stand-in ElevenLabs with one voice. Each time the account is asked,
    /// its credits used read the next of `readings`, and the last one after that.
    fn service(server: &mut mockito::Server, readings: &'static [i64]) {
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"imp","name":"Imperial - Male"}],"has_more":false}"#).create();
        let asked = Arc::new(AtomicUsize::new(0));
        server
            .mock("GET", "/v1/user/subscription")
            .with_body_from_request(move |_| {
                let n = asked.fetch_add(1, Ordering::SeqCst).min(readings.len() - 1);
                format!(r#"{{"character_count":{},"character_limit":100000}}"#, readings[n]).into_bytes()
            })
            .create();
    }

    /// Speech that ElevenLabs charges `cost` credits a piece for, expected `times` times.
    fn speech_mock(server: &mut mockito::Server, cost: u64, times: usize) -> mockito::Mock {
        server.mock("POST", "/v1/text-to-speech/imp/with-timestamps").match_query(mockito::Matcher::Any).with_header("character-cost", &cost.to_string()).with_body(r#"{"audio_base64":"AAEC"}"#).expect(times).create()
    }

    fn music_mock(server: &mut mockito::Server, times: usize) -> mockito::Mock {
        server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_body([7u8, 7, 7]).expect(times).create()
    }

    fn go(paths: &Paths, client: &Client, script: &speech::Script, piece: Option<&Piece>, approved: u64) -> Result<Spent> {
        spend(paths, client, script, piece, approved, &mut |_| {}, &|| false)
    }

    #[test]
    fn one_number_covers_the_voice_and_the_music() {
        let paths = temp("one-number");
        let number = |more: serde_json::Value| -> (u64, u64, u64) {
            let (setup, script) = decade(more);
            let (voice, music) = (speech::estimate(&paths, &script), music::cost(&paths, &setup));
            (voice.credits_to_record, music.credits, in_all(&voice, &music))
        };
        // The voice alone, as it always was.
        assert_eq!(number(json!({})), (644, 0, 644));
        // Music alone: a video with no generated voice still has something to agree to.
        assert_eq!(number(json!({ "voice": { "on": false }, "music": { "prompt": ORGAN, "minutes": 0.5 } })), (0, 450, 450));
        // Both: one number, the two added.
        assert_eq!(number(json!({ "music": { "prompt": ORGAN } })), (644, 1800, 2444));
        // A supplied file is not made by anyone, so it costs nothing.
        assert_eq!(number(json!({ "music": { "prompt": ORGAN, "file": "chant.mp3" } })), (644, 0, 644));

        // And the asking names what the number is for, and says the music's part is rough.
        let (setup, script) = decade(json!({ "music": { "prompt": ORGAN } }));
        let (voice, piece) = (speech::estimate(&paths, &script), Piece::wanted(&setup).unwrap());
        let both = asking(&voice, Some(&piece));
        assert!(both.contains("about 2444 credits in all") && both.contains("about 644 for the voice") && both.contains("about 1800 for the music") && both.contains("rough figure"), "{both}");
        let only_voice = asking(&voice, None);
        assert!(only_voice.contains("644 letters") && only_voice.contains("about 644 credits") && !only_voice.contains("music"), "{only_voice}");
        let only_music = asking(&Estimate::default(), Some(&piece));
        assert!(only_music.contains("2 minutes of music") && only_music.contains("about 1800 credits") && only_music.contains("rough figure") && !only_music.contains("speech"), "{only_music}");
        assert_eq!((minutes_words(1.0), minutes_words(0.5), minutes_words(1.5)), ("1 minute".to_string(), "half a minute".to_string(), "1.5 minutes".to_string()));
    }

    #[test]
    fn nothing_is_spent_unless_the_yes_covers_voice_and_music_together() {
        let paths = temp("gate");
        let (setup, script) = decade(json!({ "music": { "prompt": ORGAN, "minutes": 0.5 } }));
        let piece = Piece::wanted(&setup).unwrap();
        let mut server = mockito::Server::new();
        service(&mut server, &[0]);
        let (never_speech, never_music) = (speech_mock(&mut server, 1, 0), music_mock(&mut server, 0));
        let client = Client::with_base("k", &server.url()).unwrap();

        // A yes for the voice alone (644), or for the music alone (450), is not a yes for both (1,094).
        for approved in [0, 450, 644, 1093] {
            let err = go(&paths, &client, &script, Some(&piece), approved).unwrap_err();
            assert!(matches!(err, Error::NeedsApproval(_)), "{approved}: {err}");
            assert!(err.to_string().contains("about 1094 credits in all"), "{err}");
        }
        never_speech.assert();
        never_music.assert();
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn the_voice_then_the_music_and_each_is_kept() {
        let paths = temp("both");
        let (setup, script) = decade(json!({ "music": { "prompt": ORGAN, "minutes": 0.5 } }));
        let piece = Piece::wanted(&setup).unwrap();
        let mut server = mockito::Server::new();
        // The account read 2,000 before the music and 2,430 after: 430, not the rough 450.
        service(&mut server, &[2000, 2430]);
        let (speech_calls, music_calls) = (speech_mock(&mut server, 200, 3), music_mock(&mut server, 1));
        let client = Client::with_base("k", &server.url()).unwrap();
        let spent = go(&paths, &client, &script, Some(&piece), 1094).unwrap();
        speech_calls.assert();
        music_calls.assert();
        assert_eq!((spent.recorded.pieces, spent.recorded.credits), (3, 600));
        let made = spent.music.unwrap();
        assert_eq!((made.charged, made.known_by, made.estimate), (Some(430), music::KnownBy::Counted, 450));
        // Making the same video again asks for nothing.
        assert_eq!(speech::estimate(&paths, &script).credits_to_record, 0);
        assert_eq!(music::cost(&paths, &setup), music::Cost { wanted: true, kept: true, minutes: 0.5, credits: 0 });
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn the_music_is_not_started_when_the_voice_cost_more_than_expected() {
        let paths = temp("overrun");
        let (setup, script) = decade(json!({ "music": { "prompt": ORGAN, "minutes": 0.5 } }));
        let piece = Piece::wanted(&setup).unwrap();
        let mut server = mockito::Server::new();
        service(&mut server, &[0]);
        // 644 was expected for the voice. ElevenLabs charges 300 a piece: 900.
        let (speech_calls, never_music) = (speech_mock(&mut server, 300, 3), music_mock(&mut server, 0));
        let client = Client::with_base("k", &server.url()).unwrap();
        let err = go(&paths, &client, &script, Some(&piece), 1094).unwrap_err();
        speech_calls.assert();
        never_music.assert();
        assert!(matches!(err, Error::NeedsApproval(_)), "{err}");
        let said = err.to_string();
        assert!(said.contains("Stopped before making the music") && said.contains("1094 credits agreed to") && said.contains("The voice cost 900") && said.contains("about 450 more"), "{said}");
        // The speech is kept, so only the music is left to agree to.
        assert_eq!((speech::estimate(&paths, &script).credits_to_record, music::cost(&paths, &setup).credits), (0, 450));
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn music_alone_is_agreed_to_and_made_once() {
        let paths = temp("music-only");
        let (setup, script) = decade(json!({ "voice": { "on": false }, "music": { "prompt": ORGAN, "minutes": 0.5 } }));
        assert!(script.clips.is_empty());
        let piece = Piece::wanted(&setup).unwrap();
        let mut server = mockito::Server::new();
        service(&mut server, &[100, 550]);
        let (never_speech, music_calls) = (speech_mock(&mut server, 1, 0), music_mock(&mut server, 1));
        let client = Client::with_base("k", &server.url()).unwrap();
        assert!(matches!(go(&paths, &client, &script, Some(&piece), 449), Err(Error::NeedsApproval(_))));
        let spent = go(&paths, &client, &script, Some(&piece), 450).unwrap();
        never_speech.assert();
        music_calls.assert();
        assert_eq!((spent.recorded, spent.music.unwrap().charged), (speech::Recorded::default(), Some(450)));
        assert!(piece.is_kept(&paths));
        let _ = std::fs::remove_dir_all(&paths.data);
    }
}
