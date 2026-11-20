//! The sound track: every prayer laid end to end with its pauses, then brought
//! to a normal listening level. Each scene's sound is padded to a whole number
//! of video frames, so sound and picture can never drift apart.

use crate::error::{Error, Result};
use crate::plan::{Plan, SceneKind};
use crate::room;
use crate::setup::Room;
use crate::speech::{write_pcm, Clip, PartKind, Script, SR};
use crate::tools;
use std::path::{Path, PathBuf};

/// The quiet between the leader's half of a prayer and the answer.
const TURN_GAP: f32 = 0.12;
const TARGET_LUFS: f64 = -17.0;
const PEAK_DB: f64 = -1.5;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneTiming {
    pub start_frame: u64,
    pub frames: u64,
    /// When each line starts, in seconds from the start of the scene.
    pub line_starts: Vec<f32>,
    /// When the mystery is named: start and end, in seconds from the start of the scene.
    pub announce: Option<(f32, f32)>,
    /// When the prayer itself is said.
    pub prayer: Option<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct Timeline {
    pub fps: u32,
    pub scenes: Vec<SceneTiming>,
    pub samples: Vec<i16>,
    /// False when the whole track is silence.
    pub has_sound: bool,
}

impl Timeline {
    pub fn frames(&self) -> u64 {
        self.scenes.last().map(|s| s.start_frame + s.frames).unwrap_or(0)
    }
    pub fn seconds(&self) -> f32 {
        self.frames() as f32 / self.fps as f32
    }
    pub fn start(&self, scene: usize) -> f32 {
        self.scenes[scene].start_frame as f32 / self.fps as f32
    }
}

fn silence(buf: &mut Vec<i16>, seconds: f32) {
    buf.resize(buf.len() + (seconds.max(0.0) * SR as f32).round() as usize, 0);
}

fn now(buf: &[i16]) -> f32 {
    buf.len() as f32 / SR as f32
}

/// A number from -1 to 1 for each scene. It looks like chance and is not: the
/// same scene always gets the same number, so a video comes out the same twice.
fn sway(scene: usize) -> f32 {
    let mut x = (scene as u64).wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 40) as f32 / (1u64 << 23) as f32 - 1.0
}

pub fn assemble(plan: &Plan, script: &Script, clips: &[Clip]) -> Timeline {
    let fps = plan.stage.fps as u64;
    let mut samples: Vec<i16> = Vec::new();
    let mut scenes: Vec<SceneTiming> = Vec::with_capacity(plan.scenes.len());
    let mut frame: u64 = 0;
    let mut has_sound = false;

    for (i, scene) in plan.scenes.iter().enumerate() {
        let mut buf: Vec<i16> = Vec::new();
        let mut timing = SceneTiming { start_frame: frame, ..Default::default() };
        let parts = script.scenes.get(i).map(Vec::as_slice).unwrap_or(&[]);
        if scene.kind != SceneKind::Prayer {
            silence(&mut buf, scene.seconds);
        } else if parts.is_empty() {
            // No voice here. The bead waits long enough to read the words.
            let per_line = scene.pace.seconds_per_line;
            timing.line_starts = (0..scene.lines.len()).map(|k| k as f32 * per_line).collect();
            silence(&mut buf, (scene.lines.len() as f32 * per_line).max(scene.pace.min_seconds));
        } else {
            silence(&mut buf, scene.pace.before);
            timing.line_starts = vec![0.0; scene.lines.len()];
            let mut said_prayer = false;
            for part in parts {
                let clip = &clips[part.clip];
                match part.kind {
                    PartKind::Announce => {
                        let start = now(&buf);
                        buf.extend_from_slice(&clip.samples);
                        has_sound = true;
                        timing.announce = Some((start, now(&buf)));
                        silence(&mut buf, scene.pace.announce_gap + scene.pace.think);
                    }
                    PartKind::Prayer => {
                        if said_prayer {
                            silence(&mut buf, TURN_GAP);
                        }
                        let start = now(&buf);
                        for (k, t) in clip.line_starts.iter().enumerate() {
                            if let Some(slot) = timing.line_starts.get_mut(part.first_line + k) {
                                *slot = start + t;
                            }
                        }
                        if scene.muted {
                            // The voice has dropped out. The bead keeps the time the prayer takes.
                            buf.resize(buf.len() + clip.samples.len(), 0);
                        } else {
                            buf.extend_from_slice(&clip.samples);
                            has_sound = true;
                        }
                        timing.prayer = Some((timing.prayer.map(|p| p.0).unwrap_or(start), now(&buf)));
                        said_prayer = true;
                    }
                }
            }
            silence(&mut buf, (scene.pace.after + sway(i) * scene.pace.vary).max(0.05));
        }
        // Round the scene up to whole frames, measured from the very start so nothing adds up.
        let n = ((buf.len() as u64 * fps).div_ceil(SR as u64)).max(1);
        frame += n;
        let target = (frame * SR as u64 / fps) as usize;
        samples.extend_from_slice(&buf);
        samples.resize(target, 0);
        timing.frames = n;
        scenes.push(timing);
    }
    Timeline { fps: plan.stage.fps, scenes, samples, has_sound }
}

