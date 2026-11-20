//! Background music made by ElevenLabs from a description in words.
//!
//! A piece is made once and kept, like speech, so a video that asks for the
//! same piece again costs nothing.
//!
//! What sets music apart is the price. ElevenLabs gives no way to ask it
//! before a piece is made, and does not say afterwards what it charged. So:
//!
//! - the cost shown beforehand is a rough figure (`CREDITS_A_MINUTE`), and is
//!   always called one;
//! - the cost reported afterwards is read off the account, the credits used
//!   before and after, and is only called a cost when that reading makes sense.
//!   A guess is never passed off as what was charged.

use crate::elevenlabs::{Client, MusicRequest, MUSIC_MODEL};
use crate::error::{Error, Result};
use crate::paths::Paths;
use crate::setup::Setup;
use crate::speech::{hash, SR};
use crate::tools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};

/// The only figure ElevenLabs publishes for music: "Eleven Music 900 credits
/// per minute", given as approximate. Its other pages price music differently,
/// and none says how a single piece is counted.
pub const CREDITS_A_MINUTE: u64 = 900;

/// How long the end of a piece is faded into its start, so it can go round.
const JOIN_SECONDS: f32 = 3.0;

/// One piece of music a setup asks ElevenLabs for.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub words: String,
    /// How long, in thousandths of a second.
    pub length_ms: u64,
    pub instrumental: bool,
    /// Which piece of these words. ElevenLabs cannot be asked for the same
    /// piece twice, so a new number is simply a new piece.
    pub take: u32,
}

impl Piece {
    /// The piece this setup's video would play, if it asks for one: there are
    /// words, no supplied file wins over them, and the video has sound to put
    /// the music under.
    pub fn wanted(setup: &Setup) -> Option<Piece> {
        if !(setup.output.video || setup.output.audio) {
            return None;
        }
        let words = setup.music.words()?;
        Some(Piece { words: words.to_string(), length_ms: (setup.music.minutes as f64 * 60_000.0).round() as u64, instrumental: setup.music.instrumental, take: setup.music.take })
    }

    /// The name this piece is filed under.
    ///
    /// Kept music is paid for, so this must not change for a setup that has not
    /// changed. It is a plain list in a fixed order, never a JSON object: the
    /// app and `pvc` can be built with objects that order their keys
    /// differently, and the two must file a piece under the same name.
    fn key(&self) -> String {
        hash(&json!(["music", 1, MUSIC_MODEL, self.words, self.length_ms, self.instrumental, self.take]))
    }

    pub fn file(&self, paths: &Paths) -> PathBuf {
        paths.music().join(format!("{}.mp3", self.key()))
    }

    /// Already made and kept: it costs nothing now.
    pub fn is_kept(&self, paths: &Paths) -> bool {
        self.file(paths).is_file()
    }

    pub fn minutes(&self) -> f32 {
        self.length_ms as f32 / 60_000.0
    }

    /// Roughly what making it costs. ElevenLabs does not give the price before
    /// it makes music, so this is its published rough figure, rounded up.
    pub fn estimate(&self) -> u64 {
        (self.length_ms * CREDITS_A_MINUTE).div_ceil(60_000)
    }
}

/// What a video's music would cost, for telling the person before they agree.
#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Cost {
    /// True when the setup asks ElevenLabs for music and the video would use it.
    pub wanted: bool,
    /// Already made and kept.
    pub kept: bool,
    pub minutes: f32,
    /// Roughly what making it would cost. 0 when there is nothing to make.
    /// A rough figure: ElevenLabs does not give the price beforehand.
    pub credits: u64,
}

/// Reads the disk only. Free and quick.
pub fn cost(paths: &Paths, setup: &Setup) -> Cost {
    match Piece::wanted(setup) {
        Some(piece) => {
            let kept = piece.is_kept(paths);
            Cost { wanted: true, kept, minutes: piece.minutes(), credits: if kept { 0 } else { piece.estimate() } }
        }
        None => Cost::default(),
    }
}

