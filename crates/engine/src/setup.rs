//! A setup is everything a person chose for one video.
//!
//! Three promises live in this file:
//!
//! 1. **It works with no choices made.** Every setting has a default, so `{}` is a
//!    whole setup.
//! 2. **Changing one thing means touching one thing.** A setup file only holds what
//!    differs. It can start from another setup with `"extends"`.
//! 3. **Settings narrow.** `voice`, `pace`, `words`, `picture` and `colors` can be set
//!    for the whole video, or by a rule for one section, one mystery, one kind of
//!    prayer, or one bead. The narrowest rule wins.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

// ───────────────────────── the whole setup ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Setup {
    pub name: String,
    pub description: String,
    pub beads: Beads,
    pub prayers: Prayers,
    pub voice: Voice,
    pub pace: Pace,
    pub words: Words,
    pub picture: Picture,
    pub colors: Colors,
    pub screen: Screen,
    pub music: Music,
    pub output: Output,
    pub rules: Vec<Rule>,
}

/// The settings that can narrow. A rule's `set` is a partial copy of this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Style {
    pub voice: Voice,
    pub pace: Pace,
    pub words: Words,
    pub picture: Picture,
    pub colors: Colors,
}

pub const STYLE_GROUPS: [&str; 5] = ["voice", "pace", "words", "picture", "colors"];

