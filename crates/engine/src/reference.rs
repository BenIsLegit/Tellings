//! Every setting, with its choices, in plain words.
//!
//! This is what `pvc options` prints and what `docs/settings.md` is made from.
//! A test keeps it honest: a setting cannot be added to the tool without being
//! described here, and nothing here can outlive its setting.

pub struct Item {
    pub path: &'static str,
    /// The choices, or the kind of value.
    pub takes: &'static str,
    pub what: &'static str,
}

pub struct Group {
    pub title: &'static str,
    /// True when rules can set this group for part of the video.
    pub narrows: bool,
    pub items: &'static [Item],
}

const fn i(path: &'static str, takes: &'static str, what: &'static str) -> Item {
    Item { path, takes, what }
}

pub const GROUPS: &[Group] = &[
    Group {
        title: "The setup itself",
        narrows: false,
        items: &[
            i("name", "words", "The name of the setup. Also the name of the video file."),
            i("description", "words", "A line about it, shown in the list."),
        ],
    },
    Group {
        title: "The beads",
        narrows: false,
        items: &[
            i("beads.kind", "five-decade-rosary", "Which beads. Only the five-decade rosary is built so far."),
            i("beads.medal", "plain | st-benedict | miraculous", "The medal where the strand joins the loop."),
            i("beads.cross", "latin | st-benedict | orthodox | greek | knotted | tassel | celtic | san-damiano | medal", "What hangs at the end."),
            i("beads.style", "outline | wood | glass | stone | pearl | metal | knots", "How the beads look. outline is the first video."),
            i("beads.color", "a color, like #9A6B43", "Bead color. Leave out for the usual color of the style."),
            i("beads.size", "0.4 to 2", "Bead size. 1 is the first video."),
            i("beads.cord", "chain | cord", "A fine chain or a thicker cord."),
            i("beads.cordColor", "a color", "Color of the chain or cord."),
            i("beads.direction", "right-first | left-first", "Which way around the loop you go."),
        ],
    },
    Group {
        title: "The prayers",
        narrows: false,
        items: &[
            i("prayers.title", "words", "The big title on the opening card."),
            i("prayers.mysteries", "today | joyful | sorrowful | glorious | luminous | custom", "Which mysteries. today picks by the day of the week."),
            i("prayers.custom.name", "words", "For your own mysteries: shown as \"<name> Mysteries\"."),
            i("prayers.custom.items", "a list of five names", "For your own mysteries: one name for each decade."),
            i("prayers.opening", "true | false", "Start at the crucifix (true), or go straight to the first decade (false)."),
            i("prayers.closing", "true | false", "End with the Hail, Holy Queen and the Sign of the Cross."),
            i("prayers.firstDecade", "1 to 5", "The decade to start on."),
            i("prayers.lastDecade", "1 to 5", "The decade to end on."),
            i("prayers.fatima", "true | false", "O My Jesus after each Glory Be. Optional in the rosary."),
            i("prayers.closingPrayer", "true | false", "The Closing Prayer after the Hail, Holy Queen. Optional."),
            i("prayers.gloryBe", "chain | next-bead", "Where the Glory Be goes: on the chain, or on the next big bead."),
            i("prayers.wording", "{ \"hailmary\": [\"line\", \"line\"] }", "Your own wording for a prayer, line by line. Keys: sign, creed, ourfather, hailmary, glorybe, fatima, hhq, closing."),
            i("prayers.extraStart", "[{ \"title\": \"…\", \"lines\": [\"…\"] }]", "Extra prayers after the opening Sign of the Cross."),
            i("prayers.extraEnd", "[{ \"title\": \"…\", \"lines\": [\"…\"] }]", "Extra prayers on the medal, before the final Sign of the Cross."),
            i("prayers.readings", "{ \"1\": \"text\" }", "A reading before a mystery, keyed 1 to 5. You supply the text, word for word."),
        ],
    },
    Group {
        title: "The voice",
        narrows: true,
        items: &[
            i("voice.on", "true | false", "false means no voice: words on screen only, and no credits spent."),
            i("voice.name", "a voice name, or several joined with +", "The name of a voice in your ElevenLabs account. See: pvc voices. Several names take turns: Bella + George."),
            i("voice.turnEvery", "prayer | decade", "With several voices: a different one on every prayer, or one voice for each decade. The opening and closing count as a decade each."),
            i("voice.takes", "1 to 20", "How many different recordings of each prayer take turns. 1 is one recording used every time, as in the first video. More costs more credits, once."),
            i("voice.freshEachTime", "true | false", "A new recording every time a prayer is said, so no two sound the same. Costs the most credits. Wins over takes."),
            i("voice.shuffle", "true | false", "Mix up the order of the takes and the voices, instead of going in turn. The mix is the same each time the video is made."),
            i("voice.id", "a voice id", "An exact ElevenLabs voice id. Wins over the name, and leads alone."),
            i("voice.model", "a model id", "The ElevenLabs model that speaks. See them, and what each costs, with: pvc models. eleven_multilingual_v2 is the first video."),
            i("voice.direction", "words", "How to say it, like: reverent, unhurried. Only Eleven v3 and v4 take direction. It is sent as a tag the voice does not read aloud, and left out for other models."),
            i("voice.speed", "0.4 to 2.5", "1 is the pace of the first video. 0.8 is slower, 1.2 is faster."),
            i("voice.exactSpeedup", "0.5 to 4", "Skip the measuring and speed the recording up by exactly this much."),
            i("voice.modelSpeed", "0.7 to 1.2", "How fast ElevenLabs itself speaks. The tool still lands on voice.speed, so a higher number here means less speeding up afterwards. Eleven v4 has no speed setting."),
            i("voice.stability", "0 to 1", "ElevenLabs voice setting. Lower lets one recording differ more from the next. Leave out to use the voice's own."),
            i("voice.similarity", "0 to 1", "ElevenLabs voice setting: how closely to hold to the voice. Leave out to use the voice's own."),
            i("voice.style", "0 to 1", "ElevenLabs voice setting: how much to play up the voice's style. Leave out to use the voice's own. Not every model uses it."),
            i("voice.speakerBoost", "true | false", "ElevenLabs voice setting: holds closer to the original speaker. Leave out to use the voice's own. Not every model uses it."),
            i("voice.cloneStrength", "0 to 1", "Only for a professional voice clone on Eleven v4: how closely to follow its recordings. ElevenLabs suggests 0."),
            i("voice.seed", "a whole number", "The same number asks ElevenLabs for the same recording again, as near as it can. Each take adds one, so takes still differ."),
            i("voice.language", "a language code, like en", "Tells the model which language the words are in. Eleven Multilingual v2 works it out by itself and ignores this."),
            i("voice.normalize", "auto | on | off", "Whether ElevenLabs spells out numbers and short forms before speaking them. auto lets it decide."),
            i("voice.quality", "standard | high", "How fine the sound from ElevenLabs is. high needs the Creator plan or above. standard is the first video."),
            i("voice.dictionary", "a dictionary name, or up to three joined with +", "Pronunciation dictionaries in your ElevenLabs account, for a word the voice keeps saying wrong. See: pvc dictionaries."),
            i("voice.recording", "a sound file", "Your own recording of the prayer, used in place of a generated voice. Set it with a rule for one kind of prayer."),
            i("voice.announceRecording", "a sound file", "Your own recording of the mystery being named."),
            i("voice.recordingSpeed", "0.5 to 4", "Speed change for your own recordings. 1 leaves them alone."),
            i("voice.respond", "a voice name", "A second voice that answers with the second half of the Our Father, Hail Mary and Glory Be."),
            i("voice.dropAfter", "a number", "0 = the voice says every Hail Mary. 3 = it says the first three of each decade, then leaves the rest to you."),
            i("voice.announce", "true | false", "Say the name of each mystery aloud."),
        ],
    },
    Group {
        title: "The pace",
        narrows: true,
        items: &[
            i("pace.before", "seconds", "Quiet before each prayer."),
            i("pace.after", "seconds", "Quiet after each prayer."),
            i("pace.announceGap", "seconds", "Between naming the mystery and starting the Our Father."),
            i("pace.think", "seconds", "Extra time to think after the mystery is named."),
            i("pace.secondsPerLine", "seconds", "With no voice: how long each line of a prayer stays up."),
            i("pace.minSeconds", "seconds", "With no voice: the shortest any bead stays up."),
            i("pace.vary", "seconds", "Lets the quiet after each prayer differ from bead to bead, by up to this much either way. 0 keeps every pause the same."),
            i("pace.transition", "fade | cut", "How one bead changes to the next."),
            i("pace.fade", "seconds", "Length of the soft fade."),
            i("pace.intro", "seconds", "How long the opening title stays up. Whole video only."),
            i("pace.outro", "seconds", "How long the ending stays up. Whole video only."),
        ],
    },
    Group {
        title: "The words on screen",
        narrows: true,
        items: &[
            i("words.size", "0.4 to 2.5", "Size of the words. 1 is the first video."),
            i("words.text", "always | first | never", "Show the prayer's words always, only the first time each prayer comes up, or never."),
            i("words.follow", "true | false", "The line being spoken lights up."),
            i("words.counter", "true | false", "The dots that fill as you go through a decade."),
            i("words.progress", "true | false", "\"Prayer 13 of 75\" and the bar under it."),
            i("words.mysteryName", "true | false", "The mystery's name beside the prayer."),
            i("words.hints", "true | false", "Lines like \"Many people offer this one for faith.\""),
            i("words.place", "true | false", "The line that says where your fingers are."),
            i("words.optionalLabel", "true | false", "The \"Optional\" label on optional prayers."),
        ],
    },
    Group {
        title: "The picture",
        narrows: true,
        items: &[
            i("picture.file", "a picture file", "The picture. Set it with a rule for one mystery or section. Leave out for none."),
            i("picture.focus", "like 50% 90%", "Which part sits in the middle: across, then down."),
            i("picture.zoom", "1 to 5", "1 shows as much as fits. Higher moves in closer."),
            i("picture.credit", "words", "Artist, title and date."),
            i("picture.showCredit", "true | false", "Show the credit line. On by default."),
            i("picture.place", "loop | beside | background", "Inside the loop of beads, above the words, or behind everything."),
            i("picture.show", "true | false", "false hides the picture here without forgetting which file it was."),
        ],
    },
    Group {
        title: "Colors and fonts",
        narrows: true,
        items: &[
            i("colors.mode", "dark | light", "Dark is the first video."),
            i("colors.background", "a color", "The background."),
            i("colors.panel", "a color", "Behind pictures while they load, and other quiet surfaces."),
            i("colors.text", "a color", "The words."),
            i("colors.quietText", "a color", "The smaller, quieter words."),
            i("colors.lines", "a color", "The chain and thin lines."),
            i("colors.done", "a color", "Beads you have already prayed."),
            i("colors.marker", "a color", "The bead you are on. Gold in the first video."),
            i("colors.titleFont", "a font name", "The font for titles. Cardo in the first video."),
            i("colors.textFont", "a font name", "The font for everything else. Fira Sans in the first video."),
        ],
    },
    Group {
        title: "The screen",
        narrows: false,
        items: &[
            i("screen.preset", "pixel-9-pro-fold | pixel-9-pro-fold-cover | phone-tall | phone-wide | tablet | tablet-tall | tv-1080p | tv-4k | square", "A named screen size."),
            i("screen.width", "pixels", "An exact width. Needs height too."),
            i("screen.height", "pixels", "An exact height. Needs width too."),
            i("screen.safe.top", "pixels", "Space kept clear at the top, for a camera cutout."),
            i("screen.safe.right", "pixels", "Space kept clear on the right."),
            i("screen.safe.bottom", "pixels", "Space kept clear at the bottom."),
            i("screen.safe.left", "pixels", "Space kept clear on the left."),
            i("screen.fps", "1 to 120", "Frames a second. 30 is the first video."),
        ],
    },
    Group {
        title: "Background sound",
        narrows: false,
        items: &[i("music.file", "a sound file", "Music or chant that you supply. Leave out for silence."), i("music.volume", "0 to 1", "How loud it is under the voice.")],
    },
    Group {
        title: "What comes out",
        narrows: false,
        items: &[
            i("output.video", "true | false", "A video file (.mp4)."),
            i("output.audio", "true | false", "A sound-only file (.m4a), for praying on a walk."),
            i("output.captions", "true | false", "A separate captions file (.srt)."),
            i("output.chapters", "true | false", "Chapter marks, so you can jump to a decade."),
            i("output.guide", "true | false", "The tap-through guide: a page you move a bead at a time."),
            i("output.folder", "a folder", "Where the files go. Leave out for Videos/Prayer Videos."),
            i("output.fileName", "words", "The file name, without the ending. Leave out to use the setup's name."),
            i("output.quality", "0 to 51", "Picture quality. Lower is better and bigger. 18 is the first video."),
        ],
    },
];

