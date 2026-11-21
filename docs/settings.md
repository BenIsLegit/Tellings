# Settings

Every setting, with its choices. Each one has a default, so a setup only lists what it changes.

This page is made by the tool itself (`pvc options --markdown`), so it always matches what the tool accepts.

## The setup itself

| Setting | Choices | What it does |
| --- | --- | --- |
| `name` | words | The name of the setup. Also the name of the video file. |
| `description` | words | A line about it, shown in the list. |

## The beads

| Setting | Choices | What it does |
| --- | --- | --- |
| `beads.kind` | five-decade-rosary | Which beads. Only the five-decade rosary is built so far. |
| `beads.medal` | plain \| st-benedict \| miraculous | The medal where the strand joins the loop. |
| `beads.cross` | latin \| st-benedict \| orthodox \| greek \| knotted \| tassel \| celtic \| san-damiano \| medal | What hangs at the end. |
| `beads.style` | outline \| wood \| glass \| stone \| pearl \| metal \| knots | How the beads look. outline is the first video. |
| `beads.color` | a color, like #9A6B43 | Bead color. Leave out for the usual color of the style. |
| `beads.size` | 0.4 to 2 | Bead size. 1 is the first video. |
| `beads.cord` | chain \| cord | A fine chain or a thicker cord. |
| `beads.cordColor` | a color | Color of the chain or cord. |
| `beads.direction` | right-first \| left-first | Which way around the loop you go. |

## The prayers

| Setting | Choices | What it does |
| --- | --- | --- |
| `prayers.title` | words | The big title on the opening card. |
| `prayers.mysteries` | today \| joyful \| sorrowful \| glorious \| luminous \| custom | Which mysteries. today picks by the day of the week. |
| `prayers.custom.name` | words | For your own mysteries: shown as "<name> Mysteries". |
| `prayers.custom.items` | a list of five names | For your own mysteries: one name for each decade. |
| `prayers.opening` | true \| false | Start at the crucifix (true), or go straight to the first decade (false). |
| `prayers.closing` | true \| false | End with the Hail, Holy Queen and the Sign of the Cross. |
| `prayers.firstDecade` | 1 to 5 | The decade to start on. |
| `prayers.lastDecade` | 1 to 5 | The decade to end on. |
| `prayers.fatima` | true \| false | O My Jesus after each Glory Be. Optional in the rosary. |
| `prayers.closingPrayer` | true \| false | The Closing Prayer after the Hail, Holy Queen. Optional. |
| `prayers.gloryBe` | chain \| next-bead | Where the Glory Be goes: on the chain, or on the next big bead. |
| `prayers.wording` | { "hailmary": ["line", "line"] } | Your own wording for a prayer, line by line. Keys: sign, creed, ourfather, hailmary, glorybe, fatima, hhq, closing. |
| `prayers.extraStart` | [{ "title": "…", "lines": ["…"] }] | Extra prayers after the opening Sign of the Cross. |
| `prayers.extraEnd` | [{ "title": "…", "lines": ["…"] }] | Extra prayers on the medal, before the final Sign of the Cross. |
| `prayers.readings` | { "1": "text" } | A reading before a mystery, keyed 1 to 5. You supply the text, word for word. |

## The voice

Can be set for part of the video with a rule.

