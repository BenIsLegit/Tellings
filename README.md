# Prayer Video Creator

A tool that makes follow-along prayer videos for the beads you are holding.

You tell it what you have: a five-decade rosary, a one-decade rosary, an Orthodox prayer rope, or something else. You choose how it looks and sounds. It makes a video that shows your beads, lights up the bead you are on, shows the words, and says the prayer aloud.

## Why this exists

I was learning to pray the rosary and found it hard to follow.

- The Hallow app has guided rosaries, but I couldn't find a visual guide in it.
- Guided rosary videos confused me more. Some show a different number of beads above the crucifix. Some show none. I couldn't match them to the beads in my hand.

Claude made me a tap-through guide that draws the whole rosary and lights up one bead at a time. That made it click. Then we turned it into a video, with a voice and a painting for each mystery. That worked too.

This project turns that one video into a tool. I want to make the other sets of mysteries, change anything about them, and make versions for other kinds of prayer beads.

A video should match what a person owns. Nobody should have to translate from someone else's beads to their own.

## What a video shows

- **Your beads**, drawn the way they are really laid out.
- **A gold bead** that marks where your fingers should be.
- **The words** of the prayer for that bead.
- **A picture** for each mystery or section, so you know what to picture.
- **A voice** saying each prayer, so you can pray along.

## What you can change

Nearly everything: the beads, the cross, the prayers, the voice, the pictures, the screen size and the pace.

A choice can cover the whole video, or one section, one mystery, one kind of prayer, or one single bead. Each mystery can have its own voice, for example.

Every choice has a default, so a video works before you change anything.

- [What you can change](docs/what-you-can-change.md) is the full list.
- [Prayer beads](docs/prayer-beads.md) lists the kinds of beads the tool should cover.

## How to make a video

There are three ways. They all use the same engine, so the video comes out the same.

### 1. Ask Claude Code or Codex

Open this folder in Claude Code or Codex and say what you want:

> Make the Joyful Mysteries video with the voice "Imperial - Male".

> Make today's rosary for my TV, with no voice.

> Same as my last one, but a different voice for every Our Father.

It will tell you what it costs in ElevenLabs credits and wait for your yes before it spends anything. Then it tells you where the file is.

### 2. Use the app

Double-click `start.bat`. The first time takes a few minutes.

1. **Setups.** Pick a preset and press **Make**. Or press **Change it** first.
2. **The editor.** The preview on the right is the real frame. Click a bead to jump to it. Change anything on the left.
3. **This choice covers.** Voice, Words, Colors and Pace have a bar at the top. Pick the whole video, this section, this mystery, every prayer of this kind, or this bead only. Then change the setting. The narrowest choice wins.
4. **Make the video.** It shows the credit cost first. Nothing is spent until you press the button that names the number.

Add your ElevenLabs key once, under **Settings**. It is kept in Windows' own password store, never in this folder.

### 3. Type the commands yourself

```
pvc.bat list                              the setups
pvc.bat show luminous                     what it will be, and what it costs
pvc.bat make luminous                     make it
pvc.bat make joyful --voice "Some Voice"  change one thing
pvc.bat help                              everything else
```

Videos go to your **Videos** folder, in **Prayer Videos**.

## Pictures

Put your pictures in the pictures folder. The app has a button that opens it: **Pictures**, then **Open my pictures folder**. The Luminous setup looks there for `luminous-1` to `luminous-5`. See [`pictures/README.md`](pictures/README.md).

If a picture is missing, the video is still made. The loop of beads holds the name of the mystery instead.

## First time on a computer

The tool needs four free programs: [Node.js](https://nodejs.org), [Rust](https://rustup.rs), [FFmpeg](https://ffmpeg.org) and Chrome or Edge. Then:

- `start.bat` opens the app from this folder.
- `build.bat` runs every check and builds the app and the `pvc` tool into `target\release\`.
- `build.bat --release` also makes an installer.

On macOS and Linux use `start.sh`, `build.sh` and `./pvc`.

## Where it stands

- **Made so far:** one video, the Luminous Mysteries of the rosary. It runs 13 minutes 10 seconds.
- **Built:** the tool, for the five-decade rosary. [What works today](docs/what-works-today.md) lists what is built and what is not.
- **Not checked yet:** a real ElevenLabs voice. The voice was tested against a stand-in, at no cost. The first real video is the test. Listen to it before you trust it.
- **Not built yet:** other kinds of beads. Each one has to be checked against a source its own tradition trusts first.

## What is in this folder

| Folder | What it is |
| --- | --- |
| `crates/engine/` | The engine: prayers, beads, setups, voice, sound and video. |
| `crates/cli/` | `pvc`, the command-line tool. |
| `src/` and `src-tauri/` | The app. |
| `renderer/` | The script that draws one frame. The app's preview and the video both use it. |
| `presets/` | The setups that come with the tool. |
| `docs/` | [What you can change](docs/what-you-can-change.md), [what works today](docs/what-works-today.md), [every setting](docs/settings.md), [prayer beads](docs/prayer-beads.md). |
| `prototype/` | The rough scripts that made the first video. A reference. |

[AGENTS.md](AGENTS.md) is the guide for Claude Code and Codex.