const RULES_HELP: &str = "\
Rules: one setting for part of the video
  The groups marked [narrows] can be set for part of the video with a rule.
  The narrowest rule wins: one bead beats one kind of prayer, which beats one
  mystery, which beats one section, which beats the whole video.

  A rule says where, then what:
    --rule \"mystery=3: voice.name=Bella\"
    --rule \"prayer=ourfather: voice.name=George\"
    --rule \"section=closing: words.size=1.2\"
    --rule \"bead=d3b10: voice.name=Sarah\"
    --rule \"prayer=hailmary: voice.name=Bella + George; voice.takes=3\"
    --rule \"prayer=creed: voice.model=eleven_v4; voice.direction=reverent, unhurried\"
    --rule \"mystery=1: picture.file=pictures/baptism.jpg; picture.credit=Piero della Francesca, The Baptism of Christ, c. 1450\"

  where:
    section  intro | opening | decade-1 … decade-5 | closing | outro
    mystery  1 to 5
    prayer   sign | creed | ourfather | hailmary | glorybe | fatima | hhq | closing | reading | extra
    bead     cross | p1 … p5 (the strand, p1 nearest the cross) | medal
             d1b1 … d5b10 (decade 3, bead 10 is d3b10) | L2 … L5 (the big beads in the loop)
             g0 … g5 (the stretches of chain where the Glory Be is said)