| Setting | Choices | What it does |
| --- | --- | --- |
| `voice.on` | true \| false | false means no voice: words on screen only, and no credits spent. |
| `voice.name` | a voice name, or several joined with + | The name of a voice in your ElevenLabs account. See: pvc voices. Several names take turns: Bella + George. |
| `voice.turnEvery` | prayer \| decade | With several voices: a different one on every prayer, or one voice for each decade. The opening and closing count as a decade each. |
| `voice.takes` | 1 to 20 | How many different recordings of each prayer take turns. 1 is one recording used every time, as in the first video. More costs more credits, once. |
| `voice.freshEachTime` | true \| false | A new recording every time a prayer is said, so no two sound the same. Costs the most credits. Wins over takes. |
| `voice.shuffle` | true \| false | Mix up the order of the takes and the voices, instead of going in turn. The mix is the same each time the video is made. |
| `voice.id` | a voice id | An exact ElevenLabs voice id. Wins over the name, and leads alone. |
| `voice.model` | a model id | The ElevenLabs model that speaks. See them, and what each costs, with: pvc models. eleven_v4 is the usual one. The first video was recorded with eleven_multilingual_v2. |
| `voice.direction` | words | How to say it, like: reverent, unhurried. Only Eleven v3 and v4 take direction. It is sent as a tag the voice does not read aloud, and left out for other models. |
| `voice.speed` | 0.4 to 2.5 | 1 is the pace of the first video. 0.8 is slower, 1.2 is faster. |
| `voice.exactSpeedup` | 0.5 to 4 | Skip the measuring and speed the recording up by exactly this much. |
| `voice.modelSpeed` | 0.7 to 1.2 | How fast ElevenLabs itself speaks. The tool still lands on voice.speed, so a higher number here means less speeding up afterwards. Eleven v4 has no speed setting. |
| `voice.stability` | 0 to 1 | ElevenLabs voice setting. Lower lets one recording differ more from the next. Leave out to use the voice's own. |
| `voice.similarity` | 0 to 1 | ElevenLabs voice setting: how closely to hold to the voice. Leave out to use the voice's own. |
| `voice.style` | 0 to 1 | ElevenLabs voice setting: how much to play up the voice's style. Leave out to use the voice's own. Not every model uses it. |
| `voice.speakerBoost` | true \| false | ElevenLabs voice setting: holds closer to the original speaker. Leave out to use the voice's own. Not every model uses it. |
| `voice.cloneStrength` | 0 to 1 | Only for a professional voice clone on Eleven v4: how closely to follow its recordings. ElevenLabs suggests 0. |
| `voice.seed` | a whole number | The same number asks ElevenLabs for the same recording again, as near as it can. Each take adds one, so takes still differ. |
| `voice.language` | a language code, like en | Tells the model which language the words are in. Eleven Multilingual v2 works it out by itself and ignores this. |
| `voice.normalize` | auto \| on \| off | Whether ElevenLabs spells out numbers and short forms before speaking them. auto lets it decide. |
| `voice.quality` | standard \| high | How fine the sound from ElevenLabs is. high needs the Creator plan or above. standard is the first video. |
| `voice.dictionary` | a dictionary name, or up to three joined with + | Pronunciation dictionaries in your ElevenLabs account, for a word the voice keeps saying wrong. See: pvc dictionaries. |
| `voice.recording` | a sound file | Your own recording of the prayer, used in place of a generated voice. Set it with a rule for one kind of prayer. |
| `voice.announceRecording` | a sound file | Your own recording of the mystery being named. |
| `voice.recordingSpeed` | 0.5 to 4 | Speed change for your own recordings. 1 leaves them alone. |
| `voice.respond` | a voice name | A second voice that answers with the second half of the Our Father, Hail Mary and Glory Be. |
| `voice.together` | true \| false | With a second voice that answers: record both halves in one go, as one conversation, so the answer follows on. Only Eleven v4 and v3 can. It is kept for that pair of voices and reused, like any recording. |
| `voice.dropAfter` | a number | 0 = the voice says every Hail Mary. 3 = it says the first three of each decade, then leaves the rest to you. |
| `voice.announce` | true \| false | Say the name of each mystery aloud. |

## The pace

Can be set for part of the video with a rule.

| Setting | Choices | What it does |
| --- | --- | --- |
| `pace.before` | seconds | Quiet before each prayer. |
| `pace.after` | seconds | Quiet after each prayer. |
| `pace.announceGap` | seconds | Between naming the mystery and starting the Our Father. |
| `pace.think` | seconds | Extra time to think after the mystery is named. |
| `pace.secondsPerLine` | seconds | With no voice: how long each line of a prayer stays up. |
| `pace.minSeconds` | seconds | With no voice: the shortest any bead stays up. |
| `pace.vary` | seconds | Lets the quiet after each prayer differ from bead to bead, by up to this much either way. 0 keeps every pause the same. |
| `pace.transition` | fade \| cut | How one bead changes to the next. |
| `pace.fade` | seconds | Length of the soft fade. |
| `pace.intro` | seconds | How long the opening title stays up. Whole video only. |
| `pace.outro` | seconds | How long the ending stays up. Whole video only. |

## The words on screen

Can be set for part of the video with a rule.

| Setting | Choices | What it does |
| --- | --- | --- |
| `words.size` | 0.4 to 2.5 | Size of the words. 1 is the first video. |
| `words.text` | always \| first \| never | Show the prayer's words always, only the first time each prayer comes up, or never. |
| `words.follow` | true \| false | The line being spoken lights up. |
| `words.counter` | true \| false | The dots that fill as you go through a decade. |
| `words.progress` | true \| false | "Prayer 13 of 75" and the bar under it. |
| `words.mysteryName` | true \| false | The mystery's name beside the prayer. |
| `words.hints` | true \| false | Lines like "Many people offer this one for faith." |
| `words.place` | true \| false | The line that says where your fingers are. |
| `words.optionalLabel` | true \| false | The "Optional" label on optional prayers. |

## The picture

Can be set for part of the video with a rule.

| Setting | Choices | What it does |
| --- | --- | --- |
| `picture.file` | a picture file | The picture. Set it with a rule for one mystery or section. Leave out for none. |
| `picture.focus` | like 50% 90% | Which part sits in the middle: across, then down. |
| `picture.zoom` | 1 to 5 | 1 shows as much as fits. Higher moves in closer. |
| `picture.credit` | words | Artist, title and date. |
| `picture.showCredit` | true \| false | Show the credit line. On by default. |
| `picture.place` | loop \| beside \| background | Inside the loop of beads, above the words, or behind everything. |
| `picture.show` | true \| false | false hides the picture here without forgetting which file it was. |