/// Things about the music worth telling the person before they pay.
pub fn notes(setup: &Setup) -> Vec<String> {
    let described = setup.music.prompt.as_deref().is_some_and(|p| !p.trim().is_empty());
    if described && setup.music.words().is_none() {
        vec!["A sound file is chosen for the background sound, so the music described in words is not made. Take the file away to have ElevenLabs make it.".to_string()]
    } else {
        Vec::new()
    }
}

/// How the cost of a piece came to be known.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum KnownBy {
    /// ElevenLabs said so in its answer.
    Reported,
    /// Read off the account: the credits used before and after.
    Counted,
    /// The account's credits did not move at all.
    Unmoved,
    /// Nothing told.
    Unknown,
}

/// A piece that has just been made, and what is known of its cost.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MadeMusic {
    pub minutes: f32,
    /// What it cost, when that could be told. Never a guess.
    pub charged: Option<u64>,
    pub known_by: KnownBy,
    /// The rough figure shown beforehand. Not what was charged.
    pub estimate: u64,
    /// The account's credits used, read just before and just after.
    pub used_before: Option<i64>,
    pub used_after: Option<i64>,
}

impl MadeMusic {
    /// What the piece cost, in a sentence, for telling the person afterwards.
    /// It says only what is known. The rough figure is never called the cost.
    pub fn said(&self) -> String {
        let made = format!("Made {} of music.", crate::make::minutes_words(self.minutes));
        let readings = match (self.used_before, self.used_after) {
            (Some(b), Some(a)) => format!(" Your credits used went from {b} to {a} while it was made."),
            _ => String::new(),
        };
        match (self.known_by, self.charged) {
            (KnownBy::Reported, Some(cost)) => format!("{made} ElevenLabs says it cost {cost} credits."),
            (KnownBy::Counted, Some(cost)) => format!("{made} It cost {cost} credits, going by your account.{readings} The rough figure beforehand was {}.", self.estimate),
            (KnownBy::Unmoved, _) => format!("{made} Your credits did not go down. ElevenLabs may count music apart from credits. Look at your usage at elevenlabs.io to see what it cost."),
            _ => format!("{made} What it cost could not be told.{readings} The rough figure beforehand was {} credits, and that is not a measurement. Look at your usage at elevenlabs.io.", self.estimate),
        }
    }
}

/// What is kept beside a piece.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct MusicMeta {
    words: String,
    model: String,
    instrumental: bool,
    take: u32,
    /// ElevenLabs' own name for the piece and for the request, for asking them about it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    song_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    #[serde(flatten)]
    made: MadeMusic,
}

/// What the readings of the account say a piece cost. `reported` is what
/// ElevenLabs itself said, if it said anything.
///
/// The before-and-after reading is only trusted when it is a number music
/// could have cost. If something else spent credits in the same minutes, or
/// the count started again, the reading is not the music's and is not used.
fn told(reported: Option<u64>, before: Option<i64>, after: Option<i64>, estimate: u64) -> (Option<u64>, KnownBy) {
    if let Some(cost) = reported {
        return (Some(cost), KnownBy::Reported);
    }
    match (before, after) {
        (Some(b), Some(a)) if a == b => (None, KnownBy::Unmoved),
        (Some(b), Some(a)) if a > b && ((a - b) as u64) <= estimate.saturating_mul(10) => (Some((a - b) as u64), KnownBy::Counted),
        _ => (None, KnownBy::Unknown),
    }
}