";

/// The reference as plain text, for the terminal.
pub fn text() -> String {
    let mut out = String::from("Everything you can change. Each has a default, so set only what you want different.\n\n");
    for g in GROUPS {
        out.push_str(&format!("{}{}\n", g.title, if g.narrows { "  [narrows]" } else { "" }));
        for item in g.items {
            out.push_str(&format!("  {:<26} {}\n  {:<26}   choices: {}\n", item.path, item.what, "", item.takes));
        }
        out.push('\n');
    }
    out.push_str(RULES_HELP);
    out
}

/// The reference as a Markdown page, for `docs/settings.md`.
pub fn markdown() -> String {
    let mut out = String::from(
        "# Settings\n\nEvery setting, with its choices. Each one has a default, so a setup only lists what it changes.\n\n\
         This page is made by the tool itself (`pvc options --markdown`), so it always matches what the tool accepts.\n\n",
    );
    for g in GROUPS {
        out.push_str(&format!("## {}\n\n", g.title));
        if g.narrows {
            out.push_str("Can be set for part of the video with a rule.\n\n");
        }
        out.push_str("| Setting | Choices | What it does |\n| --- | --- | --- |\n");
        for item in g.items {
            out.push_str(&format!("| `{}` | {} | {} |\n", item.path, item.takes.replace('|', "\\|"), item.what.replace('|', "\\|")));
        }
        out.push('\n');
    }
    out.push_str("## Rules\n\n```\n");
    out.push_str(RULES_HELP);
    out.push_str("```\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::Setup;
    use serde_json::Value;
    use std::collections::BTreeSet;

    /// Settings whose value is a list or a map a person fills in. Described as one item.
    const WHOLE: [&str; 5] = ["prayers.custom.items", "prayers.wording", "prayers.extraStart", "prayers.extraEnd", "prayers.readings"];

    fn leaves(v: &Value, prefix: &str, out: &mut BTreeSet<String>) {
        match v {
            Value::Object(o) if !WHOLE.contains(&prefix) => {
                for (k, v) in o {
                    leaves(v, &if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") }, out);
                }
            }
            _ => {
                out.insert(prefix.to_string());
            }
        }
    }

    #[test]
    fn every_setting_is_described_and_nothing_is_stale() {
        let mut real = BTreeSet::new();
        leaves(&serde_json::to_value(Setup::default()).unwrap(), "", &mut real);
        real.remove("rules");
        let described: BTreeSet<String> = GROUPS.iter().flat_map(|g| g.items.iter().map(|i| i.path.to_string())).collect();
        let missing: Vec<&String> = real.difference(&described).collect();
        let stale: Vec<&String> = described.difference(&real).collect();
        assert!(missing.is_empty(), "settings with no description: {missing:?}");
        assert!(stale.is_empty(), "descriptions with no setting: {stale:?}");
    }

    #[test]
    fn the_groups_that_narrow_are_the_ones_rules_accept() {
        let narrowing: Vec<&str> = GROUPS.iter().filter(|g| g.narrows).map(|g| g.items[0].path.split('.').next().unwrap()).collect();
        assert_eq!(narrowing, ["voice", "pace", "words", "picture", "colors"]);
        assert_eq!(narrowing, crate::setup::STYLE_GROUPS);
    }

    #[test]
    fn listed_choices_are_really_accepted() {
        for g in GROUPS {
            for item in g.items {
                if !item.takes.contains(" | ") || item.takes.contains("true") {
                    continue;
                }
                for choice in item.takes.split(" | ") {
                    let mut doc = serde_json::json!({});
                    crate::setup::set_path(&mut doc, item.path, Value::String(choice.to_string())).unwrap();
                    if choice == "custom" {
                        crate::setup::set_path(&mut doc, "prayers.custom.items", serde_json::json!(["a", "b", "c", "d", "e"])).unwrap();
                    }
                    crate::setup::from_doc(&doc).unwrap_or_else(|e| panic!("{} = {choice}: {e}", item.path));
                }
            }
        }
    }
}
