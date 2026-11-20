//! The voice.
//!
//! Each prayer is recorded once per voice and kept, so the Hail Mary that plays
//! 53 times is paid for once, and making the same video again costs nothing.
//!
//! The work happens in four steps, and only the third one spends credits:
//!
//! 1. `script`   — list every piece of speech the plan needs, without repeats
//! 2. `estimate` — see which pieces are already recorded
//! 3. `record`   — ask ElevenLabs for the missing ones
//! 4. `prepare`  — trim each piece, bring it to the right pace, and time its lines

use crate::elevenlabs::{Alignment, Client, SpeechRequest, VoiceInfo};
use crate::error::{Error, Result};
use crate::paths::Paths;
use crate::plan::{find_file, Plan, SceneKind, FIRST_VIDEO_PACE};
use crate::tools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// Samples per second for all sound inside the tool.
pub const SR: u32 = 48_000;
pub const SOUND_ENDINGS: [&str; 6] = ["mp3", "wav", "m4a", "ogg", "flac", "aac"];

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceRef {
    pub name: String,
    pub id: Option<String>,
    pub model: String,
    pub stability: Option<f32>,
    pub similarity: Option<f32>,
    pub style: Option<f32>,
}

impl VoiceRef {
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    search: &'a [PathBuf],
}

impl Builder<'_> {
    /// Add a piece, or find the one already there that sounds the same.
    fn add(&mut self, spec: ClipSpec) -> usize {
        let key = match &spec.source {
            Source::Speech { voice, text } => format!("S|{}|{}|{:?}|{:?}|{:?}|{:?}|{text}", voice.tag(), voice.model, voice.stability, voice.similarity, voice.style, spec.speed),
            Source::Recording { file } => format!("R|{}|{:?}", file.display(), spec.speed),
        };
        if let Some(i) = self.index.get(&key) {
            return *i;
        }
        self.out.clips.push(spec);
        self.index.insert(key, self.out.clips.len() - 1);
        self.out.clips.len() - 1
    }

    fn speech(&mut self, voice: VoiceRef, lines: &[String], speed: Speed) -> usize {
        self.add(ClipSpec { source: Source::Speech { voice, text: lines.join(" ") }, speed, lines: lines.to_vec() })
    }

    fn recording(&mut self, name: &str, lines: &[String], speed: f32) -> Option<usize> {
        match find_file(name, self.search, &SOUND_ENDINGS) {
            Some(file) => Some(self.add(ClipSpec { source: Source::Recording { file }, speed: Speed::Exact(speed), lines: lines.to_vec() })),
            None => {
                if !self.out.missing.iter().any(|m| m == name) {
                    self.out.missing.push(name.to_string());
                }
                None
            }
        }
    }
}

