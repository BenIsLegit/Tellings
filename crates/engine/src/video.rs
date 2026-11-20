//! Turning stills and sound into files a person can play.
//!
//! Each still is held for as long as its sound lasts, with a soft fade from the
//! one before. The pieces are then joined and the sound laid on top. This is how
//! the first video was made (`prototype/render.py`), with the same settings.

use crate::audio::Timeline;
use crate::error::{Error, Result};
use crate::frames::Shot;
use crate::plan::{Plan, SceneKind};
use crate::setup::Transition;
use crate::tools;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;

/// A still and how long it is held.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub shot: Shot,
    pub frames: u64,
    /// Frames of soft fade from the still before. 0 is a plain cut.
    pub fade: u64,
}

/// Work out which stills are needed and for how long. Most scenes are one still.
/// A scene that follows along is one still for each line.
pub fn spans(plan: &Plan, timeline: &Timeline) -> Vec<Span> {
    let fps = plan.stage.fps as f32;
    let mut out: Vec<Span> = Vec::new();
    for (i, scene) in plan.scenes.iter().enumerate() {
        let t = &timeline.scenes[i];
        let scene_fade = match scene.pace.transition {
            Transition::Fade => (scene.pace.fade * fps).round() as u64,
            Transition::Cut => 0,
        };
        let follow = scene.kind == SceneKind::Prayer && scene.words.follow && scene.show_lines && scene.lines.len() > 1 && t.line_starts.len() == scene.lines.len();
        if !follow {
            out.push(Span { shot: Shot { scene: i, line: None }, frames: t.frames, fade: scene_fade });
            continue;
        }
        // Where each line's still begins, in frames from the start of the scene.
        let mut starts: Vec<(Option<usize>, u64)> = Vec::new();
        let first = (t.line_starts[0] * fps).round() as u64;
        if t.announce.is_some() && first > 0 {
            // While the mystery is being named, no line is lit yet.
            starts.push((None, 0));
            starts.push((Some(0), first));
        } else {
            starts.push((Some(0), 0));
        }
        for k in 1..scene.lines.len() {
            let at = (t.line_starts[k] * fps).round() as u64;
            let after_last = starts.last().unwrap().1 + 1;
            starts.push((Some(k), at.max(after_last)));
        }
        starts.retain(|(_, at)| *at < t.frames);
        let line_fade = if scene_fade > 0 { 3 } else { 0 };
        for (j, (line, at)) in starts.iter().enumerate() {
            let end = starts.get(j + 1).map(|s| s.1).unwrap_or(t.frames);
            out.push(Span { shot: Shot { scene: i, line: *line }, frames: end - at, fade: if j == 0 { scene_fade } else { line_fade } });
        }
    }
    out
}

/// The stills a list of spans needs, each once.
pub fn shots(spans: &[Span]) -> Vec<Shot> {
    let mut v: Vec<Shot> = spans.iter().map(|s| s.shot).collect();
    v.sort();
    v.dedup();
    v
}

const TO_YUV: &str = "format=rgb24,scale=out_color_matrix=bt709:out_range=tv:flags=accurate_rnd+full_chroma_int,format=yuv420p,loop=-1:1";

