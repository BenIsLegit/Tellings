# Prayer Video Creator

Read [README.md](README.md) first. It says what this is and why I want it.

This file says what matters to me and why. It does not say how to build anything. That part is yours to work out, and to ask me about. How the tool is put together, and how to make a video with it, is in [AGENTS.md](AGENTS.md).

## The one rule

Someone holding prayer beads should always be able to see two things: which bead they are on, and what to say on it.

If a choice makes the video prettier but makes the bead harder to follow, pick the bead.

## It has to be easy

Deep customization is the point of this tool. It only counts if it is easy to use.

- **It works with no choices made.** Every setting has a sensible default.
- **Changing one thing means touching one thing.** I should not have to restate everything else.
- **Settings narrow.** A choice can cover the whole video, one section, one mystery, one kind of prayer, or one single bead. The narrowest choice wins.
- **No code.** I don't write code by hand. I should be able to change anything without editing code.
- **Setups can be saved.** Once a video is how I like it, I want to reuse that setup and tweak it.

Voice is the example I care about most. I want to set a voice per bead, per section and per mystery, and it should take seconds.

The full list of what should be changeable is in [docs/what-you-can-change.md](docs/what-you-can-change.md).

## What has to be right

Other guides failed me because they didn't match my beads. Accuracy is the reason this exists.

### The beads

A drawing must match the real thing: the count, the grouping, and where the cross and medal sit.

A standard five-decade rosary has 59 beads:

- Crucifix
- Five beads above it: one big, three small, one big
- The medal
- Five sets of ten small beads, with one big bead between sets

A St. Benedict rosary is this same layout. Only the medal and the cross are St. Benedict's.

### The order

| Where | Prayer |
| --- | --- |
| Crucifix | Sign of the Cross, then the Apostles' Creed |
| First big bead | Our Father |
| Three small beads | Hail Mary on each |
| Chain (no bead) | Glory Be |
| Second big bead | Name the first mystery, then Our Father |
| Ten small beads | Hail Mary on each |
| Chain (no bead) | Glory Be, then O My Jesus (optional) |
| Next big bead | Name the next mystery, then Our Father |
| ...repeat for all five sets of ten | |
| Medal | Hail, Holy Queen, then the Closing Prayer (optional) |
| Crucifix | Sign of the Cross |

The mysteries by day: Joyful on Monday and Saturday. Sorrowful on Tuesday and Friday. Glorious on Wednesday and Sunday. Luminous on Thursday.

### The words

Use the traditional wording of each prayer exactly. The wording used in the first video is in `crates/engine/src/prayers.rs`, copied line for line from `prototype/core.js`. Don't reword or modernize a prayer unless I pick a different wording. Label optional prayers as optional.

If you quote Scripture, use the NLT, word for word.

### Other beads and other traditions

[docs/prayer-beads.md](docs/prayer-beads.md) lists the kinds of beads I want covered. Only the five-decade rosary has been checked so far.

- Before building any other kind, check its layout and its prayers against a source that tradition itself trusts. Don't rely on memory, yours or mine.
- These are real prayers from living traditions. Use each tradition's own names and wording, and don't blend one into another.

## Voices

- Voices come from ElevenLabs. The tool should talk to ElevenLabs itself, so nobody has to download and pass audio files around by hand.
- The voice in the first video is called "Imperial - Male" in my ElevenLabs account.
- That recording was told to speak slowly, and it came out far too slow. I had it sped up to 1.75x. Aim for that brisk pace by default, and let me change it.
- Generating speech spends my credits. Record each prayer once per voice and reuse it. The first video uses one Hail Mary recording 53 times.
- Tell me before anything that would spend a lot of credits.

## Be careful with

- **Keys.** My ElevenLabs API key must never be saved in this repo, or shown in a video, a log or a commit.
- **Pictures.** Use pictures I supply, or ones that are in the public domain. Show the artist and title on screen.

## About me

- I edit video and make graphics. I don't write code by hand. Explain what you're doing in plain words and tell me exactly how to run anything.
- I read slowly. Keep what you write to me short. In the videos, keep text large and leave it up long enough to read.
- My phone is a Pixel 9 Pro Fold. The first video is sized for its inner screen, 2076 x 2152.

## When I ask for a video

Read [AGENTS.md](AGENTS.md). It has the steps. In short: use the `pvc` tool, show me the credit cost before spending anything, then tell me where the file is.

## When I ask you to change the tool

[AGENTS.md](AGENTS.md) has the commands, how the code fits together, and how to check a change. The line below loads it along with this file.

@AGENTS.md

## Versioning

**Single source of truth: `package.json` `version` field.**

When bumping the version, update `package.json`, `src-tauri/tauri.conf.json`, and the root `Cargo.toml` to match.
`build.bat` produces a test build with the commit hash (e.g., `0.1.0-a1b2c3d`).
`build.bat --release` produces a clean release build (e.g., `0.1.0`).

## Ask me first

I haven't decided these. Ask instead of guessing:

- What the tool is like to use: an app with buttons, a web page, or something else. *(Decided: a desktop app, plus the `pvc` tool so Claude Code or Codex can make a video when asked.)*
- Which kind of beads to add after the five-decade rosary
- Anything that would change how the first video looks or sounds
- Anything that would spend a lot of ElevenLabs credits
