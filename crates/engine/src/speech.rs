//! The voice.
//!
//! Each prayer is recorded once per voice and kept, so the Hail Mary that plays
//! 53 times is paid for once, and making the same video again costs nothing.
//!
//! A setup can ask for more than one recording of a prayer ("takes"), so the
//! repeats do not all sound alike. Each take is recorded once and kept too.
//!
//! The work happens in four steps, and only the third one spends credits:
//!
//! 1. `script`   — list every piece of speech the plan needs, without repeats
//! 2. `estimate` — see which pieces are already recorded, and what the rest would cost
//! 3. `record`   — ask ElevenLabs for the missing ones
//! 4. `prepare`  — trim each piece, bring it to the right pace, and time its lines
//!
//! What a piece costs depends on the model: most charge one credit a letter,
//! the fast ones half. `Rates` holds what ElevenLabs last said each one charges.

use crate::elevenlabs::{takes_direction, Alignment, Client, DictionaryInfo, ModelInfo, OwnSettings, SpeechRequest, VoiceInfo, MP3};
use crate::error::{Error, Result};
use crate::paths::Paths;
use crate::plan::{find_file, Plan, SceneKind, FIRST_VIDEO_PACE};
use crate::setup::{Normalize, SoundQuality, Voice};
use crate::tools;
use crate::variety::Turns;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// Samples per second for all sound inside the tool.
pub const SR: u32 = 48_000;
pub const SOUND_ENDINGS: [&str; 6] = ["mp3", "wav", "m4a", "ogg", "flac", "aac"];

/// A voice, and everything that changes how its recordings sound.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoiceRef {
    pub name: String,
    pub id: Option<String>,
    pub model: String,
    pub stability: Option<f32>,
    pub similarity: Option<f32>,
    pub style: Option<f32>,
    pub speaker_boost: Option<bool>,
    pub model_speed: Option<f32>,
    pub clone_strength: Option<f32>,
    pub seed: Option<u32>,
    pub language: Option<String>,
    pub normalize: Normalize,
    pub quality: SoundQuality,
    /// Pronunciation dictionaries, by name.
    pub dictionaries: Vec<String>,
    /// How to say it. Only the models that take direction are sent it.
    pub direction: Option<String>,
}

impl VoiceRef {
    /// A scene's voice settings, spoken by `name` (or by exactly `id`).
    fn of(v: &Voice, name: String, id: Option<String>) -> Self {
        let words = |s: &Option<String>| s.as_deref().map(str::trim).filter(|t| !t.is_empty()).map(String::from);
        VoiceRef {
            name,
            id,
            model: v.model.trim().to_string(),
            stability: v.stability,
            similarity: v.similarity,
            style: v.style,
            speaker_boost: v.speaker_boost,
            model_speed: v.model_speed,
            clone_strength: v.clone_strength,
            seed: v.seed,
            language: words(&v.language),
            normalize: v.normalize,
            quality: v.quality,
            dictionaries: v.dictionaries(),
            direction: words(&v.direction),
        }
    }

    /// What the recordings are filed under: the exact id if one was given, else the name.
    fn tag(&self) -> String {
        match &self.id {
            Some(id) if !id.trim().is_empty() => format!("id:{}", id.trim()),
            _ => format!("name:{}", self.name.trim().to_lowercase()),
        }
    }
    pub fn label(&self) -> String {
        if self.name.trim().is_empty() {
            self.id.clone().unwrap_or_default()
        } else {
            self.name.trim().to_string()
        }
    }

    /// The settings added after the first video, and only the ones this voice
    /// uses, as name, value, name, value. A voice that uses none is filed
    /// exactly where it always was, so speech recorded before these settings
    /// existed is still found.
    ///
    /// A plain list in a fixed order, never a JSON object: the app and `pvc`
    /// can be built with objects that order their keys differently, and the
    /// two must file a recording under the same name.
    fn extras(&self) -> Vec<serde_json::Value> {
        let mut list = Vec::new();
        let mut add = |name: &str, value: serde_json::Value| list.extend([json!(name), value]);
        if let Some(v) = self.speaker_boost {
            add("speakerBoost", json!(v));
        }
        if let Some(v) = self.model_speed {
            add("modelSpeed", json!(v));
        }
        if let Some(v) = self.clone_strength {
            add("cloneStrength", json!(v));
        }
        if let Some(v) = self.seed {
            add("seed", json!(v));
        }
        if let Some(v) = &self.language {
            add("language", json!(v));
        }
        if self.normalize != Normalize::Auto {
            add("normalize", json!(self.normalize));
        }
        if self.quality != SoundQuality::Standard {
            add("quality", json!(self.quality));
        }
        if !self.dictionaries.is_empty() {
            add("dictionaries", json!(self.dictionaries.iter().map(|d| d.to_lowercase()).collect::<Vec<_>>()));
        }
        list
    }

    /// The words as they are sent: with the direction in front, for a model that takes it.
    fn sent(&self, words: &str) -> String {
        match self.direction.as_deref() {
            Some(d) if takes_direction(&self.model) => {
                // "reverent, slow" becomes one tag. Tags already written out are sent as they are.
                if d.starts_with('[') {
                    format!("{d} {words}")
                } else {
                    format!("[{d}] {words}")
                }
            }
            _ => words.to_string(),
        }
    }

