//! Turn a setup into a plan: every scene of the video, in order, with the
//! narrowest choice already picked for each one.
//!
//! The order of prayers is the table in CLAUDE.md. It is checked by the tests at
//! the bottom of this file.

use crate::beads::{self, Drawing};
use crate::error::Result;
use crate::prayers::{self, ORDINAL, ROMAN, VIRTUES};
use crate::setup::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The pace of the first video, in letters per second of speech. Worked out from
/// its length (13 min 10 s), its pauses and its words. Voice speed 1 aims for this.
pub const FIRST_VIDEO_PACE: f32 = 23.5;

/// What the plan needs to know about the world outside the setup.
#[derive(Clone, Debug)]
pub struct Context {
    /// Decides the mysteries when the setup says "today".
    pub today: chrono::NaiveDate,
    /// Folders to look in for pictures and recordings given as relative paths.
    pub search: Vec<PathBuf>,
}

impl Context {
    pub fn now(search: Vec<PathBuf>) -> Self {
        Context { today: chrono::Local::now().date_naive(), search }
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneKind {
    Intro,
    Prayer,
    Outro,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Announce {
    /// "The first Luminous Mystery"
    pub lead: String,
    /// "The Baptism in the Jordan"
    pub say: String,
}

/// The text inside the loop of beads when no picture is there.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Center {
    pub label: String,
    pub number: Option<String>,
    pub name: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PictureDraw {
    /// Full path of the file on this computer.
    pub file: String,
    pub focus: String,
    pub zoom: f32,
    pub credit: String,
    pub place: PicturePlace,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    /// "intro", "000", "001", … "outro". Also the name of its still picture.
    pub id: String,
    pub kind: SceneKind,
    /// Prayer number, counting from 1.
    pub number: Option<usize>,
    pub section: String,
    pub mystery: Option<u8>,
    pub prayer: Option<String>,
    pub bead: Option<String>,

    // What is drawn.
    pub place: String,
    pub title: String,
    pub lines: Vec<String>,
    pub show_lines: bool,
    pub pip: Option<[u32; 2]>,
    pub label: Option<String>,
    pub announce: Option<Announce>,
    pub hint: Option<String>,
    pub count: String,
    pub progress: f32,
    pub center: Center,
    pub picture: Option<PictureDraw>,
    pub words: Words,
    pub colors: Palette,

    // What is heard.
    pub voice: Voice,
    pub pace: Pace,
    /// The mystery, named aloud before the prayer.
    pub say_first: Option<String>,
    /// With two voices, the line where the second one comes in.
    pub response_from: Option<usize>,
    /// The voice has dropped out here. The bead still gets its time.
    pub muted: bool,
    pub chapter: Option<String>,
    /// A guess at how long this scene lasts, before any sound is made.
    pub seconds: f32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub width: u32,
    pub height: u32,
    pub safe: SafeEdges,
    pub fps: u32,
    pub drawing: Drawing,
    pub beads: Beads,
    /// For each bead, the first scene that lands on it. Beads before the current
    /// scene are drawn as done.
    pub first_scene: BTreeMap<String, usize>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub name: String,
    pub set_key: String,
    pub set_name: String,
    pub stage: Stage,
    pub scenes: Vec<Scene>,
    pub prayer_count: usize,
    /// A guess at the length of the video, in seconds.
    pub seconds: f32,
    /// Things worth telling the person, like a picture that could not be found.
    pub warnings: Vec<String>,
}

struct MysteryInfo {
    name: String,
    reference: Option<String>,
}

struct SetInfo {
    key: String,
    name: String,
    days: String,
    items: Vec<MysteryInfo>,
}

fn set_info(setup: &Setup, ctx: &Context) -> SetInfo {
    let key = match setup.prayers.mysteries {
        Mysteries::Today => prayers::set_for_date(ctx.today),
        Mysteries::Joyful => "joyful",
        Mysteries::Sorrowful => "sorrowful",
        Mysteries::Glorious => "glorious",
        Mysteries::Luminous => "luminous",
        Mysteries::Custom => {
            let c = &setup.prayers.custom;
            return SetInfo {
                key: "custom".into(),
                name: if c.name.trim().is_empty() { "My".into() } else { c.name.trim().to_string() },
                days: String::new(),
                items: c.items.iter().map(|n| MysteryInfo { name: n.clone(), reference: None }).collect(),
            };
        }
    };
    let set = prayers::set(key).expect("built-in set");
    SetInfo {
        key: key.into(),
        name: set.name.into(),
        days: set.days.into(),
        items: set.items.iter().map(|m| MysteryInfo { name: m.name.into(), reference: m.reference.map(Into::into) }).collect(),
    }
}

/// One prayer on one bead, before any style is chosen for it.
struct Step {
    section: String,
    mystery: Option<u8>,
    prayer: String,
    bead: String,
    place: String,
    title: String,
    lines: Vec<String>,
    pip: Option<[u32; 2]>,
    label: Option<String>,
    hint: Option<String>,
    /// Name the mystery aloud before this one.
    announce: bool,
    response_from: Option<usize>,
}

fn steps(setup: &Setup, set: &SetInfo) -> Vec<Step> {
    let p = &setup.prayers;
    let text = |key: &str| -> (String, Vec<String>, Option<usize>) {
        let base = prayers::prayer(key).expect("built-in prayer");
        match p.wording.get(key) {
            // A person's own wording has no known place for the second voice to come in.
            Some(own) if !own.is_empty() => (base.title.to_string(), own.clone(), None),
            _ => (base.title.to_string(), base.lines.iter().map(|s| s.to_string()).collect(), base.response_from),
        }
    };
    let mut out: Vec<Step> = Vec::new();
    let mut add = |section: &str, mystery: Option<u8>, key: &str, bead: &str, place: &str| {
        let (title, lines, response_from) = text(key);
        out.push(Step {
            section: section.into(),
            mystery,
            prayer: key.into(),
            bead: bead.into(),
            place: place.into(),
            title,
            lines,
            pip: None,
            label: None,
            hint: None,
            announce: false,
            response_from,
        });
    };
    let on_chain = p.glory_be == GloryBePlace::Chain;

    if p.opening {
        add("opening", None, "sign", "cross", "Hold the crucifix");
        add("opening", None, "creed", "cross", "Still holding the crucifix");
        add("opening", None, "ourfather", "p1", "First big bead");
        for i in 0..3 {
            add("opening", None, "hailmary", &format!("p{}", i + 2), &format!("Small bead {} of 3", i + 1));
        }
        if on_chain {
            add("opening", None, "glorybe", "g0", "No bead for this one. Hold the chain before the next big bead.");
        } else {
            add("opening", None, "glorybe", "p5", "On the second big bead, before the Our Father");
        }
    }
    for d in p.first_decade..=p.last_decade {
        let section = format!("decade-{d}");
        let big = if d == 1 { "p5".to_string() } else { format!("L{d}") };
        let place = if d == 1 { "Second big bead, just below the medal" } else { "Big bead" };
        add(&section, Some(d), "ourfather", &big, place);
        for b in 1..=10 {
            add(&section, Some(d), "hailmary", &format!("d{d}b{b}"), &format!("Small bead {b} of 10"));
        }
        let (bead, glory_place, fatima_place) = if on_chain {
            (format!("g{d}"), "No bead for this one. Hold the chain after the tenth small bead.", "Still on the chain")
        } else if d < 5 {
            (format!("L{}", d + 1), "On the next big bead, before the Our Father", "Still on the big bead")
        } else {
            ("medal".to_string(), "On the medal", "Still on the medal")
        };
        add(&section, Some(d), "glorybe", &bead, glory_place);
        if p.fatima {
            add(&section, Some(d), "fatima", &bead, fatima_place);
        }
    }
    if p.closing {
        add("closing", None, "hhq", "medal", "You are back at the medal");
        if p.closing_prayer {
            add("closing", None, "closing", "medal", "Still on the medal");
        }
        add("closing", None, "sign", "cross", "Finish on the crucifix");
    }
    drop(add);

    // Fill in what the plain list above leaves out.
    let mut opening_hail_marys = 0;
    let mut decade_bead = 0;
    for s in out.iter_mut() {
        match (s.section.as_str(), s.prayer.as_str()) {
            ("opening", "hailmary") => {
                s.pip = Some([opening_hail_marys + 1, 3]);
                s.hint = Some(format!("Many people offer this one for {}.", VIRTUES[opening_hail_marys as usize]));
                opening_hail_marys += 1;
            }
            (_, "ourfather") if s.mystery.is_some() => {
                s.announce = true;
                decade_bead = 0;
            }
            (_, "hailmary") if s.mystery.is_some() => {
                decade_bead += 1;
                s.pip = Some([decade_bead, 10]);
            }
            (_, "fatima") => s.label = Some("Optional. Many people add this after each Glory Be.".into()),
            (_, "closing") => s.label = Some("Optional.".into()),
            _ => {}
        }
    }

    // Extra prayers a person added, and readings they supplied.
    let extra = |e: &ExtraPrayer, section: &str, bead: &str, place: &str| Step {
        section: section.into(),
        mystery: None,
        prayer: "extra".into(),
        bead: bead.into(),
        place: place.into(),
        title: e.title.clone(),
        lines: e.lines.clone(),
        pip: None,
        label: None,
        hint: None,
        announce: false,
        response_from: None,
    };
    if p.opening && !p.extra_start.is_empty() {
        let at = out.iter().position(|s| s.prayer == "sign").map(|i| i + 1).unwrap_or(0);
        let extras: Vec<Step> = p.extra_start.iter().map(|e| extra(e, "opening", "cross", "Still holding the crucifix")).collect();
        out.splice(at..at, extras);
    }
    if p.closing && !p.extra_end.is_empty() {
        let at = out.len() - 1;
        let extras: Vec<Step> = p.extra_end.iter().map(|e| extra(e, "closing", "medal", "Still on the medal")).collect();
        out.splice(at..at, extras);
    }
    for d in p.first_decade..=p.last_decade {
        let Some(reading) = p.readings.get(&d.to_string()).filter(|t| !t.trim().is_empty()) else { continue };
        let Some(at) = out.iter().position(|s| s.mystery == Some(d) && s.prayer == "ourfather") else { continue };
        let info = &set.items[(d - 1) as usize];
        out[at].announce = false;
        let step = Step {
            section: format!("decade-{d}"),
            mystery: Some(d),
            prayer: "reading".into(),
            bead: out[at].bead.clone(),
            place: info.reference.clone().unwrap_or_else(|| "A reading".into()),
            title: "A Reading".into(),
            lines: reading.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect(),
            pip: None,
            label: None,
            hint: Some("Listen, then pray the Our Father on this same bead.".into()),
            announce: true,
            response_from: None,
        };
        out.insert(at, step);
    }
    out
}

/// The rules that cover a scene, laid over the whole-video style, widest first.
struct Cascade<'a> {
    base: Value,
    /// Rules in the order they apply. Later ones win.
    ordered: Vec<(usize, &'a Rule)>,
}

impl<'a> Cascade<'a> {
    fn new(setup: &'a Setup) -> Result<Self> {
        let mut ordered: Vec<(usize, &Rule)> = setup.rules.iter().enumerate().collect();
        // Narrowest last. Among equals, the one with more conditions, then the later one.
        ordered.sort_by_key(|(i, r)| (r.target.level(), r.target.conditions(), *i));
        Ok(Cascade { base: serde_json::to_value(setup.style())?, ordered })
    }

    fn style(&self, section: &str, mystery: Option<u8>, prayer: Option<&str>, bead: Option<&str>) -> Result<Style> {
        let mut v = self.base.clone();
        for (i, rule) in &self.ordered {
            let t = &rule.target;
            let hit = t.section.as_deref().is_none_or(|s| s == section)
                && t.mystery.is_none_or(|m| mystery == Some(m))
                && t.prayer.as_deref().is_none_or(|p| prayer == Some(p))
                && t.bead.as_deref().is_none_or(|b| bead == Some(b));
            if hit {
                overlay_rule(&mut v, rule, &format!("Rule {}", i + 1))?;
            }
        }
        Ok(serde_json::from_value(v)?)
    }
}

/// Find a file a setup names. Tries the name as given, then the search folders,
/// then the common picture endings if none was typed.
pub fn find_file(name: &str, search: &[PathBuf], endings: &[&str]) -> Option<PathBuf> {
    let given = Path::new(name);
    let mut bases: Vec<PathBuf> = Vec::new();
    if given.is_absolute() {
        bases.push(given.to_path_buf());
    } else {
        for dir in search {
            bases.push(dir.join(given));
        }
    }
    for base in &bases {
        if base.is_file() {
            return Some(base.clone());
        }
        for ext in endings {
            let mut with = base.as_os_str().to_owned();
            with.push(".");
            with.push(ext);
            let with = PathBuf::from(with);
            if with.is_file() {
                return Some(with);
            }
        }
    }
    None
}

pub const PICTURE_ENDINGS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "avif"];

fn spoken_seconds(lines: &[String], speed: f32) -> f32 {
    let letters: usize = lines.iter().map(|l| l.chars().count() + 1).sum();
    letters as f32 / (FIRST_VIDEO_PACE * speed.max(0.1))
}

pub fn build(setup: &Setup, ctx: &Context) -> Result<Plan> {
    setup.check()?;
    let set = set_info(setup, ctx);
    let steps = steps(setup, &set);
    let cascade = Cascade::new(setup)?;
    let total = steps.len();
    let (width, height) = setup.screen.size()?;
    let mut warnings: Vec<String> = Vec::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let set_label = format!("{} Mysteries", set.name);

    let mut picture = |style: &Style, _warnings: &mut Vec<String>| -> Option<PictureDraw> {
        let p = &style.picture;
        let file = p.file.as_deref().map(str::trim).filter(|f| !f.is_empty())?;
        if !p.show {
            return None;
        }
        match find_file(file, &ctx.search, &PICTURE_ENDINGS) {
            Some(path) => Some(PictureDraw {
                file: path.to_string_lossy().replace('\\', "/"),
                focus: p.focus.clone(),
                zoom: p.zoom,
                credit: if p.show_credit { p.credit.clone() } else { String::new() },
                place: p.place,
            }),
            None => {
                missing.insert(file.to_string());
                None
            }
        }
    };

    let mut scenes: Vec<Scene> = Vec::with_capacity(total + 2);

    // The opening title.
    let style = cascade.style("intro", None, None, None)?;
    let whole_rosary = setup.prayers.opening && setup.prayers.closing && setup.prayers.first_decade == 1 && setup.prayers.last_decade == 5;
    let default_marker = Colors { mode: style.colors.mode, ..Colors::default() }.palette().marker;
    let marker_word = if style.colors.palette().marker.eq_ignore_ascii_case(&default_marker) { "gold" } else { "bright" };
    let mut intro_lines = vec![format!("Follow the {marker_word} bead.")];
    if setup.voice.on {
        intro_lines.push("Pray along with the voice.".into());
    }
    scenes.push(Scene {
        id: "intro".into(),
        kind: SceneKind::Intro,
        number: None,
        section: "intro".into(),
        mystery: None,
        prayer: None,
        bead: None,
        place: set_label.clone(),
        title: setup.prayers.title.clone(),
        lines: intro_lines,
        show_lines: true,
        pip: None,
        label: None,
        announce: None,
        hint: None,
        count: format!("{total} prayers"),
        progress: 0.0,
        center: Center { label: set_label.clone(), number: None, name: if set.days.is_empty() { String::new() } else { format!("Prayed on {}", set.days) } },
        picture: picture(&style, &mut warnings),
        colors: style.colors.palette(),
        words: style.words,
        voice: style.voice,
        pace: style.pace,
        say_first: None,
        response_from: None,
        muted: false,
        chapter: Some(setup.prayers.title.clone()),
        seconds: setup.pace.intro,
    });

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut last_section = String::new();
    for (i, st) in steps.into_iter().enumerate() {
        let style = cascade.style(&st.section, st.mystery, Some(&st.prayer), Some(&st.bead))?;
        let info = st.mystery.map(|d| &set.items[(d - 1) as usize]);
        let first_time = seen.insert(if st.prayer == "extra" || st.prayer == "reading" { format!("{}:{}", st.prayer, st.title) } else { st.prayer.clone() });
        let show_lines = match style.words.text {
            TextMode::Always => true,
            TextMode::First => first_time,
            TextMode::Never => false,
        };
        let announce = match (info, st.mystery, style.words.mystery_name) {
            (Some(info), Some(d), true) => Some(Announce { lead: format!("The {} {} Mystery", ORDINAL[(d - 1) as usize], set.name), say: info.name.clone() }),
            _ => None,
        };
        let say_first = match (info, st.mystery) {
            (Some(info), Some(d)) if st.announce && style.voice.on && style.voice.announce => {
                Some(format!("The {} {} Mystery. {}.", ORDINAL[(d - 1) as usize], set.name, info.name.trim_end_matches('.')))
            }
            _ => None,
        };
        let center = match (info, st.mystery) {
            (Some(info), Some(d)) => Center { label: set_label.clone(), number: Some(ROMAN[(d - 1) as usize].into()), name: info.name.clone() },
            _ => Center { label: set_label.clone(), number: None, name: if st.section == "opening" { "Opening prayers".into() } else { "Closing prayers".into() } },
        };
        let muted = style.voice.drop_after > 0 && st.mystery.is_some() && st.prayer == "hailmary" && st.pip.is_some_and(|p| p[0] > style.voice.drop_after);
        let chapter = if st.section != last_section {
            last_section = st.section.clone();
            Some(match (info, st.mystery) {
                (Some(info), Some(d)) => format!("{}. {}", ROMAN[(d - 1) as usize], info.name),
                _ if st.section == "opening" => "Opening prayers".into(),
                _ => "Closing prayers".into(),
            })
        } else {
            None
        };
        let seconds = if style.voice.on {
            let mut s = style.pace.before + spoken_seconds(&st.lines, style.voice.speed) + style.pace.after;
            if say_first.is_some() {
                s += 2.4 / style.voice.speed.max(0.1) + style.pace.announce_gap + style.pace.think;
            }
            s
        } else {
            (st.lines.len() as f32 * style.pace.seconds_per_line).max(style.pace.min_seconds)
        };
        scenes.push(Scene {
            id: format!("{i:03}"),
            kind: SceneKind::Prayer,
            number: Some(i + 1),
            place: if style.words.place {
                match st.mystery {
                    Some(d) => format!("Decade {d} · {}", st.place),
                    None => st.place,
                }
            } else {
                String::new()
            },
            section: st.section,
            mystery: st.mystery,
            prayer: Some(st.prayer),
            bead: Some(st.bead),
            title: st.title,
            lines: st.lines,
            show_lines,
            pip: st.pip.filter(|_| style.words.counter),
            label: st.label.filter(|_| style.words.optional_label),
            announce,
            hint: st.hint.filter(|_| style.words.hints),
            count: format!("Prayer {} of {total}", i + 1),
            progress: (i + 1) as f32 / total as f32,
            center,
            picture: picture(&style, &mut warnings),
            colors: style.colors.palette(),
            words: style.words,
            voice: style.voice,
            pace: style.pace,
            say_first,
            response_from: st.response_from,
            muted,
            chapter,
            seconds,
        });
    }

    // The ending.
    let style = cascade.style("outro", None, None, None)?;
    scenes.push(Scene {
        id: "outro".into(),
        kind: SceneKind::Outro,
        number: None,
        section: "outro".into(),
        mystery: None,
        prayer: None,
        bead: None,
        place: "Finished".into(),
        title: "Amen.".into(),
        lines: vec![if whole_rosary { "That is the whole rosary.".into() } else { "That is the end of this part.".into() }],
        show_lines: true,
        pip: None,
        label: None,
        announce: None,
        hint: None,
        count: format!("All {total} prayers said"),
        progress: 1.0,
        center: Center { label: set_label.clone(), number: None, name: "Finished".into() },
        picture: picture(&style, &mut warnings),
        colors: style.colors.palette(),
        words: style.words,
        voice: style.voice,
        pace: style.pace,
        say_first: None,
        response_from: None,
        muted: false,
        chapter: None,
        seconds: setup.pace.outro,
    });

    drop(picture);
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(String::as_str).collect();
        warnings.push(format!(
            "{} not found: {}. The loop of beads will hold the mystery's name there instead.",
            if names.len() == 1 { "Picture" } else { "Pictures" },
            names.join(", ")
        ));
    }

    let mut first_scene: BTreeMap<String, usize> = BTreeMap::new();
    for (i, sc) in scenes.iter().enumerate() {
        if let Some(b) = &sc.bead {
            first_scene.entry(b.clone()).or_insert(i);
        }
    }
    let seconds = scenes.iter().map(|s| s.seconds).sum();
    let name = if setup.name.trim().is_empty() { format!("{} Mysteries", set.name) } else { setup.name.trim().to_string() };
    Ok(Plan {
        name,
        set_key: set.key,
        set_name: set.name,
        stage: Stage {
            width,
            height,
            safe: setup.screen.safe,
            fps: setup.screen.fps,
            drawing: beads::drawing(setup.beads.kind, setup.beads.direction),
            beads: setup.beads.clone(),
            first_scene,
        },
        scenes,
        prayer_count: total,
        seconds,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::from_doc;
    use serde_json::json;

    fn ctx() -> Context {
        // A Thursday, so "today" means Luminous.
        Context { today: chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(), search: vec![] }
    }

    fn plan(doc: serde_json::Value) -> Plan {
        build(&from_doc(&doc).unwrap(), &ctx()).unwrap()
    }

    fn prayers_of(p: &Plan) -> Vec<(String, String)> {
        p.scenes.iter().filter(|s| s.kind == SceneKind::Prayer).map(|s| (s.bead.clone().unwrap(), s.prayer.clone().unwrap())).collect()
    }

    #[test]
    fn the_whole_rosary_is_75_prayers_in_the_right_order() {
        let p = plan(json!({}));
        let got = prayers_of(&p);
        assert_eq!(got.len(), 75);
        assert_eq!(p.scenes.len(), 77, "75 prayers, an opening title and an ending");

        // The table in CLAUDE.md, row by row.
        let mut want: Vec<(String, String)> = Vec::new();
        let mut row = |bead: &str, prayer: &str| want.push((bead.to_string(), prayer.to_string()));
        row("cross", "sign");
        row("cross", "creed");
        row("p1", "ourfather");
        row("p2", "hailmary");
        row("p3", "hailmary");
        row("p4", "hailmary");
        row("g0", "glorybe");
        for d in 1..=5 {
            row(&if d == 1 { "p5".to_string() } else { format!("L{d}") }, "ourfather");
            for b in 1..=10 {
                row(&format!("d{d}b{b}"), "hailmary");
            }
            row(&format!("g{d}"), "glorybe");
            row(&format!("g{d}"), "fatima");
        }
        row("medal", "hhq");
        row("medal", "closing");
        row("cross", "sign");
        assert_eq!(got, want);

        let count = |k: &str| got.iter().filter(|(_, p)| p == k).count();
        assert_eq!(count("hailmary"), 53);
        assert_eq!(count("ourfather"), 6);
        assert_eq!(count("glorybe"), 6);
    }

    #[test]
    fn every_bead_is_prayed_on() {
        let p = plan(json!({}));
        for spot in &p.stage.drawing.spots {
            assert!(p.stage.first_scene.contains_key(&spot.id), "nothing is prayed on {}", spot.id);
        }
    }

    #[test]
    fn optional_prayers_are_labelled_and_can_be_left_out() {
        let p = plan(json!({}));
        let labelled: Vec<&str> = p.scenes.iter().filter(|s| s.label.is_some()).map(|s| s.prayer.as_deref().unwrap()).collect();
        assert_eq!(labelled.iter().filter(|k| **k == "fatima").count(), 5);
        assert_eq!(labelled.iter().filter(|k| **k == "closing").count(), 1);
        assert_eq!(labelled.len(), 6);

        let p = plan(json!({ "prayers": { "fatima": false, "closingPrayer": false } }));
        assert_eq!(p.prayer_count, 69);
        assert!(p.scenes.iter().all(|s| s.label.is_none()));
    }

    #[test]
    fn today_picks_the_mysteries_for_the_day() {
        assert_eq!(plan(json!({})).set_key, "luminous");
        assert_eq!(plan(json!({ "prayers": { "mysteries": "joyful" } })).set_key, "joyful");
        let p = plan(json!({ "prayers": { "mysteries": "sorrowful" } }));
        let first = p.scenes.iter().find(|s| s.say_first.is_some()).unwrap();
        assert_eq!(first.say_first.as_deref(), Some("The first Sorrowful Mystery. The Agony in the Garden."));
        assert_eq!(first.prayer.as_deref(), Some("ourfather"));
        assert_eq!(p.scenes.iter().filter(|s| s.say_first.is_some()).count(), 5);
    }

    #[test]
    fn the_narrowest_choice_wins() {
        // The example in docs/what-you-can-change.md.
        let p = plan(json!({
            "voice": { "name": "One" },
            "rules": [
                { "for": { "bead": "cross", "section": "closing" }, "set": { "voice": { "name": "Four" } } },
                { "for": { "mystery": 3 }, "set": { "voice": { "name": "Three" } } },
                { "for": { "prayer": "ourfather" }, "set": { "voice": { "name": "Two" } } }
            ]
        }));
        let voice = |bead: &str, prayer: &str| p.scenes.iter().rev().find(|s| s.bead.as_deref() == Some(bead) && s.prayer.as_deref() == Some(prayer)).unwrap().voice.name.clone();
        assert_eq!(voice("d1b4", "hailmary"), "One", "whole video");
        assert_eq!(voice("d3b4", "hailmary"), "Three", "one mystery beats the whole video");
        assert_eq!(voice("L2", "ourfather"), "Two", "one kind of prayer");
        assert_eq!(voice("L3", "ourfather"), "Two", "a kind of prayer is narrower than a mystery");
        assert_eq!(voice("cross", "sign"), "Four", "one bead beats everything");
        // The other settings are not disturbed.
        assert!(p.scenes.iter().all(|s| s.voice.speed == 1.0 && s.pace.after == 0.70));
    }

    #[test]
    fn a_section_is_wider_than_a_mystery() {
        let p = plan(json!({ "rules": [
            { "for": { "mystery": 2 }, "set": { "words": { "size": 1.2 } } },
            { "for": { "section": "decade-2" }, "set": { "words": { "size": 0.8 } } }
        ] }));
        let s = p.scenes.iter().find(|s| s.bead.as_deref() == Some("d2b1")).unwrap();
        assert_eq!(s.words.size, 1.2);
    }

    #[test]
    fn one_decade_on_its_own() {
        let p = plan(json!({ "prayers": { "opening": false, "closing": false, "firstDecade": 3, "lastDecade": 3 } }));
        assert_eq!(p.prayer_count, 13);
        let first = &p.scenes[1];
        assert_eq!((first.bead.as_deref(), first.prayer.as_deref()), (Some("L3"), Some("ourfather")));
        assert_eq!(p.scenes.last().unwrap().lines[0], "That is the end of this part.");
    }

    #[test]
    fn glory_be_on_the_next_big_bead() {
        let p = plan(json!({ "prayers": { "gloryBe": "next-bead" } }));
        let glory: Vec<&str> = p.scenes.iter().filter(|s| s.prayer.as_deref() == Some("glorybe")).map(|s| s.bead.as_deref().unwrap()).collect();
        assert_eq!(glory, ["p5", "L2", "L3", "L4", "L5", "medal"]);
    }

    #[test]
    fn the_voice_can_drop_out_and_text_can_show_once() {
        let p = plan(json!({ "voice": { "dropAfter": 3 }, "words": { "text": "first" } }));
        let decade2: Vec<&Scene> = p.scenes.iter().filter(|s| s.mystery == Some(2) && s.prayer.as_deref() == Some("hailmary")).collect();
        assert_eq!(decade2.iter().filter(|s| s.muted).count(), 7);
        assert!(!decade2[2].muted && decade2[3].muted);
        // The three opening Hail Marys are never dropped.
        assert!(p.scenes.iter().filter(|s| s.section == "opening").all(|s| !s.muted));
        // The Hail Mary's words show the first time only. The lines are still there for the voice.
        let hail: Vec<&Scene> = p.scenes.iter().filter(|s| s.prayer.as_deref() == Some("hailmary")).collect();
        assert!(hail[0].show_lines && !hail[1].show_lines && hail[1].lines.len() == 8);
    }

    #[test]
    fn own_wording_extras_and_readings() {
        let p = plan(json!({ "prayers": {
            "mysteries": "luminous",
            "wording": { "glorybe": ["Gloria Patri."] },
            "extraStart": [{ "title": "My intention", "lines": ["For my family."] }],
            "readings": { "1": "First line.\nSecond line." }
        } }));
        assert_eq!(p.prayer_count, 77);
        assert_eq!(p.scenes[2].title, "My intention");
        let glory = p.scenes.iter().find(|s| s.prayer.as_deref() == Some("glorybe")).unwrap();
        assert_eq!(glory.lines, ["Gloria Patri."]);
        let reading = p.scenes.iter().find(|s| s.prayer.as_deref() == Some("reading")).unwrap();
        assert_eq!(reading.place, "Decade 1 · Matthew 3:13-17");
        assert_eq!(reading.lines.len(), 2);
        assert!(reading.say_first.is_some(), "the mystery is named before the reading");
        let our_father = p.scenes.iter().find(|s| s.mystery == Some(1) && s.prayer.as_deref() == Some("ourfather")).unwrap();
        assert!(our_father.say_first.is_none(), "and not named twice");
    }

    #[test]
    fn a_missing_picture_is_a_warning_not_a_failure() {
        let p = plan(json!({ "rules": [{ "for": { "mystery": 1 }, "set": { "picture": { "file": "pictures/nowhere", "credit": "Someone" } } }] }));
        assert_eq!(p.warnings.len(), 1);
        assert!(p.scenes.iter().all(|s| s.picture.is_none()));
    }

    #[test]
    fn the_first_video_is_about_thirteen_minutes() {
        let p = plan(json!({ "prayers": { "mysteries": "luminous" } }));
        assert!((740.0..840.0).contains(&p.seconds), "guessed {} seconds", p.seconds);
    }
}
