# For Claude Code, Codex and other agents

Read [CLAUDE.md](CLAUDE.md) first. It says what matters to the person who owns this project. This file says how to do the two things you will be asked for: **make a video**, and **change the tool**.

The person does not write code and reads slowly. Keep what you say to them short and plain.

## When asked to make a video

Use the `pvc` tool in this folder. It does everything the app does, through the same engine. Never edit code to make a video.

On Windows run `.\pvc.bat`. On macOS or Linux run `./pvc`. The first run builds the tool, which takes a few minutes.

### The steps

1. **Work out the setup.** Start from a preset and add changes.

   ```
   pvc list                 the presets and the person's saved setups
   pvc options              every setting and its choices
   ```

2. **Look before you make.** This is free and fast.

   ```
   pvc show luminous --voice "Imperial - Male"
   ```

   It prints the length, the voices, the pictures, the files it would write, and a `Credits:` line.

3. **If `Credits:` is not "none", stop and ask.** Tell the person the number in one sentence, like "This will spend about 2,573 ElevenLabs credits. OK?" Wait for a yes in this conversation. An earlier yes for a different video does not count.

4. **Make it.**

   ```
   pvc make luminous --voice "Imperial - Male" --approve-credits 2573
   ```

   Leave `--approve-credits` out when no credits are needed. The tool refuses to spend more than the number you pass. It exits with code 2 and spends nothing if approval is missing.

5. **Tell the person, in a few short lines:** where the file is, how long it is, what was spent, and any `Note:` lines the tool printed.

### Changes

```
--mysteries joyful                       joyful | sorrowful | glorious | luminous | today
--voice "Name"                           a voice in the person's ElevenLabs account
--voice "Bella + George"                 several voices take turns
--takes 5                                five recordings of each prayer take turns
--no-voice                               words on screen only, no credits
--screen tv-1080p                        see pvc options for the sizes
--to "C:\some\folder"                    where the files go
--set words.follow=true                  any setting from pvc options
--rule "mystery=3: voice.name=Bella"     a setting for only part of the video
```

Rules are how the person's main wish works: a different voice (or picture, or pace, or colors) for one section, one mystery, one kind of prayer, or one bead. The narrowest rule wins.

```
--rule "prayer=ourfather: voice.name=George"          every Our Father
--rule "mystery=3: voice.name=Bella"                  the whole third mystery
--rule "bead=cross, section=closing: voice.name=Sarah" the very last prayer
--rule "mystery=1: picture.file=C:\pics\baptism.jpg; picture.credit=Piero della Francesca, The Baptism of Christ, c. 1450"
```

### Variety

One recording of the Hail Mary played 53 times sounds stamped out. These settings fix that. They are voice settings, so each works for the whole video or in a rule.

```
--takes 5                                  five recordings of each prayer take turns
--set voice.freshEachTime=true             a new recording every time a prayer is said
--set voice.shuffle=true                   mixed-up order instead of in turn
--voice "Bella + George"                   the voices take turns, a new one each prayer
--set voice.turnEvery=decade               ... or one voice for each decade
--set pace.vary=0.2                        pauses differ by up to 0.2 seconds
--rule "prayer=hailmary: voice.takes=5; voice.shuffle=true"     only the Hail Marys
--rule "prayer=hailmary: voice.name=Bella + George + Sarah"     three voices share the Hail Marys
```

**Variety costs credits.** On a whole rosary, one take is about 2,600 credits, five takes about 5,800, and a new recording every time about 16,300. Each recording is made once and kept. Run `pvc show` again after adding variety, and ask about the new number.

If one recording came out wrong, `pvc rerecord luminous --scene d1b3` throws away what is heard on that bead. Pass the same setup and changes you make the video with. The next `pvc make` records it afresh and asks for credits as usual.

Bead names: `cross`, `p1` to `p5` (the strand, `p1` nearest the cross), `medal`, `d3b10` (decade 3, bead 10), `L2` to `L5` (the big beads in the loop), `g0` to `g5` (the chain, where the Glory Be is said).