fn measured_loudness(ffmpeg: &Path, pcm: &Path) -> Result<f64> {
    let out = tools::run(
        tools::command(ffmpeg)
            .args(["-hide_banner", "-nostats", "-f", "s16le", "-ar", &SR.to_string(), "-ac", "1", "-i"])
            .arg(pcm)
            .args(["-af", "loudnorm=print_format=json", "-f", "null", "-"]),
        "FFmpeg (measuring loudness)",
    )?;
    let err = String::from_utf8_lossy(&out.stderr);
    let (open, close) = (err.rfind('{'), err.rfind('}'));
    let json = match (open, close) {
        (Some(a), Some(b)) if b > a => &err[a..=b],
        _ => return Err(Error::tool("FFmpeg did not report how loud the voice is.")),
    };
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| Error::tool(format!("Could not read FFmpeg's loudness report: {e}")))?;
    let lufs = v["input_i"].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(f64::NEG_INFINITY);
    Ok(lufs)
}

/// How to ask this FFmpeg to lay a room's clap over the voice and change
/// nothing else. Left to itself the filter rescales the clap, which would
/// turn the voice down, and the way to stop that was renamed along the way.
fn plain_convolution(ffmpeg: &Path) -> Result<&'static str> {
    let help = tools::run(tools::command(ffmpeg).args(["-hide_banner", "-h", "filter=afir"]), "FFmpeg (checking it can add a room)")?;
    let help = String::from_utf8_lossy(&help.stdout);
    if help.contains("irnorm") {
        Ok("irnorm=-1:irlink=0")
    } else if help.contains("gtype") {
        Ok("gtype=none")
    } else {
        Err(Error::tool("This FFmpeg is too old to put the voice in a room. Get a newer FFmpeg, or set the room to none."))
    }
}

/// The voice's part of the filter graph when it stands in a room: raise it to
/// level, lay the room's clap over it, hold the peaks. `clap` is which input
/// the clap is, and `samples` how long the voice is. The echo rings into the
/// pauses, but never past the end: the sound stays exactly as long as the picture.
fn voice_in_room(gain: f64, louder_by: f64, plain: &str, limiter: &str, clap: usize, samples: usize) -> String {
    format!("[0:a]volume={:.2}dB[dry];[dry][{clap}:a]afir={plain}[wet];[wet]atrim=end_sample={samples},{limiter}", gain - louder_by)
}