// ───────────────────────── beads ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum BeadKind {
    /// The standard Catholic rosary: 59 beads. The only kind checked so far.
    #[default]
    FiveDecadeRosary,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Medal {
    #[default]
    Plain,
    StBenedict,
    Miraculous,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Cross {
    #[default]
    Latin,
    StBenedict,
    Orthodox,
    Greek,
    Knotted,
    Tassel,
    Celtic,
    SanDamiano,
    Medal,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum BeadStyle {
    /// Plain outlined circles, as in the first video.
    #[default]
    Outline,
    Wood,
    Glass,
    Stone,
    Pearl,
    Metal,
    Knots,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Cord {
    #[default]
    Chain,
    Cord,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    /// The decades go up the right-hand side first, as in the first video.
    #[default]
    RightFirst,
    LeftFirst,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Beads {
    pub kind: BeadKind,
    pub medal: Medal,
    pub cross: Cross,
    pub style: BeadStyle,
    /// Bead color. Empty means the usual color for the style.
    pub color: Option<String>,
    /// 1 is the size in the first video.
    pub size: f32,
    pub cord: Cord,
    pub cord_color: Option<String>,
    pub direction: Direction,
}

impl Default for Beads {
    fn default() -> Self {
        Beads {
            kind: BeadKind::default(),
            medal: Medal::default(),
            cross: Cross::default(),
            style: BeadStyle::default(),
            color: None,
            size: 1.0,
            cord: Cord::default(),
            cord_color: None,
            direction: Direction::default(),
        }
    }
}

// ───────────────────────── prayers ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Mysteries {
    /// Whatever is usual for the day the video is made.
    #[default]
    Today,
    Joyful,
    Sorrowful,
    Glorious,
    Luminous,
    /// Your own five meditations, written in `prayers.custom`.
    Custom,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum GloryBePlace {
    /// On the chain, with no bead. The usual way.
    #[default]
    Chain,
    /// On the next big bead, before the Our Father.
    NextBead,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomMysteries {
    /// Shown as "<name> Mysteries".
    pub name: String,
    /// Five names, one for each decade.
    pub items: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtraPrayer {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Prayers {
    /// The big title on the opening card.
    pub title: String,
    pub mysteries: Mysteries,
    pub custom: CustomMysteries,
    /// Start at the crucifix (true), or go straight to the first decade (false).
    pub opening: bool,
    /// End with the Hail, Holy Queen and the Sign of the Cross.
    pub closing: bool,
    pub first_decade: u8,
    pub last_decade: u8,
    /// "O My Jesus" after each Glory Be. Optional in the rosary; on in the first video.
    pub fatima: bool,
    /// The Closing Prayer after the Hail, Holy Queen. Optional; on in the first video.
    pub closing_prayer: bool,
    pub glory_be: GloryBePlace,
    /// Your own wording for a prayer: `{ "hailmary": ["line one", "line two"] }`.
    pub wording: BTreeMap<String, Vec<String>>,
    /// Extra prayers after the opening Sign of the Cross.
    pub extra_start: Vec<ExtraPrayer>,
    /// Extra prayers on the medal, before the final Sign of the Cross.
    pub extra_end: Vec<ExtraPrayer>,
    /// A reading before each mystery, keyed "1" to "5". You supply the text, word for word.
    pub readings: BTreeMap<String, String>,
}

impl Default for Prayers {
    fn default() -> Self {
        Prayers {
            title: "The Holy Rosary".into(),
            mysteries: Mysteries::default(),
            custom: CustomMysteries::default(),
            opening: true,
            closing: true,
            first_decade: 1,
            last_decade: 5,
            fatima: true,
            closing_prayer: true,
            glory_be: GloryBePlace::default(),
            wording: BTreeMap::new(),
            extra_start: Vec::new(),
            extra_end: Vec::new(),
            readings: BTreeMap::new(),
        }
    }
}

// ───────────────────────── voice (narrows) ─────────────────────────

/// When several voices take turns: how often the next one takes over.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TurnEvery {
    /// A different voice on every prayer.
    #[default]
    Prayer,
    /// One voice leads a whole decade, then the next takes over. The opening
    /// prayers and the closing prayers count as one turn each.
    Decade,
}

/// What joins several voice names into one setting: "Bella + George".
pub const VOICE_JOIN: &str = " + ";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Voice {
    /// false = no voice. Words on screen only.
    pub on: bool,
    /// The name of a voice in your ElevenLabs account. Several names joined
    /// with " + " take turns: "Bella + George".
    pub name: String,
    /// An exact ElevenLabs voice id. Wins over the name when both are given.
    pub id: Option<String>,
    pub model: String,
    /// 1 is the pace of the first video. 0.8 is slower, 1.2 is faster.
    /// The voice is recorded at its natural pace and then sped up to land on this.
    pub speed: f32,
    /// Skip the measuring and speed the recording up by exactly this much (1.75 = 1.75x).
    pub exact_speedup: Option<f32>,
    pub stability: Option<f32>,
    pub similarity: Option<f32>,
    pub style: Option<f32>,
    /// Your own sound file for the prayer, used in place of a generated voice.
    pub recording: Option<String>,
    /// Your own sound file for naming the mystery.
    pub announce_recording: Option<String>,
    /// Speed change for your own recordings. 1 leaves them alone.
    pub recording_speed: f32,
    /// A second voice that answers with the second half of each prayer.
    pub respond: Option<String>,
    /// 0 = the voice says every Hail Mary. 3 = it says the first three of each
    /// decade and then leaves you to say the rest.
    pub drop_after: u32,
    /// Say the name of each mystery aloud.
    pub announce: bool,
    /// With several voices in `name`: how often the next one takes over.
    pub turn_every: TurnEvery,
    /// How many different recordings of each prayer take turns. 1 = one
    /// recording, used every time, as in the first video.
    pub takes: u32,
    /// Record each prayer afresh every time it is said, so no two sound the
    /// same. Wins over `takes`. Costs the most credits.
    pub fresh_each_time: bool,
    /// Mix up the order of the takes and of the voices, instead of going in turn.
    pub shuffle: bool,
}

impl Voice {
    /// The voices that lead, in the order given. More than one means they take turns.
    pub fn names(&self) -> Vec<String> {
        self.name.split(VOICE_JOIN).map(str::trim).filter(|n| !n.is_empty()).map(String::from).collect()
    }

    /// True when one exact voice id has been given. It leads alone, whatever `name` says.
    pub fn has_id(&self) -> bool {
        self.id.as_deref().is_some_and(|i| !i.trim().is_empty())
    }
}

impl Default for Voice {
    fn default() -> Self {
        Voice {
            on: true,
            name: "Imperial - Male".into(),
            id: None,
            model: "eleven_multilingual_v2".into(),
            speed: 1.0,
            exact_speedup: None,
            stability: None,
            similarity: None,
            style: None,
            recording: None,
            announce_recording: None,
            recording_speed: 1.0,
            respond: None,
            drop_after: 0,
            announce: true,
            turn_every: TurnEvery::default(),
            takes: 1,
            fresh_each_time: false,
            shuffle: false,
        }
    }
}

// ───────────────────────── pace (narrows) ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Transition {
    #[default]
    Fade,
    Cut,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Pace {
    /// Quiet before each prayer, in seconds.
    pub before: f32,
    /// Quiet after each prayer, in seconds.
    pub after: f32,
    /// Between naming the mystery and starting the Our Father.
    pub announce_gap: f32,
    /// Extra time to think after the mystery is named.
    pub think: f32,
    /// With no voice: how long each line of a prayer stays up.
    pub seconds_per_line: f32,
    /// With no voice: the shortest any bead stays up.
    pub min_seconds: f32,
    pub transition: Transition,
    /// Lets the quiet after each prayer differ a little from bead to bead, by up
    /// to this many seconds either way. 0 keeps every pause the same.
    pub vary: f32,
    /// Length of the soft fade between beads, in seconds.
    pub fade: f32,
    /// How long the opening title stays up. Whole video only.
    pub intro: f32,
    /// How long the ending stays up. Whole video only.
    pub outro: f32,
}

impl Default for Pace {
    fn default() -> Self {
        Pace {
            before: 0.30,
            after: 0.70,
            announce_gap: 0.45,
            think: 0.0,
            seconds_per_line: 2.0,
            min_seconds: 4.0,
            transition: Transition::default(),
            vary: 0.0,
            fade: 0.3,
            intro: 4.0,
            outro: 4.0,
        }
    }
}

// ───────────────────────── words on screen (narrows) ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TextMode {
    #[default]
    Always,
    /// Only the first time each prayer comes up.
    First,
    Never,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Words {
    /// 1 is the size in the first video.
    pub size: f32,
    pub text: TextMode,
    /// The line being spoken stands out.
    pub follow: bool,
    /// The dots that fill as you go through a decade.
    pub counter: bool,
    /// "Prayer 13 of 75" and the bar under it.
    pub progress: bool,
    /// The mystery name and number.
    pub mystery_name: bool,
    /// Lines like "Many people offer this one for faith."
    pub hints: bool,
    /// The line that says where your fingers are: "Small bead 4 of 10".
    pub place: bool,
    /// The "Optional" label on optional prayers.
    pub optional_label: bool,
}

impl Default for Words {
    fn default() -> Self {
        Words {
            size: 1.0,
            text: TextMode::default(),
            follow: false,
            counter: true,
            progress: true,
            mystery_name: true,
            hints: true,
            place: true,
            optional_label: true,
        }
    }
}

// ───────────────────────── picture (narrows) ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PicturePlace {
    /// In a circle inside the loop of beads, as in the first video.
    #[default]
    Loop,
    /// Above the words.
    Beside,
    /// Filling the whole screen, dimmed, behind everything.
    Background,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Picture {
    /// The picture file. Empty means no picture; the loop holds text instead.
    pub file: Option<String>,
    /// Which part sits in the middle: "50% 90%" is centered, near the bottom.
    pub focus: String,
    /// 1 shows as much as fits. 1.2 moves in closer, toward the focus.
    pub zoom: f32,
    /// Artist, title and date.
    pub credit: String,
    pub show_credit: bool,
    pub place: PicturePlace,
    /// false hides the picture here without forgetting which file it was.
    pub show: bool,
}

impl Default for Picture {
    fn default() -> Self {
        Picture {
            file: None,
            focus: "50% 50%".into(),
            zoom: 1.0,
            credit: String::new(),
            show_credit: true,
            place: PicturePlace::default(),
            show: true,
        }
    }
}

// ───────────────────────── colors and fonts (narrows) ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Dark,
    Light,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Colors {
    pub mode: Mode,
    pub background: Option<String>,
    pub panel: Option<String>,
    pub text: Option<String>,
    pub quiet_text: Option<String>,
    pub lines: Option<String>,
    /// Beads you have already prayed.
    pub done: Option<String>,
    /// The bead you are on. Gold in the first video.
    pub marker: Option<String>,
    pub title_font: Option<String>,
    pub text_font: Option<String>,
}

/// Colors with every blank filled in. This is what a frame is drawn with.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub mode: Mode,
    pub background: String,
    pub panel: String,
    pub text: String,
    pub quiet_text: String,
    pub lines: String,
    pub done: String,
    pub marker: String,
    pub title_font: String,
    pub text_font: String,
}

impl Colors {
    pub fn palette(&self) -> Palette {
        // Dark is the first video, value for value (prototype/frame.html).
        let base: [&str; 7] = match self.mode {
            Mode::Dark => ["#0E1626", "#172138", "#E7ECF5", "#9BA8C0", "#34425F", "#6F80A6", "#E6B556"],
            Mode::Light => ["#F7F3EA", "#ECE5D6", "#1C2434", "#5A657B", "#CFC6B4", "#98A3B9", "#C8860A"],
        };
        let pick = |own: &Option<String>, i: usize| own.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| base[i].to_string());
        Palette {
            mode: self.mode,
            background: pick(&self.background, 0),
            panel: pick(&self.panel, 1),
            text: pick(&self.text, 2),
            quiet_text: pick(&self.quiet_text, 3),
            lines: pick(&self.lines, 4),
            done: pick(&self.done, 5),
            marker: pick(&self.marker, 6),
            title_font: self.title_font.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "Cardo".into()),
            text_font: self.text_font.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "Fira Sans".into()),
        }
    }
}

// ───────────────────────── screen, music, output (whole video) ─────────────────────────

pub struct ScreenPreset {
    pub key: &'static str,
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
}

pub const SCREENS: &[ScreenPreset] = &[
    ScreenPreset { key: "pixel-9-pro-fold", name: "Pixel 9 Pro Fold, inner screen", width: 2076, height: 2152 },
    ScreenPreset { key: "pixel-9-pro-fold-cover", name: "Pixel 9 Pro Fold, cover screen", width: 1080, height: 2424 },
    ScreenPreset { key: "phone-tall", name: "Phone, held upright", width: 1080, height: 1920 },
    ScreenPreset { key: "phone-wide", name: "Phone, on its side", width: 1920, height: 1080 },
    ScreenPreset { key: "tablet", name: "Tablet, on its side", width: 2048, height: 1536 },
    ScreenPreset { key: "tablet-tall", name: "Tablet, upright", width: 1536, height: 2048 },
    ScreenPreset { key: "tv-1080p", name: "TV or computer, 1080p", width: 1920, height: 1080 },
    ScreenPreset { key: "tv-4k", name: "TV, 4K", width: 3840, height: 2160 },
    ScreenPreset { key: "square", name: "Square", width: 1080, height: 1080 },
];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct SafeEdges {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Screen {
    /// One of the named sizes. See `SCREENS`.
    pub preset: String,
    /// An exact size. Wins over the preset when both width and height are given.
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Space kept clear at each edge, for camera cutouts and rounded corners.
    pub safe: SafeEdges,
    pub fps: u32,
}

impl Default for Screen {
    fn default() -> Self {
        Screen { preset: "pixel-9-pro-fold".into(), width: None, height: None, safe: SafeEdges::default(), fps: 30 }
    }
}

impl Screen {
    /// The size in pixels. Always even, because the video format needs that.
    pub fn size(&self) -> Result<(u32, u32)> {
        let (w, h) = match (self.width, self.height) {
            (Some(w), Some(h)) => (w, h),
            _ => {
                let p = SCREENS.iter().find(|p| p.key == self.preset).ok_or_else(|| {
                    Error::setup(format!(
                        "There is no screen size called \"{}\". The choices are: {}.",
                        self.preset,
                        SCREENS.iter().map(|p| p.key).collect::<Vec<_>>().join(", ")
                    ))
                })?;
                (p.width, p.height)
            }
        };
        if !(240..=7680).contains(&w) || !(240..=7680).contains(&h) {
            return Err(Error::setup(format!("A screen of {w} x {h} is outside what this tool can make (240 to 7680 each way).")));
        }
        Ok((w & !1, h & !1))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Music {
    /// A sound file you supply: music or chant. Empty means silence.
    pub file: Option<String>,
    /// How loud it is under the voice. 0 to 1.
    pub volume: f32,
}

impl Default for Music {
    fn default() -> Self {
        Music { file: None, volume: 0.12 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Output {
    pub video: bool,
    /// A sound-only file, for praying on a walk.
    pub audio: bool,
    /// A separate captions file (.srt).
    pub captions: bool,
    /// Chapter marks, so you can jump to a decade.
    pub chapters: bool,
    /// The tap-through guide: one bead at a time, at your own pace.
    pub guide: bool,
    /// Where the files go. Empty means your Videos folder, in "Prayer Videos".
    pub folder: Option<String>,
    /// The file name, without the ending. Empty means the setup's name.
    pub file_name: Option<String>,
    /// Picture quality. Lower is better and bigger. 18 is the first video.
    pub quality: u8,
}

impl Default for Output {
    fn default() -> Self {
        Output { video: true, audio: false, captions: false, chapters: true, guide: false, folder: None, file_name: None, quality: 18 }
    }
}

// ───────────────────────── rules ─────────────────────────

pub const SECTIONS: [&str; 9] = ["intro", "opening", "decade-1", "decade-2", "decade-3", "decade-4", "decade-5", "closing", "outro"];

/// What part of the video a rule covers. Blank fields cover everything.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    /// "intro", "opening", "decade-1" to "decade-5", "closing" or "outro".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// 1 to 5.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mystery: Option<u8>,
    /// "ourfather", "hailmary", "glorybe" and so on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prayer: Option<String>,
    /// One bead: "cross", "p1".."p5", "medal", "d3b10" (decade 3, bead 10), "L2".."L5", "g0".."g5" (the chain).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bead: Option<String>,
}

impl Target {
    /// How narrow this is. 1 = whole video … 5 = one bead. The widest list in
    /// docs/what-you-can-change.md, in the same order.
    pub fn level(&self) -> u8 {
        if self.bead.is_some() {
            5
        } else if self.prayer.is_some() {
            4
        } else if self.mystery.is_some() {
            3
        } else if self.section.is_some() {
            2
        } else {
            1
        }
    }

    pub fn conditions(&self) -> usize {
        self.section.is_some() as usize + self.mystery.is_some() as usize + self.prayer.is_some() as usize + self.bead.is_some() as usize
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    #[serde(rename = "for", default)]
    pub target: Target,
    /// A partial copy of the narrowing settings: `{ "voice": { "name": "…" } }`.
    #[serde(default)]
    pub set: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ───────────────────────── reading and merging ─────────────────────────

/// Lay `over` on top of `base`. Objects merge key by key. Everything else replaces.
/// `null` in `over` removes the key, which puts that setting back to its default.
pub fn merge(base: &mut Value, over: &Value) {
    match (base, over) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                if v.is_null() {
                    b.remove(k);
                } else if let Some(existing) = b.get_mut(k) {
                    merge(existing, v);
                } else {
                    b.insert(k.clone(), strip_nulls(v));
                }
            }
        }
        (b, o) => *b = strip_nulls(o),
    }
}

fn strip_nulls(v: &Value) -> Value {
    match v {
        Value::Object(o) => Value::Object(o.iter().filter(|(_, v)| !v.is_null()).map(|(k, v)| (k.clone(), strip_nulls(v))).collect()),
        other => other.clone(),
    }
}

/// Lay a child setup on a parent. Same as `merge`, except the child's rules are
/// added after the parent's instead of replacing them.
pub fn merge_docs(parent: &Value, child: &Value) -> Value {
    let mut out = parent.clone();
    let mut child = child.clone();
    let child_rules = child.as_object_mut().and_then(|o| o.remove("rules"));
    merge(&mut out, &child);
    if let Some(Value::Array(extra)) = child_rules {
        let obj = out.as_object_mut().expect("a setup is an object");
        let rules = obj.entry("rules").or_insert_with(|| Value::Array(Vec::new()));
        if let Value::Array(list) = rules {
            list.extend(extra);
        }
    }
    out
}

/// Turn a sparse setup into a full one, with every default filled in.
/// `doc` must already have its `extends` chain folded in (see `library`).
pub fn from_doc(doc: &Value) -> Result<Setup> {
    if !doc.is_object() {
        return Err(Error::setup("A setup has to be a JSON object: { ... }."));
    }
    let mut full = serde_json::to_value(Setup::default())?;
    let mut doc = doc.clone();
    if let Some(o) = doc.as_object_mut() {
        o.remove("extends");
        o.remove("$schema");
    }
    merge(&mut full, &doc);
    let setup: Setup = serde_json::from_value(full).map_err(|e| Error::setup(explain(&e.to_string())))?;
    setup.check()?;
    Ok(setup)
}

/// serde's messages are close to plain English already. Tidy the rest.
fn explain(msg: &str) -> String {
    let msg = msg.replace("unknown field", "there is no setting called").replace("unknown variant", "there is no choice called");
    format!("This setup has a mistake: {msg}.")
}

impl Setup {
    pub fn style(&self) -> Style {
        Style { voice: self.voice.clone(), pace: self.pace.clone(), words: self.words.clone(), picture: self.picture.clone(), colors: self.colors.clone() }
    }

    /// Catch what the types cannot.
    pub fn check(&self) -> Result<()> {
        let p = &self.prayers;
        if !(1..=5).contains(&p.first_decade) || !(1..=5).contains(&p.last_decade) || p.first_decade > p.last_decade {
            return Err(Error::setup(format!(
                "prayers.firstDecade and prayers.lastDecade must run from 1 to 5, first before last. They are {} and {}.",
                p.first_decade, p.last_decade
            )));
        }
        if p.mysteries == Mysteries::Custom && p.custom.items.len() != 5 {
            return Err(Error::setup(format!(
                "Your own mysteries need five names in prayers.custom.items, one for each decade. There are {}.",
                p.custom.items.len()
            )));
        }
        for key in p.wording.keys() {
            if crate::prayers::prayer(key).is_none() {
                return Err(Error::setup(format!(
                    "prayers.wording has \"{key}\", which is not a prayer this tool knows. The prayers are: {}.",
                    crate::prayers::PRAYERS.iter().map(|p| p.key).collect::<Vec<_>>().join(", ")
                )));
            }
        }
        for key in p.readings.keys() {
            if !matches!(key.as_str(), "1" | "2" | "3" | "4" | "5") {
                return Err(Error::setup(format!("prayers.readings is keyed by mystery number, \"1\" to \"5\". \"{key}\" is not one.")));
            }
        }
        self.screen.size()?;
        if !(1..=120).contains(&self.screen.fps) {
            return Err(Error::setup("screen.fps must be between 1 and 120."));
        }
        if !(0.4..=2.0).contains(&self.beads.size) {
            return Err(Error::setup("beads.size must be between 0.4 and 2."));
        }
        let base = self.style();
        check_style(&base, "The setup")?;
        let base_value = serde_json::to_value(&base)?;
        for (i, rule) in self.rules.iter().enumerate() {
            let label = format!("Rule {}", i + 1);
            rule.target.check(&label)?;
            let style = apply_rule(&base_value, rule, &label)?;
            check_style(&style, &label)?;
        }
        Ok(())
    }
}

impl Target {
    fn check(&self, label: &str) -> Result<()> {
        if let Some(s) = &self.section {
            if !SECTIONS.contains(&s.as_str()) {
                return Err(Error::setup(format!("{label}: \"{s}\" is not a section. The sections are: {}.", SECTIONS.join(", "))));
            }
        }
        if let Some(m) = self.mystery {
            if !(1..=5).contains(&m) {
                return Err(Error::setup(format!("{label}: mystery must be 1 to 5. It is {m}.")));
            }
        }
        if let Some(p) = &self.prayer {
            if crate::prayers::prayer(p).is_none() && p != "reading" && p != "extra" {
                return Err(Error::setup(format!(
                    "{label}: \"{p}\" is not a kind of prayer. The kinds are: {}, reading, extra.",
                    crate::prayers::PRAYERS.iter().map(|p| p.key).collect::<Vec<_>>().join(", ")
                )));
            }
        }
        if let Some(b) = &self.bead {
            if !crate::beads::is_bead_id(b) {
                return Err(Error::setup(format!(
                    "{label}: \"{b}\" is not a bead. Beads are: cross, p1 to p5, medal, d1b1 to d5b10, L2 to L5, and g0 to g5 for the chain."
                )));
            }
        }
        Ok(())
    }
}

fn check_style(s: &Style, label: &str) -> Result<()> {
    let bad = |what: &str| Err(Error::setup(format!("{label}: {what}")));
    if !(0.4..=2.5).contains(&s.voice.speed) {
        return bad("voice.speed must be between 0.4 and 2.5. 1 is the pace of the first video.");
    }
    if let Some(x) = s.voice.exact_speedup {
        if !(0.5..=4.0).contains(&x) {
            return bad("voice.exactSpeedup must be between 0.5 and 4.");
        }
    }
    if !(0.5..=4.0).contains(&s.voice.recording_speed) {
        return bad("voice.recordingSpeed must be between 0.5 and 4.");
    }
    for (name, v) in [("stability", s.voice.stability), ("similarity", s.voice.similarity), ("style", s.voice.style)] {
        if let Some(v) = v {
            if !(0.0..=1.0).contains(&v) {
                return bad(&format!("voice.{name} must be between 0 and 1."));
            }
        }
    }
    if s.voice.on && !s.voice.has_id() && s.voice.names().is_empty() && s.voice.recording.is_none() {
        return bad("the voice is on but has no name. Set voice.name, or turn the voice off with voice.on = false.");
    }
    if s.voice.names().len() > 12 {
        return bad("voice.name lists more than 12 voices to take turns. 12 is the most.");
    }
    if !(1..=20).contains(&s.voice.takes) {
        return bad("voice.takes must be between 1 and 20. 1 is one recording of each prayer, used every time.");
    }
    if !(0.0..=5.0).contains(&s.pace.vary) {
        return bad("pace.vary must be between 0 and 5 seconds.");
    }
    for (name, v) in [
        ("before", s.pace.before),
        ("after", s.pace.after),
        ("announceGap", s.pace.announce_gap),
        ("think", s.pace.think),
        ("secondsPerLine", s.pace.seconds_per_line),
        ("minSeconds", s.pace.min_seconds),
        ("fade", s.pace.fade),
        ("intro", s.pace.intro),
        ("outro", s.pace.outro),
    ] {
        if !(0.0..=120.0).contains(&v) {
            return bad(&format!("pace.{name} must be between 0 and 120 seconds."));
        }
    }
    if !(0.4..=2.5).contains(&s.words.size) {
        return bad("words.size must be between 0.4 and 2.5. 1 is the size in the first video.");
    }
    if !(1.0..=5.0).contains(&s.picture.zoom) {
        return bad("picture.zoom must be between 1 and 5.");
    }
    Ok(())
}

/// One rule laid over a style.
pub fn apply_rule(style: &Value, rule: &Rule, label: &str) -> Result<Style> {
    let mut v = style.clone();
    overlay_rule(&mut v, rule, label)?;
    serde_json::from_value(v).map_err(|e| Error::setup(format!("{label}: {}", explain(&e.to_string()))))
}

pub fn overlay_rule(style: &mut Value, rule: &Rule, label: &str) -> Result<()> {
    match &rule.set {
        Value::Null => Ok(()),
        Value::Object(set) => {
            for key in set.keys() {
                if !STYLE_GROUPS.contains(&key.as_str()) {
                    return Err(Error::setup(format!(
                        "{label}: a rule can only change {}. It tries to change \"{key}\", which belongs to the whole video.",
                        STYLE_GROUPS.join(", ")
                    )));
                }
            }
            merge(style, &rule.set);
            Ok(())
        }
        _ => Err(Error::setup(format!("{label}: \"set\" has to be an object, like {{ \"voice\": {{ \"name\": \"…\" }} }}."))),
    }
}

// ───────────────────────── changing one thing ─────────────────────────

/// Read a value typed by a person: `true`, `1.5`, `"quoted"`, or plain words.
pub fn parse_value(raw: &str) -> Value {
    let t = raw.trim();
    match serde_json::from_str::<Value>(t) {
        Ok(v) => v,
        Err(_) => Value::String(t.to_string()),
    }
}

/// Set one thing by its dotted name: `voice.name`, `screen.preset`, `prayers.fatima`.
pub fn set_path(doc: &mut Value, path: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = path.split('.').map(str::trim).collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(Error::setup(format!("\"{path}\" is not a setting name. It should look like voice.name.")));
    }
    if !doc.is_object() {
        *doc = Value::Object(Map::new());
    }
    let mut cur = doc;
    for (i, part) in parts.iter().enumerate() {
        let obj = cur.as_object_mut().ok_or_else(|| Error::setup(format!("\"{path}\" does not fit: \"{}\" is not a group of settings.", parts[..i].join("."))))?;
        if i == parts.len() - 1 {
            obj.insert(part.to_string(), value);
            return Ok(());
        }
        let next = obj.entry(part.to_string()).or_insert_with(|| Value::Object(Map::new()));
        if !next.is_object() {
            *next = Value::Object(Map::new());
        }
        cur = next;
    }
    Ok(())
}

/// Read a rule typed on one line:
/// `mystery=3: voice.name=Bella` or `prayer=ourfather, section=decade-2: voice.speed=1.1; pace.after=1`.
pub fn parse_rule(text: &str) -> Result<Rule> {
    let (left, right) = text.split_once(':').ok_or_else(|| {
        Error::setup(format!("A rule needs a colon between where it applies and what it changes, like  mystery=3: voice.name=Bella . Got: {text}"))
    })?;
    let mut target = Map::new();
    for cond in left.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let (k, v) = cond.split_once('=').ok_or_else(|| Error::setup(format!("In a rule, \"{cond}\" should look like mystery=3.")))?;
        target.insert(k.trim().to_string(), parse_value(v));
    }
    let mut set = Value::Object(Map::new());
    for change in right.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (k, v) = change.split_once('=').ok_or_else(|| Error::setup(format!("In a rule, \"{change}\" should look like voice.name=Bella.")))?;
        set_path(&mut set, k.trim(), parse_value(v))?;
    }
    let rule: Rule = serde_json::from_value(serde_json::json!({ "for": target, "set": set }))
        .map_err(|e| Error::setup(format!("That rule has a mistake: {}.", explain(&e.to_string()))))?;
    Ok(rule)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_empty_setup_is_a_whole_setup() {
        let s = from_doc(&json!({})).unwrap();
        assert_eq!(s, Setup::default());
        assert_eq!(s.voice.name, "Imperial - Male");
        assert_eq!(s.screen.size().unwrap(), (2076, 2152));
        assert!(s.prayers.fatima && s.prayers.closing_prayer);
    }

    #[test]
    fn changing_one_thing_touches_one_thing() {
        let s = from_doc(&json!({ "voice": { "name": "Bella" } })).unwrap();
        let mut want = Setup::default();
        want.voice.name = "Bella".into();
        assert_eq!(s, want);
    }

    #[test]
    fn several_voices_in_one_name_take_turns() {
        let s = from_doc(&json!({ "voice": { "name": "Imperial - Male + Bella +  Roger - Laid-Back, Casual" } })).unwrap();
        assert_eq!(s.voice.names(), ["Imperial - Male", "Bella", "Roger - Laid-Back, Casual"], "a comma or a dash in a name is part of the name");
        assert_eq!(Setup::default().voice.names(), ["Imperial - Male"]);
        // The first video's way is still the default: one voice, one recording of each prayer.
        let v = Setup::default().voice;
        assert!(v.takes == 1 && !v.fresh_each_time && !v.shuffle);
        assert_eq!(Setup::default().pace.vary, 0.0);
        let err = from_doc(&json!({ "voice": { "takes": 0 } })).unwrap_err().to_string();
        assert!(err.contains("voice.takes"), "{err}");
    }

    #[test]
    fn a_typo_is_named() {
        let err = from_doc(&json!({ "voice": { "nmae": "Bella" } })).unwrap_err().to_string();
        assert!(err.contains("nmae"), "{err}");
        let err = from_doc(&json!({ "screen": { "preset": "tv-9000" } })).unwrap_err().to_string();
        assert!(err.contains("tv-9000") && err.contains("tv-1080p"), "{err}");
    }

    #[test]
    fn a_child_adds_rules_to_its_parent() {
        let parent = json!({ "voice": { "name": "A" }, "rules": [{ "for": { "mystery": 1 }, "set": { "voice": { "name": "B" } } }] });
        let child = json!({ "pace": { "after": 1.0 }, "rules": [{ "for": { "mystery": 2 }, "set": { "voice": { "name": "C" } } }] });
        let merged = merge_docs(&parent, &child);
        assert_eq!(merged["rules"].as_array().unwrap().len(), 2);
        assert_eq!(merged["voice"]["name"], "A");
        assert_eq!(merged["pace"]["after"], 1.0);
    }

    #[test]
    fn null_puts_a_setting_back() {
        let parent = json!({ "voice": { "name": "A", "speed": 1.2 } });
        let merged = merge_docs(&parent, &json!({ "voice": { "name": null } }));
        let s = from_doc(&merged).unwrap();
        assert_eq!(s.voice.name, "Imperial - Male");
        assert_eq!(s.voice.speed, 1.2);
    }

    #[test]
    fn rules_can_only_touch_what_narrows() {
        let err = from_doc(&json!({ "rules": [{ "for": { "mystery": 2 }, "set": { "screen": { "preset": "square" } } }] })).unwrap_err().to_string();
        assert!(err.contains("Rule 1") && err.contains("screen"), "{err}");
        let err = from_doc(&json!({ "rules": [{ "for": { "bead": "d9b1" }, "set": {} }] })).unwrap_err().to_string();
        assert!(err.contains("d9b1"), "{err}");
    }

    #[test]
    fn one_line_rules() {
        let r = parse_rule("mystery=3: voice.name=Bella").unwrap();
        assert_eq!(r.target.mystery, Some(3));
        assert_eq!(r.set, json!({ "voice": { "name": "Bella" } }));
        let r = parse_rule("prayer=ourfather, section=decade-2: voice.speed=1.1; pace.after=1").unwrap();
        assert_eq!(r.target.prayer.as_deref(), Some("ourfather"));
        assert_eq!(r.target.section.as_deref(), Some("decade-2"));
        assert_eq!(r.set, json!({ "voice": { "speed": 1.1 }, "pace": { "after": 1 } }));
        assert!(parse_rule("no colon here").is_err());
    }

    #[test]
    fn set_by_dotted_name() {
        let mut doc = json!({});
        set_path(&mut doc, "voice.name", parse_value("Imperial - Male")).unwrap();
        set_path(&mut doc, "prayers.fatima", parse_value("false")).unwrap();
        set_path(&mut doc, "screen.safe.top", parse_value("120")).unwrap();
        assert_eq!(doc, json!({ "voice": { "name": "Imperial - Male" }, "prayers": { "fatima": false }, "screen": { "safe": { "top": 120 } } }));
    }

    #[test]
    fn narrowness_follows_the_list() {
        let t = |v: Value| serde_json::from_value::<Target>(v).unwrap().level();
        assert_eq!(t(json!({})), 1);
        assert_eq!(t(json!({ "section": "opening" })), 2);
        assert_eq!(t(json!({ "mystery": 3 })), 3);
        assert_eq!(t(json!({ "prayer": "hailmary" })), 4);
        assert_eq!(t(json!({ "bead": "d3b10" })), 5);
        assert_eq!(t(json!({ "mystery": 3, "prayer": "hailmary" })), 4);
    }

    #[test]
    fn the_dark_colors_are_the_first_video() {
        let p = Colors::default().palette();
        assert_eq!(p.background, "#0E1626");
        assert_eq!(p.marker, "#E6B556");
        assert_eq!(p.title_font, "Cardo");
    }
}