fn encode_one(ffmpeg: &Path, fps: u32, quality: u8, cur: &Path, prev: Option<&Path>, span: &Span, first: bool, last: bool, out: &Path) -> Result<()> {
    let n = span.frames;
    let fade = span.fade.min(n.saturating_sub(1));
    let rate = fps.to_string();
    let mut cmd = tools::command(ffmpeg);
    cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);
    let mut graph = match prev.filter(|_| fade > 0 && !first) {
        Some(prev) => {
            cmd.args(["-framerate", &rate, "-i"]).arg(prev).args(["-framerate", &rate, "-i"]).arg(cur);
            format!(
                "[0]{TO_YUV},trim=end_frame={fade},setpts=PTS-STARTPTS[a];[1]{TO_YUV},trim=end_frame={n},setpts=PTS-STARTPTS[b];[a][b]xfade=transition=fade:duration={:.5}:offset=0",
                fade as f64 / fps as f64
            )
        }
        None => {
            cmd.args(["-framerate", &rate, "-i"]).arg(cur);
            format!("[0]{TO_YUV},trim=end_frame={n},setpts=PTS-STARTPTS")
        }
    };
    let seconds = n as f64 / fps as f64;
    if first {
        graph.push_str(&format!(",fade=t=in:st=0:d={:.3}", seconds.min(0.8)));
    }
    if last {
        let d = seconds.min(1.0);
        graph.push_str(&format!(",fade=t=out:st={:.3}:d={d:.3}", seconds - d));
    }
    graph.push_str("[v]");
    let part = out.with_extension("part.mp4");
    cmd.args(["-filter_complex", &graph, "-map", "[v]", "-frames:v", &n.to_string(), "-r", &rate])
        .args(["-c:v", "libx264", "-crf", &quality.to_string(), "-preset", "veryfast", "-pix_fmt", "yuv420p"])
        .args(["-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-color_range", "tv"])
        .args(["-video_track_timescale", &(fps * 512).to_string(), "-an"])
        .arg(&part);
    tools::run(&mut cmd, "FFmpeg (making a piece of the video)")?;
    std::fs::rename(&part, out)?;
    Ok(())
}

fn concat_line(path: &Path) -> String {
    format!("file '{}'\n", path.to_string_lossy().replace('\\', "/").replace('\'', "'\\''"))
}