/// Write the finished sound track as a WAV file in `work`.
/// `music` is a file to play underneath and how loud, 0 to 1.
/// `room` is the room the voice is put in. With none, the sound is what it always was.
pub fn master(ffmpeg: &Path, timeline: &Timeline, work: &Path, music: Option<(&Path, f32)>, room: &Room) -> Result<PathBuf> {
    std::fs::create_dir_all(work)?;
    let pcm = work.join("voice.pcm");
    write_pcm(&pcm, &timeline.samples)?;
    let out = work.join("sound.wav");

    // Raise the voice to a normal listening level. A limiter holds the few loud peaks under the ceiling.
    let mut voice_filter = String::from("anull");
    // With a room: the file that holds its clap, and the voice's part of the filter graph.
    let mut in_room: Option<(PathBuf, String)> = None;
    if timeline.has_sound {
        let lufs = measured_loudness(ffmpeg, &pcm)?;
        if lufs.is_finite() {
            let gain = (TARGET_LUFS - lufs).min(8.0);
            let limiter = format!("alimiter=limit={:.3}:attack=5:release=60:level=false", 10f64.powf(PEAK_DB / 20.0));
            voice_filter = format!("volume={gain:.2}dB,{limiter}");
            if let Some(clap) = room::impulse(room) {
                let file = work.join("room.f32");
                std::fs::write(&file, clap.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>())?;
                in_room = Some((file, voice_in_room(gain, room::makes_louder_by(room), plain_convolution(ffmpeg)?, &limiter, 1, timeline.samples.len())));
            }
        }
    }

    let mut cmd = tools::command(ffmpeg);
    cmd.args(["-y", "-v", "error", "-f", "s16le", "-ar", &SR.to_string(), "-ac", "1", "-i"]).arg(&pcm);
    if let Some((clap, _)) = &in_room {
        cmd.args(["-f", "f32le", "-ar", &SR.to_string(), "-ac", "1", "-i"]).arg(clap);
    }
    // The voice's part of the graph, and which input the music is. A room's clap takes a place in the line.
    let (voice, music_input) = match &in_room {
        Some((_, graph)) => (graph.clone(), 2),
        None => (format!("[0:a]{voice_filter}"), 1),
    };
    match music {
        Some((file, volume)) => {
            let total = timeline.seconds();
            let fade_out = (total - 3.0).max(0.0);
            cmd.args(["-stream_loop", "-1", "-i"]).arg(file).args([
                "-filter_complex",
                &format!(
                    "{voice},pan=stereo|c0=c0|c1=c0[v];\
                     [{music_input}:a]aresample={SR},aformat=channel_layouts=stereo,volume={:.3},afade=t=in:d=2,afade=t=out:st={fade_out:.3}:d=3[m];\
                     [v][m]amix=inputs=2:duration=first:dropout_transition=0:normalize=0[a]",
                    volume.clamp(0.0, 1.0)
                ),
                "-map",
                "[a]",
                "-t",
                &format!("{total:.4}"),
                "-ac",
                "2",
            ]);
        }
        None => match &in_room {
            Some((_, graph)) => {
                cmd.args(["-filter_complex", &format!("{graph}[a]"), "-map", "[a]", "-ac", "1"]);
            }
            None => {
                cmd.args(["-af", &voice_filter, "-ac", "1"]);
            }
        },
    }
    cmd.args(["-ar", &SR.to_string(), "-c:a", "pcm_s16le"]).arg(&out);
    tools::run(&mut cmd, "FFmpeg (finishing the sound)")?;
    let _ = std::fs::remove_file(&pcm);
    if let Some((clap, _)) = &in_room {
        let _ = std::fs::remove_file(clap);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{build, Context};
    use crate::setup::from_doc;
    use crate::speech::script;
    use serde_json::json;

    fn plan(doc: serde_json::Value) -> Plan {
        let ctx = Context { today: chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(), search: vec![] };
        build(&from_doc(&doc).unwrap(), &ctx).unwrap()
    }

    /// Stand-in sound: every piece is `seconds` long, with its lines spread evenly.
    fn fake_clips(s: &Script, seconds: f32) -> Vec<Clip> {
        s.clips
            .iter()
            .map(|c| Clip { samples: vec![1000; (seconds * SR as f32) as usize], line_starts: (0..c.lines.len()).map(|k| k as f32 * seconds / c.lines.len() as f32).collect() })
            .collect()
    }

    #[test]
    fn a_room_goes_between_raising_the_voice_and_holding_its_peaks() {
        // The voice is turned down by what the echo adds, the clap is the second
        // input, and the echo is cut off where the voice track ends.
        let graph = voice_in_room(4.0, 0.37, "irnorm=-1:irlink=0", "alimiter=limit=0.841", 1, 96_000);
        assert_eq!(graph, "[0:a]volume=3.63dB[dry];[dry][1:a]afir=irnorm=-1:irlink=0[wet];[wet]atrim=end_sample=96000,alimiter=limit=0.841");
    }

    #[test]
    fn sound_and_picture_stay_in_step() {
        let p = plan(json!({}));
        let s = script(&p, &[]);
        let t = assemble(&p, &s, &fake_clips(&s, 2.345));
        assert_eq!(t.scenes.len(), 77);
        // Every scene starts where the last one ended, and the sound is exactly as long as the frames.
        let mut frame = 0;
        for sc in &t.scenes {
            assert_eq!(sc.start_frame, frame);
            assert!(sc.frames >= 1);
            frame += sc.frames;
        }
        assert_eq!(t.samples.len() as u64, t.frames() * SR as u64 / 30);
        assert!(t.has_sound);
        // The opening title lasts four seconds, as in the first video.
        assert_eq!(t.scenes[0].frames, 120);
    }

    #[test]
    fn pauses_are_where_the_first_video_had_them() {
        let p = plan(json!({}));
        let s = script(&p, &[]);
        let t = assemble(&p, &s, &fake_clips(&s, 2.0));
        // A plain prayer: 0.30 s quiet, the prayer, 0.70 s quiet.
        let hail = p.scenes.iter().position(|sc| sc.prayer.as_deref() == Some("hailmary")).unwrap();
        assert_eq!(t.scenes[hail].frames, 90);
        assert_eq!(t.scenes[hail].prayer, Some((0.30, 2.30)));
        // The first decade's Our Father: quiet, the mystery named, 0.45 s, the prayer, quiet.
        let named = p.scenes.iter().position(|sc| sc.say_first.is_some()).unwrap();
        let sc = &t.scenes[named];
        assert_eq!(sc.announce, Some((0.30, 2.30)));
        assert_eq!(sc.prayer, Some((2.75, 4.75)));
        assert_eq!(sc.frames, ((5.45f32 * 30.0).ceil()) as u64);
        // The lines of the prayer are timed from where the prayer starts.
        assert!((sc.line_starts[0] - 2.75).abs() < 1e-4);
        assert!(sc.line_starts[10] > sc.line_starts[9]);
    }

    #[test]
    fn a_dropped_voice_keeps_the_time_but_makes_no_sound() {
        let p = plan(json!({ "voice": { "dropAfter": 1 }, "prayers": { "opening": false, "closing": false, "lastDecade": 1, "fatima": false } }));
        let s = script(&p, &[]);
        let t = assemble(&p, &s, &fake_clips(&s, 1.0));
        let first = p.scenes.iter().position(|sc| sc.bead.as_deref() == Some("d1b1")).unwrap();
        let second = first + 1;
        assert_eq!(t.scenes[first].frames, t.scenes[second].frames);
        let range = |i: usize| {
            let a = (t.scenes[i].start_frame * SR as u64 / 30) as usize;
            let b = a + (t.scenes[i].frames * SR as u64 / 30) as usize;
            &t.samples[a..b]
        };
        assert!(range(first).iter().any(|s| *s != 0));
        assert!(range(second).iter().all(|s| *s == 0));
    }

    #[test]
    fn with_no_voice_each_bead_waits_to_be_read() {
        let p = plan(json!({ "voice": { "on": false } }));
        let s = script(&p, &[]);
        let t = assemble(&p, &s, &[]);
        assert!(!t.has_sound);
        let hail = p.scenes.iter().position(|sc| sc.prayer.as_deref() == Some("hailmary")).unwrap();
        assert_eq!(t.scenes[hail].frames, 8 * 2 * 30, "eight lines, two seconds each");
        let sign = p.scenes.iter().position(|sc| sc.prayer.as_deref() == Some("sign")).unwrap();
        assert_eq!(t.scenes[sign].frames, 4 * 2 * 30);
        assert_eq!(t.scenes[hail].line_starts[3], 6.0);
    }
}