### Useful extras

```
pvc frame luminous --scene d3b10 --to still.png   draw one moment as a picture, then look at it
pvc rerecord luminous --scene d3b10               throw away that bead's recording, to make it afresh
pvc save "My Joyful" joyful --voice "Bella"       keep a setup under a name
pvc make my-joyful                                use it again
pvc voices                                        the voices in the account (free)
pvc credits                                       credits left (free)
pvc doctor                                        is FFmpeg, a browser and the key in place?
```

Add `--json` to any of them for output a program can read.

To check how a change looks before making a whole video, use `pvc frame` and open the picture. It takes a second.

### The ElevenLabs key

The key lives in the computer's password store. You never see it and never need to. If `pvc show` says no key is saved, ask the person to add it in the app under Settings, or to run `pvc key set` themselves in a terminal. Do not ask them to paste the key into this conversation, and never write it to a file.

### Pictures

Use pictures the person supplies, or ones in the public domain. Every picture needs a credit: artist, title, date. A missing picture is not an error. The video is still made, and the loop of beads holds the mystery's name instead. Say so in your summary.

## When asked to change the tool

### Where things are

| Folder | What it is |
| --- | --- |
| `crates/engine/` | The brain. Prayers, beads, setups, the plan, voice, sound, stills, video. Plain Rust. |
| `crates/cli/` | `pvc`, a thin shell over the engine. |
| `src-tauri/` | The app's Rust side, another thin shell over the engine. |
| `src/` | The app's window. SvelteKit, Svelte 5 runes, Tailwind 4. |
| `renderer/` | `frame.js` and `frame.css` draw one frame. Used by the app's preview, the video stills and the tap-through guide. |
| `presets/` | The built-in setups. Small JSON files. |
| `prototype/` | The rough scripts that made the first video. Reference only. |

The module list at the top of `crates/engine/src/lib.rs` shows the path a video takes.

### Rules that come from CLAUDE.md

- **The wording of prayers** is in `crates/engine/src/prayers.rs`, copied from the first video. Do not reword it.
- **The bead layout** is in `crates/engine/src/beads.rs`. Tests pin 59 beads and their places.
- **The order of prayers** is in `crates/engine/src/plan.rs`. A test checks it against the table in CLAUDE.md.
- **A new kind of beads** needs its layout and prayers checked against a source that tradition trusts, and the person asked which kind comes next. Do not add one from memory.
- **Scripture** is NLT, word for word, supplied by the person. Do not type it from memory.
- **Every setting has a default** and is described in `crates/engine/src/reference.rs`. A test fails if you add a setting without describing it. After changing settings, run `pvc options --markdown > docs/settings.md`.
- **Anything that changes how the first video looks or sounds**: ask first.

### Checking your work

```
cargo test -p pvc-engine      the engine's tests
npm run check                 the window's type check
npm test                      the window's tests
build.bat                     everything above, then builds the app and pvc
start.bat                     opens the app from this folder
```

To see a change to the drawing, use `pvc frame` and look at the picture.

To look at the app's window while the person has their own copy open, start a second copy with `PVC_SECOND_WINDOW=1`, its own `PVC_DATA_DIR`, and its own `WEBVIEW2_USER_DATA_FOLDER`. Without the first, the app only shows the window that is already open. Close your copy when you are done. If they started the app with `start.bat`, it rebuilds and restarts by itself each time you change the code.

To test the voice without spending credits, run the stand-in service, `node dev/stand-in-elevenlabs.mjs` (Windows only). The steps are at the top of that file. Always set `PVC_DATA_DIR` to a scratch folder while you do, so stand-in speech is never kept as if it were the person's real voice. Pass `--to` a scratch folder too, or the test video lands in their Videos folder.

### Versioning

The version lives in `package.json`. When it changes, change `src-tauri/tauri.conf.json` and the root `Cargo.toml` to match. `build.bat` makes a test build named like `0.1.0-a1b2c3d`. `build.bat --release` makes the real `0.1.0` with an installer.