    fn output_format(&self) -> &'static str {
        match self.quality {
            SoundQuality::Standard => MP3,
            SoundQuality::High => "mp3_44100_192",
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// Speech made by ElevenLabs.
    Speech { voice: VoiceRef, text: String },
    /// A sound file the person supplied.
    Recording { file: PathBuf },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Speed {
    /// Relative to the first video. 1 lands on its pace whatever the voice's own pace is.
    Pace(f32),
    /// Speed the sound up by exactly this much.
    Exact(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipSpec {
    pub source: Source,
    pub speed: Speed,
    /// The lines this piece covers, for timing each one.
    pub lines: Vec<String>,
    /// Which recording of these words by this voice, counting from 1. Always 1
    /// unless the setup asks for more than one take.
    pub take: u32,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PartKind {
    /// Naming the mystery.
    Announce,
    Prayer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub clip: usize,
    pub kind: PartKind,
    /// Which line of the scene this piece starts on.
    pub first_line: usize,
}

/// Every piece of sound a plan needs, each listed once, and which scene uses which.
#[derive(Clone, Debug, Default)]
pub struct Script {
    pub clips: Vec<ClipSpec>,
    /// One list for each scene of the plan, in order.
    pub scenes: Vec<Vec<Part>>,
    /// Recordings named in the setup that could not be found.
    pub missing: Vec<String>,
}

struct Builder<'a> {
    out: Script,
    index: HashMap<String, usize>,
    /// For each piece of speech, which take comes next.
    takes: HashMap<String, Turns>,
    search: &'a [PathBuf],
}

impl Builder<'_> {
    /// Add a piece, or find the one already there that sounds the same.
    fn add(&mut self, spec: ClipSpec) -> usize {
        let key = match &spec.source {
            Source::Speech { voice, text } => format!("S|{}|{:?}|{}|{text}", sound_of(voice), spec.speed, spec.take),
            Source::Recording { file } => format!("R|{}|{:?}", file.display(), spec.speed),
        };
        if let Some(i) = self.index.get(&key) {
            return *i;
        }
        self.out.clips.push(spec);
        self.index.insert(key, self.out.clips.len() - 1);
        self.out.clips.len() - 1
    }

    /// `how` is the scene's voice settings: how many takes, and in what order.
    /// `silent` is true where the voice has dropped out and the piece only keeps time.
    fn speech(&mut self, voice: VoiceRef, lines: &[String], speed: Speed, how: &Voice, silent: bool) -> usize {
        let text = voice.sent(&lines.join(" "));
        let ways = if how.fresh_each_time { "fresh".to_string() } else { format!("{}{}", how.takes.max(1), if how.shuffle { " mixed" } else { "" }) };
        let key = format!("{ways}|{}|{text}", sound_of(&voice));
        let turns = self.takes.entry(key.clone()).or_insert_with(|| if how.fresh_each_time { Turns::endless() } else { Turns::new(how.takes, how.shuffle, &key) });
        // A silent piece reuses a take that is already needed. Nobody should pay for a recording nobody hears.
        let take = 1 + if silent { turns.again() } else { turns.next() };
        self.add(ClipSpec { source: Source::Speech { voice, text }, speed, lines: lines.to_vec(), take })
    }

    fn recording(&mut self, name: &str, lines: &[String], speed: f32) -> Option<usize> {
        match find_file(name, self.search, &SOUND_ENDINGS) {
            Some(file) => Some(self.add(ClipSpec { source: Source::Recording { file }, speed: Speed::Exact(speed), lines: lines.to_vec(), take: 1 })),
            None => {
                if !self.out.missing.iter().any(|m| m == name) {
                    self.out.missing.push(name.to_string());
                }
                None
            }
        }
    }
}

/// Everything about a voice that changes how its recordings sound.
fn sound_of(voice: &VoiceRef) -> String {
    let extras = voice.extras();
    let extras = if extras.is_empty() { String::new() } else { format!("|{}", serde_json::Value::Array(extras)) };
    format!("{}|{}|{:?}|{:?}|{:?}{extras}", voice.tag(), voice.model, voice.stability, voice.similarity, voice.style)
}

/// What a voice's own pace is measured across: the voice, the model, and how
/// fast the model was told to speak.
fn pace_of(voice: &VoiceRef) -> String {
    match voice.model_speed {
        Some(speed) => format!("{}|{}|{speed}", voice.tag(), voice.model),
        None => format!("{}|{}", voice.tag(), voice.model),
    }
}

pub fn script(plan: &Plan, search: &[PathBuf]) -> Script {
    let mut b = Builder { out: Script::default(), index: HashMap::new(), takes: HashMap::new(), search };
    let named = |s: &Option<String>| s.as_deref().map(str::trim).filter(|r| !r.is_empty()).map(String::from);
    for scene in &plan.scenes {
        let mut parts: Vec<Part> = Vec::new();
        let v = &scene.voice;
        if scene.kind == SceneKind::Prayer && v.on {
            // With several voices taking turns, the plan has already picked who leads here.
            let lead = VoiceRef::of(v, scene.speaker.clone().unwrap_or_else(|| v.name.clone()), if scene.speaker.is_some() { None } else { v.id.clone() });
            let speed = v.exact_speedup.map(Speed::Exact).unwrap_or(Speed::Pace(v.speed));
            if let Some(text) = &scene.say_first {
                let lines = [text.clone()];
                let clip = match named(&v.announce_recording) {
                    Some(name) => b.recording(&name, &lines, v.recording_speed),
                    None => Some(b.speech(lead.clone(), &lines, speed, v, false)),
                };
                if let Some(clip) = clip {
                    parts.push(Part { clip, kind: PartKind::Announce, first_line: 0 });
                }
            }
            if !scene.lines.is_empty() {
                let split = scene.response_from.filter(|k| *k > 0 && *k < scene.lines.len());
                if let Some(name) = named(&v.recording) {
                    if let Some(clip) = b.recording(&name, &scene.lines, v.recording_speed) {
                        parts.push(Part { clip, kind: PartKind::Prayer, first_line: 0 });
                    }
                } else if let (Some(second), Some(k)) = (named(&v.respond), split) {
                    let clip = b.speech(lead.clone(), &scene.lines[..k], speed, v, scene.muted);
                    parts.push(Part { clip, kind: PartKind::Prayer, first_line: 0 });
                    let clip = b.speech(VoiceRef { name: second, id: None, ..lead.clone() }, &scene.lines[k..], speed, v, scene.muted);
                    parts.push(Part { clip, kind: PartKind::Prayer, first_line: k });
                } else {
                    let clip = b.speech(lead.clone(), &scene.lines, speed, v, scene.muted);
                    parts.push(Part { clip, kind: PartKind::Prayer, first_line: 0 });
                }
            }
        }
        b.out.scenes.push(parts);
    }
    b.out
}

fn hash(parts: &serde_json::Value) -> String {
    let digest = Sha256::digest(parts.to_string().as_bytes());
    digest.iter().take(16).map(|b| format!("{b:02x}")).collect()
}

fn letters(text: &str) -> u64 {
    text.chars().count() as u64
}

impl ClipSpec {
    /// The name this piece is filed under before any speed change.
    ///
    /// Recorded speech is paid for, so this must not change for a setup that
    /// has not changed. Anything new is added at the end, and only when used.
    fn raw_key(&self) -> String {
        match &self.source {
            Source::Speech { voice, text } => {
                let mut parts = vec![json!("speech"), json!(2), json!(voice.tag()), json!(voice.model), json!(voice.stability), json!(voice.similarity), json!(voice.style), json!(text)];
                // The first take is filed as it always was, so speech recorded before takes existed is still found.
                if self.take > 1 {
                    parts.extend([json!("take"), json!(self.take)]);
                }
                let extras = voice.extras();
                if !extras.is_empty() {
                    parts.push(json!("with"));
                    parts.extend(extras);
                }
                hash(&serde_json::Value::Array(parts))
            }
            Source::Recording { file } => {
                let meta = std::fs::metadata(file).ok();
                let len = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = meta.and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
                hash(&json!(["recording", file.to_string_lossy(), len, modified]))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct SpeechMeta {
    voice: String,
    voice_id: String,
    model: String,
    text: String,
    alignment: Option<Alignment>,
    /// The credits ElevenLabs said it charged for this piece.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cost: Option<u64>,
    /// ElevenLabs' own name for the request, for asking them about it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

// ───────────────────────── what each model charges ─────────────────────────

/// The models ElevenLabs offers, kept for a day so the list does not have to be
/// fetched every time. Asking for it is free.
pub fn models(paths: &Paths, client: &Client, fresh: bool) -> Result<Vec<ModelInfo>> {
    let file = paths.models_file();
    if !fresh && is_young(&file) {
        if let Some(list) = kept_models(paths).filter(|l| !l.is_empty()) {
            return Ok(list);
        }
    }
    let list = client.models()?;
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&file, serde_json::to_string(&list)?)?;
    Ok(list)
}

fn is_young(file: &Path) -> bool {
    std::fs::metadata(file).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age.as_secs() < 86_400)
}

/// The list as last fetched, however old. Reads the disk only.
fn kept_models(paths: &Paths) -> Option<Vec<ModelInfo>> {
    std::fs::read_to_string(paths.models_file()).ok().and_then(|t| serde_json::from_str(&t).ok())
}

/// What each model charges for a letter, from the list ElevenLabs gave last
/// time. Reading it never touches the network, so working out a cost stays
/// quick and free. `models` is what brings it up to date.
#[derive(Clone, Debug, Default)]
pub struct Rates {
    models: Vec<ModelInfo>,
}

impl Rates {
    pub fn kept(paths: &Paths) -> Self {
        Rates { models: kept_models(paths).unwrap_or_default() }
    }

    pub fn model(&self, id: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.id == id)
    }

    /// True once the list has been fetched at least once.
    pub fn known(&self) -> bool {
        !self.models.is_empty()
    }

    /// Credits for one letter. For a model not on the list this is a careful
    /// guess: half for the ones ElevenLabs sells at half price, one otherwise.
    pub fn per_letter(&self, model: &str) -> f32 {
        match self.model(model) {
            Some(m) => m.credits_per_letter,
            None if model.starts_with("eleven_flash_") || model.starts_with("eleven_turbo_v2") => 0.5,
            None => 1.0,
        }
    }

    /// Credits for one piece of speech of this many letters.
    pub fn cost(&self, model: &str, letters: u64) -> u64 {
        (letters as f64 * self.per_letter(model) as f64).ceil() as u64
    }
}

// ───────────────────────── 2. what is already recorded ─────────────────────────

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceCost {
    pub voice: String,
    /// Letters still to record.
    pub letters: u64,
    /// What recording them is expected to cost.
    pub credits: u64,
    pub pieces: usize,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    /// Letters ElevenLabs would be asked to record. Zero means no credits are spent.
    pub letters_to_record: u64,
    /// What recording them is expected to cost: letters times what each model
    /// charges. This is the number a person agrees to.
    pub credits_to_record: u64,
    /// Letters already recorded and kept from earlier.
    pub letters_kept: u64,
    pub pieces_to_record: usize,
    pub by_voice: Vec<VoiceCost>,
    /// The models that would do the recording.
    pub models: Vec<String>,
    /// True when what a model charges had to be guessed, because the list of
    /// models has not been fetched yet or does not have it.
    pub rates_guessed: bool,
    pub missing_recordings: Vec<String>,
}

fn is_kept(paths: &Paths, clip: &ClipSpec) -> bool {
    paths.speech().join(format!("{}.mp3", clip.raw_key())).is_file()
}

pub fn estimate(paths: &Paths, script: &Script) -> Estimate {
    let rates = Rates::kept(paths);
    let mut est = Estimate { missing_recordings: script.missing.clone(), ..Default::default() };
    let mut by_voice: BTreeMap<String, VoiceCost> = BTreeMap::new();
    let mut models: BTreeSet<String> = BTreeSet::new();
    for clip in &script.clips {
        let Source::Speech { voice, text } = &clip.source else { continue };
        let n = letters(text);
        if is_kept(paths, clip) {
            est.letters_kept += n;
        } else {
            let credits = rates.cost(&voice.model, n);
            est.letters_to_record += n;
            est.credits_to_record += credits;
            est.pieces_to_record += 1;
            est.rates_guessed |= rates.model(&voice.model).is_none();
            models.insert(voice.model.clone());
            let cost = by_voice.entry(voice.label()).or_insert_with(|| VoiceCost { voice: voice.label(), ..Default::default() });
            cost.letters += n;
            cost.credits += credits;
            cost.pieces += 1;
        }
    }
    est.by_voice = by_voice.into_values().collect();
    est.models = models.into_iter().collect();
    est
}

/// Things about the models worth telling the person before they pay: a model
/// ElevenLabs does not have, or a setting the chosen model does not use.
/// Both the app and `pvc` show these, so they name a setting the way a person
/// would say it, not by its path.
pub fn notes(paths: &Paths, script: &Script) -> Vec<String> {
    let rates = Rates::kept(paths);
    let mut out: Vec<String> = Vec::new();
    let mut say = |note: String| {
        if !out.contains(&note) {
            out.push(note);
        }
    };
    for clip in &script.clips {
        let Source::Speech { voice, .. } = &clip.source else { continue };
        let model = &voice.model;
        if voice.direction.is_some() && !takes_direction(model) {
            let name = rates.model(model).map(|m| m.name.as_str()).unwrap_or(model);
            say(format!("{name} does not take direction, so the direction is left out there. Eleven v3 and v4 take it."));
        }
        let Some(info) = rates.model(model) else {
            if rates.known() {
                say(format!("ElevenLabs does not list a speech model called \"{model}\" for your account. Pick another model."));
            }
            continue;
        };
        if voice.style.is_some() && !info.can_use_style {
            say(format!("{} does not use the style setting. Put it back, or ElevenLabs may refuse it.", info.name));
        }
        if voice.speaker_boost.is_some() && !info.can_use_speaker_boost {
            say(format!("{} does not use speaker boost. Put it back, or ElevenLabs may refuse it.", info.name));
        }
    }
    out
}

/// One piece of speech as it is heard on one scene.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Heard {
    pub part: PartKind,
    /// The voice, or the name of the person's own sound file.
    pub voice: String,
    /// True for a sound file the person supplied. There are no takes of those.
    pub own_recording: bool,
    /// Which recording of these words by this voice, counting from 1.
    pub take: u32,
    pub letters: u64,
    /// What recording it costs, or would cost again.
    pub credits: u64,
    /// The ElevenLabs model that speaks it. Empty for the person's own recording.
    pub model: String,
    /// Already recorded and kept.
    pub kept: bool,
    /// How many times this same recording is heard in the whole video.
    pub uses: usize,
    /// How many different recordings of these words by this voice the video uses.
    pub takes: u32,
}

/// For each scene of the plan, the pieces of speech heard on it.
pub fn heard(paths: &Paths, script: &Script) -> Vec<Vec<Heard>> {
    let rates = Rates::kept(paths);
    let mut uses = vec![0usize; script.clips.len()];
    for part in script.scenes.iter().flatten() {
        uses[part.clip] += 1;
    }
    let mut takes: HashMap<String, BTreeSet<u32>> = HashMap::new();
    let same_words = |c: &ClipSpec| match &c.source {
        Source::Speech { voice, text } => format!("{}|{text}", sound_of(voice)),
        Source::Recording { file } => file.display().to_string(),
    };
    for clip in &script.clips {
        takes.entry(same_words(clip)).or_default().insert(clip.take);
    }
    script
        .scenes
        .iter()
        .map(|parts| {
            parts
                .iter()
                .map(|part| {
                    let clip = &script.clips[part.clip];
                    let (voice, own_recording, letters, model, kept) = match &clip.source {
                        Source::Speech { voice, text } => (voice.label(), false, letters(text), voice.model.clone(), is_kept(paths, clip)),
                        Source::Recording { file } => (file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), true, 0, String::new(), true),
                    };
                    let credits = if own_recording { 0 } else { rates.cost(&model, letters) };
                    Heard { part: part.kind, voice, own_recording, take: clip.take, letters, credits, model, kept, uses: uses[part.clip], takes: takes[&same_words(clip)].len() as u32 }
                })
                .collect()
        })
        .collect()
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Forgotten {
    /// How many kept recordings were thrown away.
    pub pieces: usize,
    pub letters: u64,
    /// What it is expected to cost to record them again.
    pub credits: u64,
}

/// Throw away the kept recordings heard on one scene, so the next video records
/// them afresh. For a recording that came out wrong. Spends nothing itself; the
/// next `record` does, and it asks first like any other.
pub fn forget(paths: &Paths, script: &Script, scene: usize) -> Result<Forgotten> {
    let rates = Rates::kept(paths);
    let mut out = Forgotten::default();
    let mut done: BTreeSet<usize> = BTreeSet::new();
    for part in script.scenes.get(scene).map(Vec::as_slice).unwrap_or(&[]) {
        let clip = &script.clips[part.clip];
        let Source::Speech { voice, text } = &clip.source else { continue };
        if !done.insert(part.clip) {
            continue;
        }
        let key = clip.raw_key();
        let sound = paths.speech().join(format!("{key}.mp3"));
        if !sound.is_file() {
            continue;
        }
        // The sound goes first: its absence is what marks the piece as not recorded.
        std::fs::remove_file(&sound)?;
        let _ = std::fs::remove_file(paths.speech().join(format!("{key}.json")));
        // And everything worked out from it: the trimmed copy and each paced copy.
        if let Ok(dir) = std::fs::read_dir(paths.clips()) {
            for item in dir.flatten() {
                if item.file_name().to_string_lossy().starts_with(&key) {
                    let _ = std::fs::remove_file(item.path());
                }
            }
        }
        out.pieces += 1;
        out.letters += letters(text);
        out.credits += rates.cost(&voice.model, letters(text));
        log::warn!("a kept recording by voice \"{}\" (take {}) was thrown away at the person's request", voice.label(), clip.take);
    }
    Ok(out)
}

// ───────────────────────── 3. record what is missing ─────────────────────────

/// The voices in the account, kept for a day so the list does not have to be
/// fetched every time. Listing voices is free.
pub fn voices(paths: &Paths, client: &Client, fresh: bool) -> Result<Vec<VoiceInfo>> {
    let file = paths.voices_file();
    if !fresh && is_young(&file) {
        if let Some(list) = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str::<Vec<VoiceInfo>>(&t).ok()) {
            return Ok(list);
        }
    }
    let list = client.voices()?;
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&file, serde_json::to_string(&list)?)?;
    Ok(list)
}

fn voice_id(paths: &Paths, client: &Client, voice: &VoiceRef, known: &mut Option<Vec<VoiceInfo>>) -> Result<String> {
    if let Some(id) = voice.id.as_deref().map(str::trim).filter(|i| !i.is_empty()) {
        return Ok(id.to_string());
    }
    let wanted = voice.name.trim().to_lowercase();
    for fresh in [false, true] {
        if known.is_none() || fresh {
            *known = Some(voices(paths, client, fresh)?);
        }
        if let Some(v) = known.as_ref().unwrap().iter().find(|v| v.name.trim().to_lowercase() == wanted) {
            return Ok(v.id.clone());
        }
    }
    let names: Vec<&str> = known.as_ref().unwrap().iter().map(|v| v.name.as_str()).take(40).collect();
    Err(Error::voice(format!("There is no voice called \"{}\" in your ElevenLabs account. Your voices include: {}.", voice.name, names.join(", "))))
}

/// How a voice is set to speak at ElevenLabs, by its name: what is used
/// wherever a setup leaves a setting unset. Free to ask.
pub fn own_settings(paths: &Paths, client: &Client, name: &str) -> Result<OwnSettings> {
    let voice = VoiceRef { name: name.to_string(), ..Default::default() };
    client.voice_settings(&voice_id(paths, client, &voice, &mut None)?)
}

/// The account's dictionaries that go by these names, as (id, version), in the order asked for.
fn dictionary_ids(wanted: &[String], known: &[DictionaryInfo]) -> Result<Vec<(String, String)>> {
    wanted
        .iter()
        .map(|name| {
            let found = known.iter().find(|d| d.name.trim().eq_ignore_ascii_case(name.trim()) || d.id == name.trim());
            found.map(|d| (d.id.clone(), d.version.clone())).ok_or_else(|| {
                let names: Vec<&str> = known.iter().map(|d| d.name.as_str()).take(40).collect();
                Error::voice(if names.is_empty() {
                    format!("There is no pronunciation dictionary called \"{name}\" in your ElevenLabs account. It has none yet. Make one at elevenlabs.io, or put voice.dictionary back.")
                } else {
                    format!("There is no pronunciation dictionary called \"{name}\" in your ElevenLabs account. Your dictionaries are: {}.", names.join(", "))
                })
            })
        })
        .collect()
}

/// What one run of recording came to.
#[derive(Serialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Recorded {
    pub pieces: usize,
    pub letters: u64,
    /// Credits spent, by ElevenLabs' own count wherever it gave one.
    pub credits: u64,
}

/// Record every piece that is not already kept. Spends credits, and never
/// knowingly more than `limit`: it stops before any piece that would go over.
/// `on_done(i, n)` is called after each piece; return false from it to stop.
pub fn record(paths: &Paths, client: &Client, script: &Script, limit: Option<u64>, mut on_done: impl FnMut(usize, usize) -> bool) -> Result<Recorded> {
    std::fs::create_dir_all(paths.speech())?;
    let todo: Vec<&ClipSpec> = script.clips.iter().filter(|c| matches!(c.source, Source::Speech { .. }) && !is_kept(paths, c)).collect();
    let rates = Rates::kept(paths);

    // Everything that can be checked for free is checked before the first credit is spent.
    let mut known: Option<Vec<VoiceInfo>> = None;
    let mut dictionaries: Option<Vec<DictionaryInfo>> = None;
    for clip in &todo {
        let Source::Speech { voice, text } = &clip.source else { continue };
        voice_id(paths, client, voice, &mut known)?;
        if !voice.dictionaries.is_empty() {
            if dictionaries.is_none() {
                dictionaries = Some(client.dictionaries()?);
            }
            dictionary_ids(&voice.dictionaries, dictionaries.as_deref().unwrap_or(&[]))?;
        }
        if let Some(model) = rates.model(&voice.model).filter(|m| m.max_letters > 0 && letters(text) > m.max_letters) {
            return Err(Error::voice(format!(
                "One piece of speech is {} letters long, and {} takes {} at most. Pick another model for it, or split the text. It starts: \"{}…\"",
                letters(text),
                model.name,
                model.max_letters,
                text.chars().take(60).collect::<String>()
            )));
        }
    }

    let mut done = Recorded::default();
    // What the pieces recorded so far were expected to cost, to tell a small limit from a wrong guess.
    let mut planned = 0u64;
    for (i, clip) in todo.iter().enumerate() {
        let Source::Speech { voice, text } = &clip.source else { continue };
        let expected = rates.cost(&voice.model, letters(text));
        if let Some(limit) = limit.filter(|limit| done.credits + expected > *limit) {
            return Err(Error::NeedsApproval(format!(
                "Stopped before going over the {limit} credits agreed to. {} of {} pieces are recorded and kept, for {} credits.{}",
                done.pieces,
                todo.len(),
                done.credits,
                if done.credits > planned { " ElevenLabs charged more than was expected." } else { "" }
            )));
        }
        planned += expected;
        let id = voice_id(paths, client, voice, &mut known)?;
        let locators = dictionary_ids(&voice.dictionaries, dictionaries.as_deref().unwrap_or(&[]))?;
        let spoken = client.speak(&SpeechRequest {
            voice_id: &id,
            model: &voice.model,
            text,
            stability: voice.stability,
            similarity: voice.similarity,
            style: voice.style,
            speaker_boost: voice.speaker_boost,
            speed: voice.model_speed,
            clone_strength: voice.clone_strength,
            // Each take gets its own number. The same one would ask for the same recording again.
            seed: voice.seed.map(|seed| seed.wrapping_add(clip.take.saturating_sub(1))),
            language: voice.language.as_deref(),
            normalize: match voice.normalize {
                Normalize::Auto => None,
                Normalize::On => Some("on"),
                Normalize::Off => Some("off"),
            },
            dictionaries: &locators,
            output_format: voice.output_format(),
        })?;
        let key = clip.raw_key();
        let meta = SpeechMeta { voice: voice.label(), voice_id: id, model: voice.model.clone(), text: text.clone(), alignment: spoken.alignment, cost: spoken.cost, request_id: spoken.request_id };
        std::fs::write(paths.speech().join(format!("{key}.json")), serde_json::to_string(&meta)?)?;
        // The sound is written last: its presence is what marks the piece as recorded.
        std::fs::write(paths.speech().join(format!("{key}.mp3")), &spoken.audio)?;
        done.pieces += 1;
        done.letters += letters(text);
        done.credits += spoken.cost.unwrap_or(expected);
        match spoken.cost {
            Some(cost) => log::info!("recorded {} letters with voice \"{}\" and model {}: {cost} credits", letters(text), voice.label(), voice.model),
            None => log::info!("recorded {} letters with voice \"{}\" and model {}", letters(text), voice.label(), voice.model),
        }
        if !on_done(i + 1, todo.len()) {
            return Err(Error::Cancelled);
        }
    }
    Ok(done)
}

// ───────────────────────── 4. trim, pace and time ─────────────────────────

/// One piece of sound, ready to be laid into the video.
#[derive(Clone, Debug)]
pub struct Clip {
    pub samples: Vec<i16>,
    /// When each of its lines starts, in seconds from the start of the piece.
    pub line_starts: Vec<f32>,
}

impl Clip {
    pub fn seconds(&self) -> f32 {
        self.samples.len() as f32 / SR as f32
    }
}

/// How a voice was brought to pace, for telling the person.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaceNote {
    pub voice: String,
    /// The voice's own pace, in letters a second.
    pub natural: f32,
    /// How much it was sped up to reach speed 1.
    pub speedup: f32,
}