pub fn script(plan: &Plan, search: &[PathBuf]) -> Script {
    let mut b = Builder { out: Script::default(), index: HashMap::new(), search };
    let named = |s: &Option<String>| s.as_deref().map(str::trim).filter(|r| !r.is_empty()).map(String::from);
    for scene in &plan.scenes {
        let mut parts: Vec<Part> = Vec::new();
        let v = &scene.voice;
        if scene.kind == SceneKind::Prayer && v.on {
            let lead = VoiceRef { name: v.name.clone(), id: v.id.clone(), model: v.model.clone(), stability: v.stability, similarity: v.similarity, style: v.style };
            let speed = v.exact_speedup.map(Speed::Exact).unwrap_or(Speed::Pace(v.speed));
            if let Some(text) = &scene.say_first {
                let lines = [text.clone()];
                let clip = match named(&v.announce_recording) {
                    Some(name) => b.recording(&name, &lines, v.recording_speed),
                    None => Some(b.speech(lead.clone(), &lines, speed)),
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
                    let clip = b.speech(lead.clone(), &scene.lines[..k], speed);
                    parts.push(Part { clip, kind: PartKind::Prayer, first_line: 0 });
                    let clip = b.speech(VoiceRef { name: second, id: None, ..lead.clone() }, &scene.lines[k..], speed);
                    parts.push(Part { clip, kind: PartKind::Prayer, first_line: k });
                } else {
                    let clip = b.speech(lead.clone(), &scene.lines, speed);
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
    fn raw_key(&self) -> String {
        match &self.source {
            Source::Speech { voice, text } => hash(&json!(["speech", 2, voice.tag(), voice.model, voice.stability, voice.similarity, voice.style, text])),
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
}

// ───────────────────────── 2. what is already recorded ─────────────────────────

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceCost {
    pub voice: String,
    /// Letters still to record. One credit each, near enough.
    pub letters: u64,
    pub pieces: usize,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    /// Letters ElevenLabs would be asked to record. Zero means no credits are spent.
    pub letters_to_record: u64,
    /// Letters already recorded and kept from earlier.
    pub letters_kept: u64,
    pub pieces_to_record: usize,
    pub by_voice: Vec<VoiceCost>,
    pub missing_recordings: Vec<String>,
}

pub fn estimate(paths: &Paths, script: &Script) -> Estimate {
    let mut est = Estimate { missing_recordings: script.missing.clone(), ..Default::default() };
    let mut by_voice: BTreeMap<String, VoiceCost> = BTreeMap::new();
    for clip in &script.clips {
        let Source::Speech { voice, text } = &clip.source else { continue };
        let n = letters(text);
        if paths.speech().join(format!("{}.mp3", clip.raw_key())).is_file() {
            est.letters_kept += n;
        } else {
            est.letters_to_record += n;
            est.pieces_to_record += 1;
            let cost = by_voice.entry(voice.label()).or_insert_with(|| VoiceCost { voice: voice.label(), ..Default::default() });
            cost.letters += n;
            cost.pieces += 1;
        }
    }
    est.by_voice = by_voice.into_values().collect();
    est
}

// ───────────────────────── 3. record what is missing ─────────────────────────

/// The voices in the account, kept for a day so the list does not have to be
/// fetched every time. Listing voices is free.
pub fn voices(paths: &Paths, client: &Client, fresh: bool) -> Result<Vec<VoiceInfo>> {
    let file = paths.voices_file();
    if !fresh {
        let young = std::fs::metadata(&file).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age.as_secs() < 86_400);
        if young {
            if let Some(list) = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str::<Vec<VoiceInfo>>(&t).ok()) {
                return Ok(list);
            }
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

/// Record every piece that is not already kept. Spends credits. `on_done(i, n)`
/// is called after each piece; return false from it to stop.
pub fn record(paths: &Paths, client: &Client, script: &Script, mut on_done: impl FnMut(usize, usize) -> bool) -> Result<u64> {
    std::fs::create_dir_all(paths.speech())?;
    let todo: Vec<&ClipSpec> = script.clips.iter().filter(|c| matches!(c.source, Source::Speech { .. }) && !paths.speech().join(format!("{}.mp3", c.raw_key())).is_file()).collect();
    let mut known: Option<Vec<VoiceInfo>> = None;
    let mut spent = 0u64;
    for (i, clip) in todo.iter().enumerate() {
        let Source::Speech { voice, text } = &clip.source else { continue };
        let id = voice_id(paths, client, voice, &mut known)?;
        let spoken = client.speak(&SpeechRequest { voice_id: &id, model: &voice.model, text, stability: voice.stability, similarity: voice.similarity, style: voice.style })?;
        let key = clip.raw_key();
        let meta = SpeechMeta { voice: voice.label(), voice_id: id, model: voice.model.clone(), text: text.clone(), alignment: spoken.alignment };
        std::fs::write(paths.speech().join(format!("{key}.json")), serde_json::to_string(&meta)?)?;
        // The sound is written last: its presence is what marks the piece as recorded.
        std::fs::write(paths.speech().join(format!("{key}.mp3")), &spoken.audio)?;
        spent += letters(text);
        log::info!("recorded {} letters with voice \"{}\"", letters(text), voice.label());
        if !on_done(i + 1, todo.len()) {
            return Err(Error::Cancelled);
        }
    }
    Ok(spent)
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

/// When each line starts, in seconds of the raw recording.
fn raw_line_starts(lines: &[String], alignment: Option<&Alignment>, raw_start: f32, raw_seconds: f32) -> Vec<f32> {
    let total: usize = lines.iter().map(|l| l.chars().count() + 1).sum::<usize>().saturating_sub(1).max(1);
    let mut starts = Vec::with_capacity(lines.len());
    let mut offset = 0usize;
    for line in lines {
        let t = match alignment {
            // Exact: ElevenLabs told us when each letter is spoken.
            Some(a) if a.starts.len() >= total && offset < a.starts.len() => a.starts[offset],
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

    // Measure each voice's own pace across everything it says in this video.
    let mut natural: BTreeMap<String, (u64, f32, String)> = BTreeMap::new();
    for (clip, base) in script.clips.iter().zip(&bases) {
        if let Source::Speech { voice, text } = &clip.source {
            let e = natural.entry(format!("{}|{}", voice.tag(), voice.model)).or_insert((0, 0.0, voice.label()));
            e.0 += letters(text);
            e.1 += base.samples.len() as f32 / SR as f32;
        }
    }

    let mut notes: Vec<PaceNote> = Vec::new();
    let mut clips: Vec<Clip> = Vec::with_capacity(n);
    for (i, (clip, base)) in script.clips.iter().zip(&bases).enumerate() {
        let factor = match (&clip.source, clip.speed) {
            (_, Speed::Exact(x)) => x,
            (Source::Speech { voice, .. }, Speed::Pace(p)) => {
                let (count, seconds, label) = &natural[&format!("{}|{}", voice.tag(), voice.model)];
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
        let line_starts = raw_line_starts(&clip.lines, base.alignment.as_ref(), base.trim_start, raw_seconds)
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
        let spent = record(&paths, &client, &s, |_, _| true).unwrap();
        assert_eq!(spent, before.letters_to_record);
        let after = estimate(&paths, &s);
        assert_eq!(after.letters_to_record, 0);
        assert_eq!(after.letters_kept, before.letters_to_record);
        // A second run asks for nothing.
        assert_eq!(record(&paths, &client, &s, |_, _| true).unwrap(), 0);
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
        let err = record(&paths, &client, &s, |_, _| true).unwrap_err().to_string();
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
        let a = Alignment { starts: vec![0.5, 0.6, 0.7, 0.9, 1.0], ends: vec![0.6, 0.7, 0.9, 1.0, 1.2] };
        assert_eq!(raw_line_starts(&lines, Some(&a), 0.0, 2.0), [0.5, 0.9]);
        // With no timings, guess by length.
        let guess = raw_line_starts(&lines, None, 0.0, 2.0);
        assert_eq!(guess[0], 0.0);
        assert!((guess[1] - 1.2).abs() < 0.001);
    }
}