/// Have ElevenLabs make a piece, and keep it. Spends credits. The caller is
/// responsible for having asked first, with `Piece::estimate` as the rough cost.
///
/// The request is sent once. It is never sent again after a failure, because a
/// piece may have been made and charged for.
pub fn make(paths: &Paths, client: &Client, piece: &Piece) -> Result<MadeMusic> {
    std::fs::create_dir_all(paths.music())?;
    let estimate = piece.estimate();
    // Asking how many credits are used is free. It is the only way to see what music costs.
    let before = client.credits().ok().map(|c| c.used);
    let composed = match client.compose(&MusicRequest { prompt: &piece.words, length_ms: piece.length_ms, instrumental: piece.instrumental }) {
        Ok(composed) => composed,
        Err(e) => {
            // A piece that was not made should cost nothing. Do not say so unless the account agrees.
            let after = client.credits().ok().map(|c| c.used);
            return Err(match (e, before, after) {
                (Error::Voice(said), Some(b), Some(a)) if a > b => Error::voice(format!("{said} Your credits went down by {} while this was tried. Look at your usage at elevenlabs.io.", a - b)),
                (e, _, _) => e,
            });
        }
    };
    let after = client.credits().ok().map(|c| c.used);
    let (charged, known_by) = told(composed.cost, before, after, estimate);
    let made = MadeMusic { minutes: piece.minutes(), charged, known_by, estimate, used_before: before, used_after: after };
    let meta = MusicMeta { words: piece.words.clone(), model: MUSIC_MODEL.to_string(), instrumental: piece.instrumental, take: piece.take, song_id: composed.song_id, request_id: composed.request_id, made: made.clone() };
    let file = piece.file(paths);
    std::fs::write(file.with_extension("json"), serde_json::to_string(&meta)?)?;
    // The sound is written last: its presence is what marks the piece as made.
    std::fs::write(&file, &composed.audio)?;
    match charged {
        Some(cost) => log::info!("made {:.1} minutes of music: {cost} credits", piece.minutes()),
        None => log::info!("made {:.1} minutes of music; what it cost could not be told", piece.minutes()),
    }
    Ok(made)
}

/// The filter that fades the end of a piece into its start. What comes out is
/// `seconds` shorter, and ends exactly where it begins, so it can go round and
/// round under a video without a bump.
fn join_filter(seconds: f32) -> String {
    format!(
        "[0:a]aresample={SR},aformat=sample_fmts=s16:channel_layouts=stereo,asplit[a][b];\
         [a]atrim=start={seconds},asetpts=PTS-STARTPTS[main];\
         [b]atrim=end={seconds},asetpts=PTS-STARTPTS[head];\
         [main][head]acrossfade=d={seconds}:c1=tri:c2=tri[out]"
    )
}