/// Make the picture track: every span as its own short file, then all of them
/// joined. `on_done(i, n)` is called as pieces finish; return false to stop.
pub fn encode(ffmpeg: &Path, plan: &Plan, spans: &[Span], stills: &BTreeMap<Shot, PathBuf>, work: &Path, quality: u8, mut on_done: impl FnMut(usize, usize) -> bool) -> Result<PathBuf> {
    let dir = work.join("pieces");
    std::fs::create_dir_all(&dir)?;
    let fps = plan.stage.fps;
    let n = spans.len();
    let still = |shot: &Shot| stills.get(shot).ok_or_else(|| Error::other("A still is missing."));
    let files: Vec<PathBuf> = (0..n).map(|i| dir.join(format!("{i:05}.mp4"))).collect();
    for i in 0..n {
        still(&spans[i].shot)?;
    }

    let next = AtomicUsize::new(0);
    let stop = AtomicBool::new(false);
    let workers = std::thread::available_parallelism().map(|p| p.get() / 2).unwrap_or(2).clamp(1, 4);
    let (tx, rx) = mpsc::channel::<Result<()>>();
    let mut failure: Option<Error> = None;
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let tx = tx.clone();
            let (next, stop, files) = (&next, &stop, &files);
            scope.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= n || stop.load(Ordering::SeqCst) {
                    break;
                }
                let prev = if i > 0 { stills.get(&spans[i - 1].shot).map(PathBuf::as_path) } else { None };
                let result = encode_one(ffmpeg, fps, quality, &stills[&spans[i].shot], prev, &spans[i], i == 0, i == n - 1, &files[i]);
                if tx.send(result).is_err() {
                    break;
                }
            });
        }
        drop(tx);
        let mut done = 0;
        for result in rx {
            match result {
                Ok(()) => {
                    done += 1;
                    if !on_done(done, n) {
                        stop.store(true, Ordering::SeqCst);
                        failure.get_or_insert(Error::Cancelled);
                    }
                }
                Err(e) => {
                    stop.store(true, Ordering::SeqCst);
                    failure.get_or_insert(e);
                }
            }
        }
    });
    if let Some(e) = failure {
        return Err(e);
    }

    let list = work.join("pieces.txt");
    std::fs::write(&list, files.iter().map(|f| concat_line(f)).collect::<String>())?;
    let silent = work.join("picture.mp4");
    tools::run(
        tools::command(ffmpeg).args(["-y", "-hide_banner", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i"]).arg(&list).args(["-c", "copy"]).arg(&silent),
        "FFmpeg (joining the pieces)",
    )?;
    Ok(silent)
}

fn meta_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if matches!(c, '=' | ';' | '#' | '\\' | '\n') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Chapter marks in the form FFmpeg reads: one for each section of the prayer.
pub fn chapters(plan: &Plan, timeline: &Timeline) -> String {
    let fps = plan.stage.fps as u64;
    let ms = |frame: u64| frame * 1000 / fps;
    let marks: Vec<(u64, &str)> = plan.scenes.iter().enumerate().filter_map(|(i, s)| s.chapter.as_deref().map(|c| (timeline.scenes[i].start_frame, c))).collect();
    let mut out = format!(";FFMETADATA1\ntitle={}\n", meta_escape(&plan.name));
    for (j, (start, title)) in marks.iter().enumerate() {
        let end = marks.get(j + 1).map(|m| m.0).unwrap_or(timeline.frames());
        out.push_str(&format!("[CHAPTER]\nTIMEBASE=1/1000\nSTART={}\nEND={}\ntitle={}\n", ms(*start), ms(end), meta_escape(title)));
    }
    out
}

fn srt_time(seconds: f32) -> String {
    let ms = (seconds.max(0.0) * 1000.0).round() as u64;
    format!("{:02}:{:02}:{:02},{:03}", ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000)
}

/// Captions as a .srt file: every line of every prayer, timed to the voice.
pub fn captions(plan: &Plan, timeline: &Timeline) -> String {
    let mut out = String::new();
    let mut n = 0;
    let mut add = |start: f32, end: f32, text: &str| {
        if end > start && !text.trim().is_empty() {
            n += 1;
            out.push_str(&format!("{n}\n{} --> {}\n{}\n\n", srt_time(start), srt_time(end), text.trim()));
        }
    };
    for (i, scene) in plan.scenes.iter().enumerate() {
        if scene.kind != SceneKind::Prayer {
            continue;
        }
        let t = &timeline.scenes[i];
        let at = timeline.start(i);
        let length = t.frames as f32 / timeline.fps as f32;
        if let (Some((a, b)), Some(text)) = (t.announce, &scene.say_first) {
            add(at + a, at + b, text);
        }
        if t.line_starts.len() != scene.lines.len() {
            continue;
        }
        let end_of_prayer = t.prayer.map(|p| p.1).unwrap_or(length);
        for (k, line) in scene.lines.iter().enumerate() {
            let start = t.line_starts[k];
            let end = t.line_starts.get(k + 1).copied().unwrap_or(end_of_prayer);
            add(at + start, at + end.max(start + 0.2).min(length), line);
        }
    }
    out
}

/// Lay the sound on the picture and write the finished video.
pub fn mux(ffmpeg: &Path, picture: &Path, sound: &Path, chapters: Option<&Path>, out: &Path) -> Result<()> {
    let mut cmd = tools::command(ffmpeg);
    cmd.args(["-y", "-hide_banner", "-loglevel", "error", "-i"]).arg(picture).arg("-i").arg(sound);
    if let Some(c) = chapters {
        cmd.args(["-f", "ffmetadata", "-i"]).arg(c).args(["-map_metadata", "2"]);
    }
    cmd.args(["-map", "0:v", "-map", "1:a", "-c:v", "copy", "-c:a", "aac", "-b:a", "160k", "-movflags", "+faststart"]).arg(out);
    tools::run(&mut cmd, "FFmpeg (writing the video)")?;
    Ok(())
}

/// The sound on its own, with the same chapters.
pub fn sound_only(ffmpeg: &Path, sound: &Path, chapters: Option<&Path>, out: &Path) -> Result<()> {
    let mut cmd = tools::command(ffmpeg);
    cmd.args(["-y", "-hide_banner", "-loglevel", "error", "-i"]).arg(sound);
    if let Some(c) = chapters {
        cmd.args(["-f", "ffmetadata", "-i"]).arg(c).args(["-map_metadata", "1"]);
    }
    cmd.args(["-map", "0:a", "-c:a", "aac", "-b:a", "160k", "-movflags", "+faststart"]).arg(out);
    tools::run(&mut cmd, "FFmpeg (writing the sound file)")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::assemble;
    use crate::plan::{build, Context};
    use crate::setup::from_doc;
    use crate::speech::{script, Clip, Script, SR};
    use serde_json::json;

    fn made(doc: serde_json::Value) -> (Plan, Timeline) {
        let ctx = Context { today: chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(), search: vec![] };
        let p = build(&from_doc(&doc).unwrap(), &ctx).unwrap();
        let s: Script = script(&p, &[]);
        let clips: Vec<Clip> = s.clips.iter().map(|c| Clip { samples: vec![500; 4 * SR as usize], line_starts: (0..c.lines.len()).map(|k| k as f32 * 4.0 / c.lines.len() as f32).collect() }).collect();
        let t = assemble(&p, &s, &clips);
        (p, t)
    }

    #[test]
    fn one_still_for_each_scene_as_in_the_first_video() {
        let (p, t) = made(json!({}));
        let sp = spans(&p, &t);
        assert_eq!(sp.len(), 77);
        assert_eq!(shots(&sp).len(), 77);
        assert!(sp.iter().all(|s| s.fade == 9), "a 0.3 second fade at 30 frames a second");
        assert_eq!(sp.iter().map(|s| s.frames).sum::<u64>(), t.frames());
    }

    #[test]
    fn following_along_is_one_still_for_each_line() {
        let (p, t) = made(json!({ "words": { "follow": true }, "prayers": { "opening": false, "closing": false, "lastDecade": 1, "fatima": false } }));
        let sp = spans(&p, &t);
        // Intro, Our Father (11 lines, plus the time the mystery is named), ten Hail Marys (8 lines), Glory Be (7), outro.
        assert_eq!(sp.len(), 1 + (1 + 11) + 10 * 8 + 7 + 1);
        assert_eq!(sp.iter().map(|s| s.frames).sum::<u64>(), t.frames(), "the lines still add up to the whole video");
        assert!(sp.iter().all(|s| s.frames >= 1));
        let our_father: Vec<&Span> = sp.iter().filter(|s| s.shot.scene == 1).collect();
        assert_eq!(our_father[0].shot.line, None);
        assert_eq!(our_father[1].shot.line, Some(0));
        assert_eq!(our_father[1].fade, 3);
        // The same Hail Mary line on different beads is a different still: the gold bead has moved.
        assert_eq!(shots(&sp).len(), sp.len());
    }

    #[test]
    fn chapters_mark_each_mystery() {
        let (p, t) = made(json!({ "prayers": { "mysteries": "luminous" } }));
        let c = chapters(&p, &t);
        assert_eq!(c.matches("[CHAPTER]").count(), 8, "title, opening, five mysteries, closing");
        assert!(c.contains("title=II. The Wedding at Cana"));
        assert!(c.contains("START=0\n"));
        assert!(c.contains(&format!("END={}\n", t.frames() * 1000 / 30)));
    }

    #[test]
    fn captions_carry_every_line() {
        let (p, t) = made(json!({ "prayers": { "opening": false, "closing": false, "lastDecade": 1, "fatima": false } }));
        let c = captions(&p, &t);
        assert!(c.starts_with("1\n00:00:04,300 --> 00:00:08,300\nThe first Luminous Mystery. The Baptism in the Jordan.\n"), "{}", &c[..120]);
        assert_eq!(c.matches("Hail Mary, full of grace,").count(), 10);
        assert_eq!(c.matches(" --> ").count(), 1 + 11 + 80 + 7);
    }

    #[test]
    fn srt_times() {
        assert_eq!(srt_time(3725.5), "01:02:05,500");
        assert_eq!(srt_time(0.0), "00:00:00,000");
    }
}