## Colors and fonts

Can be set for part of the video with a rule.

| Setting | Choices | What it does |
| --- | --- | --- |
| `colors.mode` | dark \| light | Dark is the first video. |
| `colors.background` | a color | The background. |
| `colors.panel` | a color | Behind pictures while they load, and other quiet surfaces. |
| `colors.text` | a color | The words. |
| `colors.quietText` | a color | The smaller, quieter words. |
| `colors.lines` | a color | The chain and thin lines. |
| `colors.done` | a color | Beads you have already prayed. |
| `colors.marker` | a color | The bead you are on. Gold in the first video. |
| `colors.titleFont` | a font name | The font for titles. Cardo in the first video. |
| `colors.textFont` | a font name | The font for everything else. Fira Sans in the first video. |

## The screen

| Setting | Choices | What it does |
| --- | --- | --- |
| `screen.preset` | pixel-9-pro-fold \| pixel-9-pro-fold-cover \| phone-tall \| phone-wide \| tablet \| tablet-tall \| tv-1080p \| tv-4k \| square | A named screen size. |
| `screen.width` | pixels | An exact width. Needs height too. |
| `screen.height` | pixels | An exact height. Needs width too. |
| `screen.safe.top` | pixels | Space kept clear at the top, for a camera cutout. |
| `screen.safe.right` | pixels | Space kept clear on the right. |
| `screen.safe.bottom` | pixels | Space kept clear at the bottom. |
| `screen.safe.left` | pixels | Space kept clear on the left. |
| `screen.fps` | 1 to 120 | Frames a second. 30 is the first video. |

## Background sound

| Setting | Choices | What it does |
| --- | --- | --- |
| `music.file` | a sound file | Music or chant that you supply. Leave out for silence. A file here wins over music.prompt. |
| `music.volume` | 0 to 1 | How loud it is under the voice. |
| `music.prompt` | words | Music for ElevenLabs to make, like: slow, quiet organ in an empty chapel. Describe the sound. Do not name a hymn, a song or an artist: ElevenLabs refuses those. Making it spends credits, about 900 a minute, once. The piece is kept. |
| `music.minutes` | 0.5 to 5 | How long a piece ElevenLabs makes. It loops under the whole video. Longer costs more: about 900 credits a minute, a rough figure. |
| `music.instrumental` | true \| false | true means no singing. Only for music ElevenLabs makes. |
| `music.take` | 1 to 99 | Which piece. A new number asks ElevenLabs for a different piece from the same words, and costs again. |

## What comes out

| Setting | Choices | What it does |
| --- | --- | --- |
| `output.video` | true \| false | A video file (.mp4). |
| `output.audio` | true \| false | A sound-only file (.m4a), for praying on a walk. |
| `output.captions` | true \| false | A separate captions file (.srt). |
| `output.chapters` | true \| false | Chapter marks, so you can jump to a decade. |
| `output.guide` | true \| false | The tap-through guide: a page you move a bead at a time. |
| `output.folder` | a folder | Where the files go. Leave out for Videos/Prayer Videos. |
| `output.fileName` | words | The file name, without the ending. Leave out to use the setup's name. |
| `output.quality` | 0 to 51 | Picture quality. Lower is better and bigger. 18 is the first video. |

## Rules

```
Rules: one setting for part of the video
  The groups marked [narrows] can be set for part of the video with a rule.
  The narrowest rule wins: one bead beats one kind of prayer, which beats one
  mystery, which beats one section, which beats the whole video.

  A rule says where, then what:
    --rule "mystery=3: voice.name=Bella"
    --rule "prayer=ourfather: voice.name=George"
    --rule "section=closing: words.size=1.2"
    --rule "bead=d3b10: voice.name=Sarah"
    --rule "prayer=hailmary: voice.name=Bella + George; voice.takes=3"
    --rule "prayer=hailmary: voice.respond=George; voice.together=true"
    --rule "prayer=creed: voice.model=eleven_v3; voice.direction=reverent, unhurried"
    --rule "mystery=1: picture.file=pictures/baptism.jpg; picture.credit=Piero della Francesca, The Baptism of Christ, c. 1450"

  where:
    section  intro | opening | decade-1 … decade-5 | closing | outro
    mystery  1 to 5
    prayer   sign | creed | ourfather | hailmary | glorybe | fatima | hhq | closing | reading | extra
    bead     cross | p1 … p5 (the strand, p1 nearest the cross) | medal
             d1b1 … d5b10 (decade 3, bead 10 is d3b10) | L2 … L5 (the big beads in the loop)
             g0 … g5 (the stretches of chain where the Glory Be is said)
```