fn bytes_to_samples(bytes: &[u8]) -> Vec<i16> {
    bytes.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect()
}

fn samples_to_bytes(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

pub fn read_pcm(path: &Path) -> Result<Vec<i16>> {
    Ok(bytes_to_samples(&std::fs::read(path)?))
}

pub fn write_pcm(path: &Path, samples: &[i16]) -> Result<()> {
    std::fs::write(path, samples_to_bytes(samples))?;
    Ok(())
}

/// Turn any sound file into plain samples: one channel, 48,000 a second.
fn decode(ffmpeg: &Path, file: &Path) -> Result<Vec<i16>> {
    let out = tools::run(
        tools::command(ffmpeg).args(["-v", "error", "-i"]).arg(file).args(["-vn", "-ac", "1", "-ar", &SR.to_string(), "-f", "s16le", "-c:a", "pcm_s16le", "-"]),
        "FFmpeg (reading a sound file)",
    )?;
    Ok(bytes_to_samples(&out.stdout))
}

/// Cut the quiet off both ends. Returns the samples and how much was cut from the front, in seconds.
fn trim(samples: &[i16]) -> (Vec<i16>, f32) {
    const THRESHOLD: i16 = 184; // -45 dB
    let Some(first) = samples.iter().position(|s| s.unsigned_abs() > THRESHOLD as u16) else { return (samples.to_vec(), 0.0) };
    let last = samples.iter().rposition(|s| s.unsigned_abs() > THRESHOLD as u16).unwrap_or(samples.len() - 1);
    // Keep a breath of room, as the first video did: 0.04 s before and 0.10 s after.
    let start = first.saturating_sub((0.04 * SR as f32) as usize);
    let end = (last + (0.10 * SR as f32) as usize).min(samples.len());
    (samples[start..end].to_vec(), start as f32 / SR as f32)
}

fn fade_edges(samples: &mut [i16]) {
    let fade_in = ((0.01 * SR as f32) as usize).min(samples.len() / 2);
    let fade_out = ((0.03 * SR as f32) as usize).min(samples.len() / 2);
    for i in 0..fade_in {
        samples[i] = (samples[i] as f32 * i as f32 / fade_in as f32) as i16;
    }
    let n = samples.len();
    for i in 0..fade_out {
        samples[n - 1 - i] = (samples[n - 1 - i] as f32 * i as f32 / fade_out as f32) as i16;
    }
}

fn change_tempo(ffmpeg: &Path, input: &Path, factor: f32) -> Result<Vec<i16>> {
    // One atempo step is kept between 0.5 and 2 for the best sound; chain steps beyond that.
    let mut steps: Vec<String> = Vec::new();
    let mut left = factor;
    while left > 2.0 {
        steps.push("atempo=2.0".into());
        left /= 2.0;
    }
    while left < 0.5 {
        steps.push("atempo=0.5".into());
        left /= 0.5;
    }
    steps.push(format!("atempo={left:.5}"));
    let out = tools::run(
        tools::command(ffmpeg)
            .args(["-v", "error", "-f", "s16le", "-ar", &SR.to_string(), "-ac", "1", "-i"])
            .arg(input)
            .args(["-af", &steps.join(","), "-ar", &SR.to_string(), "-ac", "1", "-f", "s16le", "-c:a", "pcm_s16le", "-"]),
        "FFmpeg (changing the pace of the voice)",
    )?;
    Ok(bytes_to_samples(&out.stdout))
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct TrimMeta {
    trim_start: f32,
}

/// When each line starts, in seconds of the raw recording. `sent` is how many
/// letters were sent in all: more than the lines hold when a direction was put in front.
fn raw_line_starts(lines: &[String], sent: usize, alignment: Option<&Alignment>, raw_start: f32, raw_seconds: f32) -> Vec<f32> {
    let total: usize = lines.iter().map(|l| l.chars().count() + 1).sum::<usize>().saturating_sub(1).max(1);
    // Where the first line begins among the timed letters. ElevenLabs may time a
    // direction's letters or leave them out, so look for the words themselves.
    let lead = alignment.and_then(|a| {
        let spoken = lines.join(" ");
        match a.text.find(&spoken) {
            Some(at) => Some(a.text[..at].chars().count()),
            // No letters to look through: trust that every letter sent was timed.
            None if a.starts.len() >= sent => Some(sent.saturating_sub(total)),
            None => None,
        }
    });
    let mut starts = Vec::with_capacity(lines.len());
    let mut offset = 0usize;
    for line in lines {
        let t = match (alignment, lead) {
            // Exact: ElevenLabs told us when each letter is spoken.
            (Some(a), Some(lead)) if a.starts.len() >= lead + total => a.starts[lead + offset],
            // A guess: lines take time in proportion to their length.
            _ => raw_start + raw_seconds * offset as f32 / total as f32,
        };
        starts.push(t);
        offset += line.chars().count() + 1;
    }
    starts
}

/// Bring every piece to its final form. Returns the clips in the same order as
/// `script.clips`, and a note on how each voice was paced.
pub fn prepare(paths: &Paths, ffmpeg: &Path, script: &Script, mut on_done: impl FnMut(usize, usize) -> bool) -> Result<(Vec<Clip>, Vec<PaceNote>)> {
    std::fs::create_dir_all(paths.clips())?;
    struct Base {
        key: String,
        file: PathBuf,
        samples: Vec<i16>,
        trim_start: f32,
        alignment: Option<Alignment>,
    }
    let n = script.clips.len();

    // Trim each piece once and keep the result.
    let mut bases: Vec<Base> = Vec::with_capacity(n);
    for clip in &script.clips {
        let key = clip.raw_key();
        let (source, alignment) = match &clip.source {
            Source::Speech { voice, .. } => {
                let mp3 = paths.speech().join(format!("{key}.mp3"));
                if !mp3.is_file() {
                    return Err(Error::voice(format!("The speech for voice \"{}\" has not been recorded yet.", voice.label())));
                }
                let meta: SpeechMeta = std::fs::read_to_string(paths.speech().join(format!("{key}.json"))).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
                (mp3, meta.alignment)
            }
            Source::Recording { file } => (file.clone(), None),
        };
        let file = paths.clips().join(format!("{key}.pcm"));
        let meta_file = paths.clips().join(format!("{key}.json"));
        let kept = std::fs::read_to_string(&meta_file).ok().and_then(|t| serde_json::from_str::<TrimMeta>(&t).ok());
        let (samples, trim_start) = match kept {
            Some(meta) if file.is_file() => (read_pcm(&file)?, meta.trim_start),
            _ => {
                let (samples, trim_start) = trim(&decode(ffmpeg, &source)?);
                if samples.is_empty() {
                    return Err(Error::tool(format!("There is no sound in {}.", source.display())));
                }
                write_pcm(&file, &samples)?;
                std::fs::write(&meta_file, serde_json::to_string(&TrimMeta { trim_start })?)?;
                (samples, trim_start)
            }
        };
        bases.push(Base { key, file, samples, trim_start, alignment });
    }

    // Measure each voice's own pace across everything it says in this video. A
    // piece counts once for every time it is heard, so the Hail Mary, said 53
    // times, weighs 53 times. That way the whole video lands on the pace asked for.
    let mut uses = vec![0u64; n];
    for part in script.scenes.iter().flatten() {
        uses[part.clip] += 1;
    }
    let mut natural: BTreeMap<String, (u64, f32, String)> = BTreeMap::new();
    for (i, (clip, base)) in script.clips.iter().zip(&bases).enumerate() {
        if let Source::Speech { voice, text } = &clip.source {
            let times = uses[i].max(1);
            let e = natural.entry(pace_of(voice)).or_insert((0, 0.0, voice.label()));
            e.0 += letters(text) * times;
            e.1 += base.samples.len() as f32 / SR as f32 * times as f32;
        }
    }

    let mut notes: Vec<PaceNote> = Vec::new();
    let mut clips: Vec<Clip> = Vec::with_capacity(n);
    for (i, (clip, base)) in script.clips.iter().zip(&bases).enumerate() {
        let factor = match (&clip.source, clip.speed) {
            (_, Speed::Exact(x)) => x,
            (Source::Speech { voice, .. }, Speed::Pace(p)) => {
                let (count, seconds, label) = &natural[&pace_of(voice)];
                let own = *count as f32 / seconds.max(0.1);
                let speedup = (FIRST_VIDEO_PACE / own).clamp(0.5, 4.0);
                if !notes.iter().any(|n| &n.voice == label) {
                    notes.push(PaceNote { voice: label.clone(), natural: own, speedup });
                }
                (speedup * p).clamp(0.5, 4.0)
            }
            (Source::Recording { .. }, Speed::Pace(p)) => p,
        };
        let mut samples = if (factor - 1.0).abs() < 0.005 {
            base.samples.clone()
        } else {
            let kept = paths.clips().join(format!("{}-x{:04}.pcm", base.key, (factor * 1000.0).round() as u32));
            if kept.is_file() {
                read_pcm(&kept)?
            } else {
                let s = change_tempo(ffmpeg, &base.file, factor)?;
                write_pcm(&kept, &s)?;
                s
            }
        };
        fade_edges(&mut samples);
        let raw_seconds = base.samples.len() as f32 / SR as f32;
        let seconds = samples.len() as f32 / SR as f32;
        let sent = match &clip.source {
            Source::Speech { text, .. } => text.chars().count(),
            Source::Recording { .. } => 0,
        };
        let line_starts = raw_line_starts(&clip.lines, sent, base.alignment.as_ref(), base.trim_start, raw_seconds)
            .into_iter()
            .map(|t| ((t - base.trim_start) / factor).clamp(0.0, seconds))
            .collect();
        clips.push(Clip { samples, line_starts });
        if !on_done(i + 1, n) {
            return Err(Error::Cancelled);
        }
    }
    Ok((clips, notes))
}

/// How much recorded speech is being kept, in bytes.
pub fn kept_bytes(paths: &Paths) -> u64 {
    let size = |dir: PathBuf| -> u64 { std::fs::read_dir(dir).map(|d| d.flatten().filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum()).unwrap_or(0) };
    size(paths.speech()) + size(paths.clips())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{build, Context};
    use crate::setup::from_doc;

    fn plan(doc: serde_json::Value) -> Plan {
        let ctx = Context { today: chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(), search: vec![] };
        build(&from_doc(&doc).unwrap(), &ctx).unwrap()
    }

    fn temp(tag: &str) -> Paths {
        let dir = std::env::temp_dir().join(format!("pvc-speech-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Paths::new(dir)
    }

    #[test]
    fn each_prayer_is_recorded_once_per_voice() {
        let p = plan(json!({ "prayers": { "mysteries": "luminous" } }));
        let s = script(&p, &[]);
        // Eight prayers and five mysteries named aloud. The Hail Mary is one piece, used 53 times.
        assert_eq!(s.clips.len(), 13);
        let hail_mary = s.clips.iter().position(|c| c.lines.first().map(String::as_str) == Some("Hail Mary, full of grace,")).unwrap();
        let uses = s.scenes.iter().flatten().filter(|part| part.clip == hail_mary).count();
        assert_eq!(uses, 53);
        assert_eq!(s.scenes.len(), p.scenes.len());
        assert!(s.scenes[0].is_empty() && s.scenes.last().unwrap().is_empty());
    }

    #[test]
    fn a_second_voice_only_costs_what_it_says() {
        let p = plan(json!({ "prayers": { "mysteries": "luminous" }, "rules": [{ "for": { "prayer": "ourfather" }, "set": { "voice": { "name": "Other" } } }] }));
        let s = script(&p, &[]);
        // The Our Father is now said by "Other". The mysteries are named on the Our
        // Father's bead, so "Other" names them too. Nothing is recorded twice.
        assert_eq!(s.clips.len(), 13);
        let est = estimate(&temp("second"), &s);
        let other = est.by_voice.iter().find(|v| v.voice == "Other").unwrap();
        assert_eq!(other.pieces, 6);
        assert_eq!(est.by_voice.len(), 2);
    }

    #[test]
    fn several_takes_of_a_prayer_go_round() {
        let p = plan(json!({ "prayers": { "mysteries": "luminous" }, "voice": { "takes": 3 } }));
        let s = script(&p, &[]);
        let takes_of = |first_line: &str| -> Vec<u32> {
            s.scenes.iter().flatten().filter(|part| s.clips[part.clip].lines.first().map(String::as_str) == Some(first_line)).map(|part| s.clips[part.clip].take).collect()
        };
        // 53 Hail Marys, three recordings, in turn.
        let hail = takes_of("Hail Mary, full of grace,");
        assert_eq!(hail.len(), 53);
        assert_eq!(hail[..7], [1, 2, 3, 1, 2, 3, 1]);
        // A prayer said twice only needs two recordings. One said once needs one.
        assert_eq!(takes_of("In the name of the Father,"), [1, 2]);
        assert_eq!(takes_of("I believe in God, the Father almighty,"), [1]);
        // 8 prayers and 5 mysteries named: 13 pieces with one take. With three takes:
        // Hail Mary 3, Our Father 3, Glory Be 3, O My Jesus 3, Sign of the Cross 2, and 8 said once.
        assert_eq!(s.clips.len(), 3 + 3 + 3 + 3 + 2 + 8);
        let one = estimate(&temp("takes-1"), &script(&plan(json!({ "prayers": { "mysteries": "luminous" } })), &[]));
        let three = estimate(&temp("takes-3"), &s);
        assert!(three.letters_to_record > one.letters_to_record && three.letters_to_record < one.letters_to_record * 3);

        // What is heard on a bead says which take it is and how many there are.
        let h = heard(&temp("takes-h"), &s);
        let bead = p.scenes.iter().position(|sc| sc.bead.as_deref() == Some("d1b2")).unwrap();
        assert_eq!((h[bead][0].take, h[bead][0].takes, h[bead][0].kept), (2, 3, false));
        assert_eq!(h[bead][0].uses, 18, "53 Hail Marys over three takes: 18, 18 and 17");
    }

    #[test]
    fn one_take_is_filed_where_it_always_was() {
        let voice = VoiceRef { name: "Imperial - Male".into(), model: "m".into(), ..Default::default() };
        let clip = |take| ClipSpec { source: Source::Speech { voice: voice.clone(), text: "Amen.".into() }, speed: Speed::Pace(1.0), lines: vec!["Amen.".into()], take };
        assert_eq!(clip(1).raw_key(), hash(&json!(["speech", 2, "name:imperial - male", "m", null, null, null, "Amen."])));
        assert_eq!(clip(2).raw_key(), hash(&json!(["speech", 2, "name:imperial - male", "m", null, null, null, "Amen.", "take", 2])));
        assert_ne!(clip(1).raw_key(), clip(2).raw_key());
        assert_ne!(clip(2).raw_key(), clip(3).raw_key());
    }

    #[test]
    fn a_newer_setting_changes_the_filing_only_where_it_is_used() {
        // Kept speech is paid for. A setup that names the first video's model and
        // uses none of the newer settings must find its recordings exactly where
        // they were.
        let first = |more: serde_json::Value| -> Plan {
            let mut voice = json!({ "model": crate::setup::FIRST_MODEL });
            crate::setup::merge(&mut voice, &more);
            plan(json!({ "voice": voice }))
        };
        let plain = first(json!({}));
        let hail = |p: &Plan| -> String {
            let s = script(p, &[]);
            s.clips.iter().find(|c| c.lines.first().map(String::as_str) == Some("Hail Mary, full of grace,")).unwrap().raw_key()
        };
        let words = "Hail Mary, full of grace, the Lord is with thee. Blessed art thou among women, and blessed is the fruit of thy womb, Jesus. Holy Mary, Mother of God, pray for us sinners, now and at the hour of our death. Amen.";
        assert_eq!(hail(&plain), hash(&json!(["speech", 2, "name:imperial - male", "eleven_multilingual_v2", null, null, null, words])));

        // Each newer setting is a different sound, so a different recording.
        let mut seen = BTreeSet::from([hail(&plain)]);
        for voice in [
            json!({ "speakerBoost": false }),
            json!({ "modelSpeed": 1.1 }),
            json!({ "cloneStrength": 0.5 }),
            json!({ "seed": 7 }),
            json!({ "language": "en" }),
            json!({ "normalize": "on" }),
            json!({ "quality": "high" }),
            json!({ "dictionary": "Prayers" }),
        ] {
            assert!(seen.insert(hail(&first(voice.clone()))), "{voice} did not change where the recording is filed");
        }
        // A setup that names no model gets Eleven v4, which is a different
        // recording again. So is a direction, for a model that takes one.
        let usual = plan(json!({}));
        assert_eq!(hail(&usual), hail(&plan(json!({ "voice": { "model": "eleven_v4" } }))));
        assert_eq!(hail(&usual), hash(&json!(["speech", 2, "name:imperial - male", "eleven_v4", null, null, null, words])));
        assert!(seen.insert(hail(&usual)));
        assert!(seen.insert(hail(&plan(json!({ "voice": { "direction": "reverent" } })))));
        // And each is filed under a name that is pinned here, like the first video's.
        let seeded = first(json!({ "seed": 7, "quality": "high", "dictionary": "Prayers" }));
        assert_eq!(
            hail(&seeded),
            hash(&json!(["speech", 2, "name:imperial - male", "eleven_multilingual_v2", null, null, null, words, "with", "seed", 7, "quality", "high", "dictionaries", ["prayers"]]))
        );
        // Setting one back to what ElevenLabs does anyway is the same as leaving it out.
        assert_eq!(hail(&first(json!({ "normalize": "auto", "quality": "standard" }))), hail(&plain));
    }

    #[test]
    fn direction_goes_in_front_only_for_models_that_take_it() {
        assert!(takes_direction("eleven_v3") && takes_direction("eleven_v4") && takes_direction("eleven_v4_turbo") && takes_direction("eleven_v12"));
        assert!(!takes_direction("eleven_multilingual_v2") && !takes_direction("eleven_flash_v2_5") && !takes_direction("eleven_turbo_v2") && !takes_direction("eleven_v2"));

        let sent = |voice: serde_json::Value| -> (String, u64) {
            let p = plan(json!({ "voice": voice }));
            let s = script(&p, &[]);
            let est = estimate(&temp("direction"), &s);
            let clip = s.clips.iter().find(|c| c.lines.first().map(String::as_str) == Some("Glory be to the Father,")).unwrap();
            let Source::Speech { text, .. } = &clip.source else { panic!() };
            (text.clone(), est.letters_to_record)
        };
        let (plain, plain_letters) = sent(json!({ "model": "eleven_v4" }));
        assert!(plain.starts_with("Glory be to the Father,"));
        let (directed, directed_letters) = sent(json!({ "model": "eleven_v4", "direction": "reverent, unhurried" }));
        assert_eq!(directed, format!("[reverent, unhurried] {plain}"));
        // The direction is sent with every piece, so it is counted in the cost.
        assert!(directed_letters > plain_letters);
        // Tags a person wrote out are sent as they are.
        assert!(sent(json!({ "model": "eleven_v3", "direction": "[calm] [slowly]" })).0.starts_with("[calm] [slowly] Glory be"));
        // The usual model takes direction, so a setup that names no model gets it too.
        assert_eq!(sent(json!({ "direction": "reverent, unhurried" })).0, directed);
        // An older model would say the direction aloud, so it never gets one.
        assert_eq!(sent(json!({ "model": "eleven_multilingual_v2", "direction": "reverent" })).0, plain);
        let p = plan(json!({ "voice": { "model": "eleven_multilingual_v2", "direction": "reverent" } }));
        let notes = notes(&temp("direction-notes"), &script(&p, &[]));
        assert!(notes.len() == 1 && notes[0].contains("does not take direction"), "{notes:?}");
    }

    fn keep_models(paths: &Paths) {
        let models = json!([
            { "id": "eleven_multilingual_v2", "name": "Eleven Multilingual v2", "description": "", "creditsPerLetter": 1.0, "maxLetters": 10000, "canUseStyle": true, "canUseSpeakerBoost": true, "languages": [] },
            { "id": "eleven_flash_v2_5", "name": "Eleven Flash v2.5", "description": "", "creditsPerLetter": 0.5, "maxLetters": 40000, "canUseStyle": false, "canUseSpeakerBoost": false, "languages": [] },
            { "id": "eleven_v3", "name": "Eleven v3", "description": "", "creditsPerLetter": 1.0, "maxLetters": 300, "canUseStyle": false, "canUseSpeakerBoost": false, "languages": [] },
            { "id": "eleven_dear", "name": "Eleven Dear", "description": "", "creditsPerLetter": 2.0, "maxLetters": 0, "canUseStyle": true, "canUseSpeakerBoost": true, "languages": [] }
        ]);
        std::fs::create_dir_all(paths.models_file().parent().unwrap()).unwrap();
        std::fs::write(paths.models_file(), models.to_string()).unwrap();
    }

    #[test]
    fn the_cost_follows_what_the_model_charges() {
        let paths = temp("rates");
        let cost = |paths: &Paths, model: &str| estimate(paths, &script(&plan(json!({ "prayers": { "mysteries": "luminous" }, "voice": { "model": model } })), &[]));

        // Before the list of models has ever been fetched: one credit a letter, and it says it is a guess.
        let guess = cost(&paths, "eleven_multilingual_v2");
        assert_eq!(guess.credits_to_record, guess.letters_to_record);
        assert!(guess.rates_guessed);
        // The models ElevenLabs sells at half price are guessed at half.
        assert!(cost(&paths, "eleven_flash_v2_5").credits_to_record < guess.credits_to_record * 6 / 10);

        keep_models(&paths);
        let full = cost(&paths, "eleven_multilingual_v2");
        assert_eq!((full.credits_to_record, full.rates_guessed), (full.letters_to_record, false));
        assert_eq!(full.models, ["eleven_multilingual_v2"]);
        // Half price, rounded up piece by piece: never under by more than one credit a piece.
        let half = cost(&paths, "eleven_flash_v2_5");
        assert!(half.credits_to_record >= half.letters_to_record / 2 && half.credits_to_record <= half.letters_to_record / 2 + half.pieces_to_record as u64);
        assert_eq!(half.by_voice[0].credits, half.credits_to_record);
        assert_eq!(cost(&paths, "eleven_dear").credits_to_record, full.letters_to_record * 2);
        // A model that is not on the list is still a guess, and the person is told.
        let odd = plan(json!({ "voice": { "model": "eleven_made_up" } }));
        assert!(estimate(&paths, &script(&odd, &[])).rates_guessed);
        let notes = notes(&paths, &script(&odd, &[]));
        assert!(notes.len() == 1 && notes[0].contains("eleven_made_up"), "{notes:?}");
        // A setting the model does not use is pointed out once, however many pieces use it.
        let styled = plan(json!({ "voice": { "model": "eleven_flash_v2_5", "style": 0.3, "speakerBoost": true } }));
        let notes = super::notes(&paths, &script(&styled, &[]));
        assert_eq!(notes.len(), 2, "{notes:?}");
        // What one bead would cost to record again follows the model too.
        let p = plan(json!({ "voice": { "model": "eleven_dear" } }));
        let h = heard(&paths, &script(&p, &[]));
        let bead = p.scenes.iter().position(|sc| sc.bead.as_deref() == Some("d1b1")).unwrap();
        assert_eq!((h[bead][0].credits, h[bead][0].model.as_str()), (h[bead][0].letters * 2, "eleven_dear"));
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn recording_never_knowingly_goes_over_what_was_agreed() {
        let paths = temp("limit");
        let p = plan(json!({ "prayers": { "mysteries": "luminous" } }));
        let s = script(&p, &[]);
        let agreed = estimate(&paths, &s).credits_to_record;
        let mut server = mockito::Server::new();
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"imp","name":"Imperial - Male"}],"has_more":false}"#).create();
        // ElevenLabs charges far more than was expected for every piece.
        let speak = server
            .mock("POST", "/v1/text-to-speech/imp/with-timestamps")
            .match_query(mockito::Matcher::Any)
            .with_header("character-cost", &(agreed / 2).to_string())
            .with_body(r#"{"audio_base64":"AAEC"}"#)
            .expect_at_most(3)
            .create();
        let client = Client::with_base("k", &server.url()).unwrap();
        let err = record(&paths, &client, &s, Some(agreed), |_, _| true).unwrap_err();
        assert!(matches!(err, Error::NeedsApproval(_)), "{err}");
        assert!(err.to_string().contains("charged more than was expected"), "{err}");
        speak.assert();
        // What was recorded before it stopped is kept, so it is not paid for again.
        let after = estimate(&paths, &s);
        assert!(after.pieces_to_record < 13 && after.pieces_to_record >= 10, "{} pieces left", after.pieces_to_record);
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn the_real_cost_is_the_one_elevenlabs_reports() {
        let paths = temp("real-cost");
        let p = plan(json!({ "prayers": { "opening": false, "closing": false, "firstDecade": 1, "lastDecade": 1, "fatima": false }, "voice": { "announce": false } }));
        let s = script(&p, &[]);
        let mut server = mockito::Server::new();
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"imp","name":"Imperial - Male"}],"has_more":false}"#).create();
        server.mock("POST", "/v1/text-to-speech/imp/with-timestamps").match_query(mockito::Matcher::Any).with_header("character-cost", "10").with_header("request-id", "req-9").with_body(r#"{"audio_base64":"AAEC"}"#).create();
        let client = Client::with_base("k", &server.url()).unwrap();
        // Our Father, Hail Mary, Glory Be: three pieces at ten credits each, whatever their length.
        let done = record(&paths, &client, &s, None, |_, _| true).unwrap();
        assert_eq!((done.pieces, done.credits), (3, 30));
        assert!(done.letters > 300);
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn what_can_be_checked_is_checked_before_anything_is_spent() {
        let paths = temp("checks");
        keep_models(&paths);
        let mut server = mockito::Server::new();
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"imp","name":"Imperial - Male"}],"has_more":false}"#).create();
        server.mock("GET", "/v1/pronunciation-dictionaries").match_query(mockito::Matcher::Any).with_body(r#"{"pronunciation_dictionaries":[{"id":"d1","name":"Prayers","latest_version_id":"v2","latest_version_rules_num":3}],"has_more":false}"#).create();
        let never = server.mock("POST", "/v1/text-to-speech/imp/with-timestamps").match_query(mockito::Matcher::Any).with_body(r#"{"audio_base64":"AAEC"}"#).expect(0).create();
        let client = Client::with_base("k", &server.url()).unwrap();

        // The Creed is longer than this model takes in one piece.
        let long = script(&plan(json!({ "voice": { "model": "eleven_v3" } })), &[]);
        let err = record(&paths, &client, &long, None, |_, _| true).unwrap_err().to_string();
        assert!(err.contains("300 at most") && err.contains("Eleven v3"), "{err}");

        // A dictionary that is not in the account.
        let odd = script(&plan(json!({ "voice": { "dictionary": "Prayers + Latin" } })), &[]);
        let err = record(&paths, &client, &odd, None, |_, _| true).unwrap_err().to_string();
        assert!(err.contains("\"Latin\"") && err.contains("Prayers"), "{err}");
        never.assert();
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn a_setup_s_choices_reach_elevenlabs() {
        let paths = temp("sent");
        let p = plan(json!({
            "prayers": { "opening": false, "closing": false, "firstDecade": 1, "lastDecade": 1, "fatima": false },
            "voice": { "announce": false, "model": "eleven_flash_v2_5", "stability": 0.5, "speakerBoost": true, "modelSpeed": 1.1, "seed": 40, "language": "en", "normalize": "on", "quality": "high", "dictionary": "prayers", "takes": 2 },
            "rules": [{ "for": { "prayer": "ourfather" }, "set": { "voice": { "model": "eleven_v4", "direction": "reverent", "modelSpeed": null, "seed": null, "dictionary": null, "takes": 1 } } }]
        }));
        let s = script(&p, &[]);
        let mut server = mockito::Server::new();
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"imp","name":"Imperial - Male"}],"has_more":false}"#).create();
        server.mock("GET", "/v1/pronunciation-dictionaries").match_query(mockito::Matcher::Any).with_body(r#"{"pronunciation_dictionaries":[{"id":"d1","name":"Prayers","latest_version_id":"v2","latest_version_rules_num":3}],"has_more":false}"#).create();
        let high = mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_192".into());
        let flash = |seed: u32| {
            mockito::Matcher::PartialJson(json!({
                "model_id": "eleven_flash_v2_5",
                "voice_settings": { "stability": 0.5, "use_speaker_boost": true, "speed": 1.1 },
                "seed": seed,
                "language_code": "en",
                "apply_text_normalization": "on",
                "pronunciation_dictionary_locators": [{ "pronunciation_dictionary_id": "d1", "version_id": "v2" }],
            }))
        };
        // Hail Mary and Glory Be, two takes each. The second take asks with the next seed.
        let first = server.mock("POST", "/v1/text-to-speech/imp/with-timestamps").match_query(high.clone()).match_body(flash(40)).with_body(r#"{"audio_base64":"AAEC"}"#).expect(2).create();
        let second = server.mock("POST", "/v1/text-to-speech/imp/with-timestamps").match_query(high.clone()).match_body(flash(41)).with_body(r#"{"audio_base64":"AAEC"}"#).expect(1).create();
        // The Our Father has its own model, a direction, and no seed.
        let ours = server
            .mock("POST", "/v1/text-to-speech/imp/with-timestamps")
            .match_query(high)
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::PartialJson(json!({ "model_id": "eleven_v4", "voice_settings": { "stability": 0.5, "use_speaker_boost": true } })),
                mockito::Matcher::Regex(r#""text":"\[reverent\] Our Father"#.into()),
            ]))
            .with_body(r#"{"audio_base64":"AAEC"}"#)
            .expect(1)
            .create();
        let client = Client::with_base("k", &server.url()).unwrap();
        let done = record(&paths, &client, &s, None, |_, _| true).unwrap();
        first.assert();
        second.assert();
        ours.assert();
        assert_eq!(done.pieces, 4);
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn mixed_up_takes_use_every_take_and_never_repeat_back_to_back() {
        let p = plan(json!({ "voice": { "takes": 4, "shuffle": true } }));
        let s = script(&p, &[]);
        let hail: Vec<u32> = s.scenes.iter().flatten().filter(|part| s.clips[part.clip].lines.first().map(String::as_str) == Some("Hail Mary, full of grace,")).map(|part| s.clips[part.clip].take).collect();
        assert_eq!(hail[..4], [1, 2, 3, 4]);
        assert!(hail.windows(2).all(|w| w[0] != w[1]), "{hail:?}");
        assert!(hail.chunks(4).skip(1).any(|c| c != [1, 2, 3, 4]), "{hail:?}");
        // The same setup always gives the same mix.
        let again: Vec<u32> = script(&p, &[]).scenes.iter().flatten().filter(|part| s.clips[part.clip].lines.first().map(String::as_str) == Some("Hail Mary, full of grace,")).map(|part| s.clips[part.clip].take).collect();
        assert_eq!(hail, again);
    }

    #[test]
    fn a_fresh_recording_every_time() {
        let p = plan(json!({ "voice": { "freshEachTime": true } }));
        let s = script(&p, &[]);
        // 75 prayers and 5 mysteries named, and no recording is heard twice.
        assert_eq!(s.clips.len(), 80);
        let mut uses = vec![0; s.clips.len()];
        s.scenes.iter().flatten().for_each(|part| uses[part.clip] += 1);
        assert!(uses.iter().all(|n| *n == 1));

        // Only the Hail Marys of one decade: the rest stay one recording each.
        let p = plan(json!({ "rules": [{ "for": { "mystery": 2, "prayer": "hailmary" }, "set": { "voice": { "freshEachTime": true } } }] }));
        assert_eq!(script(&p, &[]).clips.len(), 13 + 9);
    }

    #[test]
    fn silence_is_never_paid_for() {
        // The voice says three Hail Marys of each decade and then drops out. The
        // silent beads still need the prayer's length, and borrow a take for it.
        let p = plan(json!({ "voice": { "freshEachTime": true, "dropAfter": 3 } }));
        let s = script(&p, &[]);
        let hail = s.clips.iter().filter(|c| c.lines.first().map(String::as_str) == Some("Hail Mary, full of grace,")).count();
        assert_eq!(hail, 3 + 5 * 3);
    }

    #[test]
    fn voices_taking_turns_each_keep_their_own_recordings() {
        let p = plan(json!({ "prayers": { "mysteries": "luminous" }, "rules": [{ "for": { "prayer": "hailmary" }, "set": { "voice": { "name": "Ann + Ben" } } }] }));
        let s = script(&p, &[]);
        // One more piece than a single voice needs: the second voice's Hail Mary.
        assert_eq!(s.clips.len(), 14);
        let est = estimate(&temp("turns"), &s);
        let names: Vec<&str> = est.by_voice.iter().map(|v| v.voice.as_str()).collect();
        assert_eq!(names, ["Ann", "Ben", "Imperial - Male"]);
        assert_eq!(est.by_voice[0].pieces, 1);
        assert_eq!(est.by_voice[0].letters, est.by_voice[1].letters);
    }

    #[test]
    fn a_recording_can_be_thrown_away_and_made_again() {
        let paths = temp("forget");
        let p = plan(json!({ "prayers": { "mysteries": "luminous" }, "voice": { "takes": 2 } }));
        let s = script(&p, &[]);
        let mut server = mockito::Server::new();
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"imp","name":"Imperial - Male"}],"has_more":false}"#).create();
        server.mock("POST", "/v1/text-to-speech/imp/with-timestamps").match_query(mockito::Matcher::Any).with_body(r#"{"audio_base64":"AAEC"}"#).create();
        let client = Client::with_base("k", &server.url()).unwrap();
        record(&paths, &client, &s, None, |_, _| true).unwrap();
        assert_eq!(estimate(&paths, &s).pieces_to_record, 0);

        // The second Hail Mary of decade one is take 2. Throw that one away.
        let bead = p.scenes.iter().position(|sc| sc.bead.as_deref() == Some("d1b2")).unwrap();
        let gone = forget(&paths, &s, bead).unwrap();
        assert_eq!(gone.pieces, 1);
        let est = estimate(&paths, &s);
        assert_eq!((est.pieces_to_record, est.letters_to_record), (1, gone.letters));
        // Take 1 is untouched, and throwing the same one away twice does nothing.
        let h = heard(&paths, &s);
        let first = p.scenes.iter().position(|sc| sc.bead.as_deref() == Some("d1b1")).unwrap();
        assert!(h[first][0].kept && !h[bead][0].kept);
        assert_eq!(forget(&paths, &s, bead).unwrap(), Forgotten::default());
        // Recording again fills the hole and nothing else.
        assert_eq!(record(&paths, &client, &s, None, |_, _| true).unwrap().letters, gone.letters);
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn two_voices_take_turns() {
        let p = plan(json!({ "voice": { "respond": "Answer" } }));
        let s = script(&p, &[]);
        let scene = p.scenes.iter().position(|sc| sc.prayer.as_deref() == Some("hailmary")).unwrap();
        let parts = &s.scenes[scene];
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].first_line, 4);
        let Source::Speech { voice, text } = &s.clips[parts[1].clip].source else { panic!() };
        assert_eq!(voice.name, "Answer");
        assert_eq!(text, "Holy Mary, Mother of God, pray for us sinners, now and at the hour of our death. Amen.");
        // The Creed has no answering half, so the leader says all of it.
        let creed = p.scenes.iter().position(|sc| sc.prayer.as_deref() == Some("creed")).unwrap();
        assert_eq!(s.scenes[creed].len(), 1);
    }

    #[test]
    fn no_voice_means_nothing_to_record() {
        let s = script(&plan(json!({ "voice": { "on": false } })), &[]);
        assert!(s.clips.is_empty());
        assert_eq!(estimate(&temp("none"), &s), Estimate::default());
    }

    #[test]
    fn kept_speech_is_not_paid_for_again() {
        let paths = temp("kept");
        let p = plan(json!({ "prayers": { "mysteries": "luminous" } }));
        let s = script(&p, &[]);
        let before = estimate(&paths, &s);
        assert_eq!(before.pieces_to_record, 13);
        assert!((2400..3400).contains(&before.letters_to_record), "{} letters", before.letters_to_record);

        // Pretend ElevenLabs answered, and check the tool records each piece once.
        let mut server = mockito::Server::new();
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"imp","name":"Imperial - Male"}],"has_more":false}"#).create();
        let speak = server
            .mock("POST", "/v1/text-to-speech/imp/with-timestamps")
            .match_query(mockito::Matcher::Any)
            .with_body(r#"{"audio_base64":"AAEC"}"#)
            .expect(13)
            .create();
        let client = Client::with_base("k", &server.url()).unwrap();
        let spent = record(&paths, &client, &s, None, |_, _| true).unwrap();
        assert_eq!((spent.letters, spent.credits, spent.pieces), (before.letters_to_record, before.credits_to_record, 13));
        let after = estimate(&paths, &s);
        assert_eq!(after.letters_to_record, 0);
        assert_eq!(after.letters_kept, before.letters_to_record);
        // A second run asks for nothing.
        assert_eq!(record(&paths, &client, &s, None, |_, _| true).unwrap(), Recorded::default());
        speak.assert();
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn an_unknown_voice_is_named_with_the_ones_that_exist() {
        let paths = temp("unknown");
        let mut server = mockito::Server::new();
        server.mock("GET", "/v2/voices").match_query(mockito::Matcher::Any).with_body(r#"{"voices":[{"voice_id":"a","name":"Bella"}],"has_more":false}"#).create();
        let client = Client::with_base("k", &server.url()).unwrap();
        let s = script(&plan(json!({})), &[]);
        let err = record(&paths, &client, &s, None, |_, _| true).unwrap_err().to_string();
        assert!(err.contains("Imperial - Male") && err.contains("Bella"), "{err}");
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn quiet_is_cut_from_both_ends() {
        let mut s = vec![0i16; SR as usize]; // one second of quiet
        s.extend(std::iter::repeat_n(8000i16, SR as usize / 2)); // half a second of sound
        s.extend(std::iter::repeat_n(0i16, SR as usize));
        let (cut, start) = trim(&s);
        assert!((start - 0.96).abs() < 0.001, "{start}");
        assert!((cut.len() as f32 / SR as f32 - 0.64).abs() < 0.001);
    }

    #[test]
    fn lines_are_timed_from_the_letters() {
        let lines = vec!["Ab".to_string(), "Cd".to_string()];
        // "Ab Cd": five letters. The second line starts at the fourth letter.
        let a = Alignment { starts: vec![0.5, 0.6, 0.7, 0.9, 1.0], ends: vec![0.6, 0.7, 0.9, 1.0, 1.2], text: String::new() };
        assert_eq!(raw_line_starts(&lines, 5, Some(&a), 0.0, 2.0), [0.5, 0.9]);
        // With no timings, guess by length.
        let guess = raw_line_starts(&lines, 5, None, 0.0, 2.0);
        assert_eq!(guess[0], 0.0);
        assert!((guess[1] - 1.2).abs() < 0.001);

        // A direction went in front: "[x] Ab Cd" is nine letters. ElevenLabs may
        // time the direction's letters too, or leave them out. Either way the
        // lines are found by their own words.
        let times = |n: usize| -> Vec<f32> { (0..n).map(|i| i as f32 / 10.0).collect() };
        let with = Alignment { starts: times(9), ends: times(9), text: "[x] Ab Cd".into() };
        assert_eq!(raw_line_starts(&lines, 9, Some(&with), 0.0, 2.0), [0.4, 0.7]);
        let without = Alignment { starts: times(5), ends: times(5), text: "Ab Cd".into() };
        assert_eq!(raw_line_starts(&lines, 9, Some(&without), 0.0, 2.0), [0.0, 0.3]);
        // Speech kept from before the letters were stored: every letter sent was timed.
        let old = Alignment { starts: times(9), ends: times(9), text: String::new() };
        assert_eq!(raw_line_starts(&lines, 9, Some(&old), 0.0, 2.0), [0.4, 0.7]);
        // Timings that do not cover the words are not trusted.
        let short = Alignment { starts: times(3), ends: times(3), text: String::new() };
        assert_eq!(raw_line_starts(&lines, 5, Some(&short), 0.0, 2.0)[0], 0.0);
    }
}