/// A copy of a kept piece that loops gently, written into `work`. If FFmpeg
/// cannot make it, the piece itself is used: a bump at the join is better than
/// no video.
pub fn loopable(ffmpeg: &Path, piece: &Path, work: &Path) -> PathBuf {
    let out = work.join("music-loop.wav");
    let made = tools::run(
        tools::command(ffmpeg).args(["-y", "-v", "error", "-i"]).arg(piece).args(["-filter_complex", &join_filter(JOIN_SECONDS), "-map", "[out]", "-c:a", "pcm_s16le"]).arg(&out),
        "FFmpeg (making the music loop)",
    );
    match made {
        Ok(_) if out.is_file() => out,
        Ok(_) => piece.to_path_buf(),
        Err(e) => {
            log::warn!("the music could not be made to loop gently, so it is used as it came: {e}");
            piece.to_path_buf()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::from_doc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn temp(tag: &str) -> Paths {
        let dir = std::env::temp_dir().join(format!("pvc-music-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Paths::new(dir)
    }

    fn setup(music: serde_json::Value) -> Setup {
        from_doc(&json!({ "music": music })).unwrap()
    }

    /// An account whose credits used read `readings[0]` the first time it is asked, `readings[1]` the second, and so on.
    fn account(server: &mut mockito::Server, readings: &'static [i64]) -> mockito::Mock {
        let asked = Arc::new(AtomicUsize::new(0));
        server
            .mock("GET", "/v1/user/subscription")
            .with_body_from_request(move |_| {
                let n = asked.fetch_add(1, Ordering::SeqCst).min(readings.len() - 1);
                format!(r#"{{"character_count":{},"character_limit":100000}}"#, readings[n]).into_bytes()
            })
            .create()
    }

    #[test]
    fn music_is_only_made_when_it_is_asked_for_in_words() {
        // Nothing asked: nothing wanted, nothing to pay.
        let paths = temp("wanted");
        assert_eq!(Piece::wanted(&setup(json!({}))), None);
        assert_eq!(cost(&paths, &setup(json!({}))), Cost::default());
        assert_eq!(Piece::wanted(&setup(json!({ "prompt": "   " }))), None);

        // Words: a piece of two minutes, with no singing, unless the setup says otherwise.
        let piece = Piece::wanted(&setup(json!({ "prompt": "  slow, quiet organ in an empty chapel " }))).unwrap();
        assert_eq!(piece, Piece { words: "slow, quiet organ in an empty chapel".into(), length_ms: 120_000, instrumental: true, take: 1 });
        let piece = Piece::wanted(&setup(json!({ "prompt": "a choir", "minutes": 0.5, "instrumental": false, "take": 3 }))).unwrap();
        assert_eq!((piece.length_ms, piece.instrumental, piece.take), (30_000, false, 3));

        // A sound file the person supplies wins, and they are told the words are not used.
        let both = setup(json!({ "prompt": "slow organ", "file": "chant.mp3" }));
        assert_eq!(Piece::wanted(&both), None);
        assert_eq!(cost(&paths, &both), Cost::default());
        let notes = notes(&both);
        assert!(notes.len() == 1 && notes[0].contains("sound file"), "{notes:?}");
        assert!(super::notes(&setup(json!({ "prompt": "slow organ" }))).is_empty());

        // A video with no sound to put it under needs no music: the guide alone, or captions alone.
        let quiet = from_doc(&json!({ "music": { "prompt": "slow organ" }, "output": { "video": false, "audio": false, "captions": true, "guide": true } })).unwrap();
        assert_eq!(Piece::wanted(&quiet), None);
    }

    #[test]
    fn the_rough_cost_is_nine_hundred_credits_a_minute_rounded_up() {
        let piece = |minutes: f64| Piece::wanted(&setup(json!({ "prompt": "slow organ", "minutes": minutes }))).unwrap();
        assert_eq!(piece(2.0).estimate(), 1800);
        assert_eq!(piece(0.5).estimate(), 450);
        assert_eq!(piece(5.0).estimate(), 4500);
        // 1.3 minutes is 78 seconds: 1,170 credits. A part of a credit counts as a whole one.
        assert_eq!((piece(1.3).length_ms, piece(1.3).estimate()), (78_000, 1170));
        assert_eq!(Piece { words: "x".into(), length_ms: 30_001, instrumental: true, take: 1 }.estimate(), 451);

        let paths = temp("estimate");
        let c = cost(&paths, &setup(json!({ "prompt": "slow organ" })));
        assert_eq!((c.wanted, c.kept, c.minutes, c.credits), (true, false, 2.0, 1800));
    }

    #[test]
    fn bad_values_are_refused_in_plain_words() {
        for (music, says) in [
            (json!({ "minutes": 0.2 }), "music.minutes must be between 0.5 and 5"),
            (json!({ "minutes": 10 }), "music.minutes must be between 0.5 and 5"),
            (json!({ "take": 0 }), "music.take must be a whole number from 1 to 99"),
            (json!({ "prompt": "x".repeat(2001) }), "music.prompt is 2001 letters long"),
        ] {
            let err = from_doc(&json!({ "music": music })).unwrap_err().to_string();
            assert!(err.contains(says), "{err}");
        }
        // Music belongs to the whole video. A rule cannot set it.
        let err = from_doc(&json!({ "rules": [{ "for": { "mystery": 1 }, "set": { "music": { "prompt": "organ" } } }] })).unwrap_err().to_string();
        assert!(err.contains("music"), "{err}");
    }

    #[test]
    fn a_piece_is_filed_under_a_name_that_never_moves() {
        let piece = Piece { words: "slow, quiet organ in an empty chapel".into(), length_ms: 120_000, instrumental: true, take: 1 };
        // Pinned. If this changes, music already paid for is no longer found.
        assert_eq!(piece.key(), hash(&json!(["music", 1, "music_v2_5", "slow, quiet organ in an empty chapel", 120000, true, 1])));
        assert_eq!(piece.key(), "5658a3bf2bbe3961d64686a1d8c2e0cb");
        // Everything that makes it a different piece gives it a different name.
        let mut seen = std::collections::BTreeSet::from([piece.key()]);
        for other in [
            Piece { words: "slow, quiet organ in an empty church".into(), ..piece.clone() },
            Piece { length_ms: 180_000, ..piece.clone() },
            Piece { instrumental: false, ..piece.clone() },
            Piece { take: 2, ..piece.clone() },
        ] {
            assert!(seen.insert(other.key()), "{other:?} is filed where another piece is");
        }
    }

    #[test]
    fn what_a_piece_cost_is_read_off_the_account_and_never_guessed() {
        // ElevenLabs said so itself: that wins.
        assert_eq!(told(Some(1650), Some(100), Some(5000), 1800), (Some(1650), KnownBy::Reported));
        // It did not say, and the credits used went up by a number music could cost.
        assert_eq!(told(None, Some(1000), Some(2740), 1800), (Some(1740), KnownBy::Counted));
        // The credits did not move: music may be counted apart from credits.
        assert_eq!(told(None, Some(1000), Some(1000), 1800), (None, KnownBy::Unmoved));
        // The account could not be read, the count started again, or far more went than music costs
        // (something else was spending at the same time). None of these is called a cost.
        assert_eq!(told(None, None, Some(2740), 1800), (None, KnownBy::Unknown));
        assert_eq!(told(None, Some(1000), None, 1800), (None, KnownBy::Unknown));
        assert_eq!(told(None, Some(9000), Some(40), 1800), (None, KnownBy::Unknown));
        assert_eq!(told(None, Some(1000), Some(60_000), 1800), (None, KnownBy::Unknown));
    }

    #[test]
    fn the_cost_is_said_as_what_it_is() {
        let made = |known_by, charged, before, after| MadeMusic { minutes: 2.0, charged, known_by, estimate: 1800, used_before: before, used_after: after };
        let counted = made(KnownBy::Counted, Some(1740), Some(1000), Some(2740)).said();
        assert!(counted.contains("Made 2 minutes of music.") && counted.contains("It cost 1740 credits") && counted.contains("from 1000 to 2740"), "{counted}");
        assert!(made(KnownBy::Reported, Some(1650), None, None).said().contains("ElevenLabs says it cost 1650 credits"));
        // Not known: the rough figure is named as a rough figure, never as the cost.
        let unknown = made(KnownBy::Unknown, None, None, None).said();
        assert!(unknown.contains("could not be told") && unknown.contains("not a measurement") && !unknown.contains("It cost"), "{unknown}");
        let unmoved = made(KnownBy::Unmoved, None, Some(700), Some(700)).said();
        assert!(unmoved.contains("did not go down") && !unmoved.contains("It cost") && !unmoved.contains("1800"), "{unmoved}");
    }

    #[test]
    fn a_piece_is_made_once_kept_and_its_cost_counted() {
        let paths = temp("made");
        let piece = Piece::wanted(&setup(json!({ "prompt": "slow, quiet organ", "minutes": 1 }))).unwrap();
        let mut server = mockito::Server::new();
        account(&mut server, &[5000, 5870]);
        let music = server
            .mock("POST", "/v1/music")
            .match_query(mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_128".into()))
            .match_header("xi-api-key", "k")
            .match_body(mockito::Matcher::Json(json!({ "prompt": "slow, quiet organ", "music_length_ms": 60000, "model_id": "music_v2_5", "force_instrumental": true })))
            .with_header("song-id", "song-7")
            .with_body([1u8, 2, 3, 4])
            .expect(1)
            .create();
        let client = Client::with_base("k", &server.url()).unwrap();
        assert!(!piece.is_kept(&paths));
        let made = make(&paths, &client, &piece).unwrap();
        music.assert();
        // 900 was the rough figure. 870 is what the account says it came to.
        assert_eq!((made.estimate, made.charged, made.known_by), (900, Some(870), KnownBy::Counted));
        assert_eq!((made.used_before, made.used_after), (Some(5000), Some(5870)));
        // It is kept, with what is known about it beside it, and costs nothing from now on.
        assert!(piece.is_kept(&paths));
        assert_eq!(std::fs::read(piece.file(&paths)).unwrap(), [1, 2, 3, 4]);
        let beside: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(piece.file(&paths).with_extension("json")).unwrap()).unwrap();
        assert_eq!((beside["words"].as_str(), beside["songId"].as_str(), beside["charged"].as_u64(), beside["knownBy"].as_str()), (Some("slow, quiet organ"), Some("song-7"), Some(870), Some("counted")));
        let c = cost(&paths, &setup(json!({ "prompt": "slow, quiet organ", "minutes": 1 })));
        assert_eq!((c.wanted, c.kept, c.credits), (true, true, 0));
        // Another take of the same words is another piece, and costs again.
        assert_eq!(cost(&paths, &setup(json!({ "prompt": "slow, quiet organ", "minutes": 1, "take": 2 }))).credits, 900);
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn when_the_cost_cannot_be_told_it_is_not_made_up() {
        let piece = Piece { words: "slow organ".into(), length_ms: 30_000, instrumental: true, take: 1 };

        // The account does not answer: the piece is still made and kept, and its cost is left unknown.
        let paths = temp("untold");
        let mut server = mockito::Server::new();
        server.mock("GET", "/v1/user/subscription").with_status(500).create();
        server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_body([9u8]).create();
        let made = make(&paths, &Client::with_base("k", &server.url()).unwrap(), &piece).unwrap();
        assert_eq!((made.charged, made.known_by, made.estimate), (None, KnownBy::Unknown, 450));
        assert!(piece.is_kept(&paths));
        let _ = std::fs::remove_dir_all(&paths.data);

        // The credits do not move: said as that, not as "free" and not as the rough figure.
        let paths = temp("unmoved");
        let mut server = mockito::Server::new();
        account(&mut server, &[700, 700]);
        server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_body([9u8]).create();
        let made = make(&paths, &Client::with_base("k", &server.url()).unwrap(), &piece).unwrap();
        assert_eq!((made.charged, made.known_by), (None, KnownBy::Unmoved));
        let _ = std::fs::remove_dir_all(&paths.data);

        // If ElevenLabs ever does report the cost itself, that is used.
        let paths = temp("reported");
        let mut server = mockito::Server::new();
        account(&mut server, &[700, 9000]);
        server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_header("character-cost", "412").with_body([9u8]).create();
        let made = make(&paths, &Client::with_base("k", &server.url()).unwrap(), &piece).unwrap();
        assert_eq!((made.charged, made.known_by), (Some(412), KnownBy::Reported));
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn a_refusal_is_told_plainly_and_nothing_is_kept() {
        let paths = temp("refused");
        let piece = Piece { words: "Ave Maria, sung softly".into(), length_ms: 30_000, instrumental: true, take: 1 };
        let mut server = mockito::Server::new();
        account(&mut server, &[700, 700]);
        let refused = server
            .mock("POST", "/v1/music")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"detail":{"status":"bad_prompt","message":"The prompt names a song.","data":{"prompt_suggestion":"A slow, tender melody for solo voice and organ"}}}"#)
            .expect(1)
            .create();
        let client = Client::with_base("k", &server.url()).unwrap();
        let err = make(&paths, &client, &piece).unwrap_err().to_string();
        // Asked once. A refusal is not tried again.
        refused.assert();
        assert!(err.contains("would not make music") && err.contains("hymn") && err.contains("A slow, tender melody for solo voice and organ"), "{err}");
        assert!(!err.contains("credits went down"), "{err}");
        assert!(!piece.is_kept(&paths));
        let _ = std::fs::remove_dir_all(&paths.data);

        // If the account says credits went all the same, the person is told that too.
        let paths = temp("refused-charged");
        let mut server = mockito::Server::new();
        account(&mut server, &[700, 1150]);
        server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_status(400).with_body(r#"{"detail":{"status":"bad_prompt","message":"No."}}"#).create();
        let err = make(&paths, &Client::with_base("k", &server.url()).unwrap(), &piece).unwrap_err().to_string();
        assert!(err.contains("would not make music") && err.contains("went down by 450"), "{err}");
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn the_end_of_a_piece_is_faded_into_its_start() {
        let graph = join_filter(3.0);
        assert!(graph.contains("atrim=start=3,") && graph.contains("atrim=end=3,") && graph.contains("acrossfade=d=3:"), "{graph}");
        assert!(graph.starts_with("[0:a]aresample=48000,") && graph.ends_with("[out]"), "{graph}");
    }
}
