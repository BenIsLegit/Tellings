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
--model eleven_flash_v2_5                the ElevenLabs model that speaks (see pvc models)
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

### The model, and how the voice speaks

`pvc models` lists the ElevenLabs models and what each charges. It is free. The usual model is `eleven_v4`. The first video was recorded with `eleven_multilingual_v2`. Naming that model brings back its sound, and the recordings kept from then.

```
--model eleven_multilingual_v2                          a different model for the whole video
--rule "prayer=hailmary: voice.model=eleven_flash_v2_5"  ... or for part of it
--set "voice.direction=reverent, unhurried"             how to say it. Eleven v3 and v4 only
--set voice.stability=0.4                               ElevenLabs' own controls. See pvc options
pvc voices "Imperial - Male"                            how that voice is set up at ElevenLabs
pvc dictionaries                                        pronunciation dictionaries, for voice.dictionary
```

These are voice settings, so each works for the whole video or in a rule.

**A different model is a different recording.** Change the model, or any setting that changes the sound, and those prayers are recorded again and paid for again. Changing it back is free: the earlier recordings are kept. Run `pvc show` again and ask about the new number.

The cost is letters times the model's price. Most models charge one credit a letter. The Flash and Turbo ones charge half. `pvc show` works it out.

Changing the usual model (`USUAL_MODEL` in `setup.rs`) changes how every video sounds, and records everything again. Ask first.

Bead names: `cross`, `p1` to `p5` (the strand, `p1` nearest the cross), `medal`, `d3b10` (decade 3, bead 10), `L2` to `L5` (the big beads in the loop), `g0` to `g5` (the chain, where the Glory Be is said).

### Useful extras

```
pvc frame luminous --scene d3b10 --to still.png   draw one moment as a picture, then look at it
pvc rerecord luminous --scene d3b10               throw away that bead's recording, to make it afresh
pvc save "My Joyful" joyful --voice "Bella"       keep a setup under a name
pvc make my-joyful                                use it again
pvc voices                                        the voices in the account (free)
pvc models                                        the models, and what each charges (free)
pvc credits                                       credits left, and when the count starts again (free)
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

### How it fits together

A setup is a small JSON file that holds only what differs from the defaults. The rest follows from that.

- **Sparse in, full out.** `library::resolve` folds in whatever the setup `extends`. A child's rules are added after its parent's. `setup::from_doc` lays the result over `Setup::default()` and checks it. A `null` puts a setting back to its default. Both shells keep the sparse setup and hand it to the engine, so saving keeps only the changes.
- **The plan is the whole video as data.** `plan::build` lists every scene in order and picks the narrowest rule for each. Everything after it reads the plan, not the setup.
- **`renderer/frame.js` draws from the plan and decides nothing.** It is a plain script: no imports, no build step. The engine puts it in a page and photographs it with Chrome or Edge, with no window (`frames.rs`, `cdp.rs`). The app's preview loads the same file (`Preview.svelte`).
- **The window works nothing out.** After every change the editor (`src/lib/stores/editor.svelte.ts`) sends the sparse setup to `setup_look` and shows the plan and the cost that come back.
- **Sound sets the timing.** Each still is held for as long as its sound lasts. With no voice, the pace settings say how long.
- **One gate for credits.** `make::look` is free. `make::make` stops with `NeedsApproval` unless the approved number covers the credits still to spend. That is letters times what each model charges (`Rates` in `speech.rs`, read from the list `speech::models` fetches and keeps for a day). `speech::record` is the second lock: it adds up what ElevenLabs says it charged, and stops before going over the number.
- **Only `Client::speak` spends.** Every other call to ElevenLabs is a free list or lookup. `speak` tries again only after a "busy" answer, when nothing was recorded. Never retry it after any other failure: a recording may have been made and paid for.

### What has to stay in step

- **A new setting:** the field and its default in `setup.rs`, its description in `reference.rs`, its type in `src/lib/types.ts`, a control in the matching panel in `src/lib/views/panels/`, then `docs/settings.md`. If the drawing needs it, carry it into the scene in `plan.rs` and read it in `renderer/frame.js`.
- **A new preset:** the file in `presets/`, and a line in the `PRESETS` list in `library.rs`.
- **A new app command:** the function in `src-tauri/src/commands/`, the list in `src-tauri/src/lib.rs`, and `src/lib/api.ts`.
- **The five groups that narrow** (`voice`, `pace`, `words`, `picture`, `colors`) are named in `setup.rs`, `reference.rs` and `editor.svelte.ts`.
- **The app's id** is in `paths.rs` and `src-tauri/tauri.conf.json`. It names the data folder and the saved key.

### Kept speech is paid for

Setups, pictures and recorded speech live outside the repo, in the app's own folder (`%APPDATA%\gg.cathedral.prayer-video-creator` on Windows). The app and `pvc` share it.

Each recording is filed under a hash of the voice, its settings and the exact words (`raw_key` in `speech.rs`). Change what goes into that hash, or one letter of a prayer, and the person pays to record it all again.

A setting added later joins the hash only where a setup uses it (`extras` in `speech.rs`), so everything recorded before it existed is still found. Add a new one the same way: at the end of that list, never as a JSON object (the app and `pvc` can order an object's keys differently), and pin the result in the test beside it.

### Rules that come from CLAUDE.md

- **The wording of prayers** is in `crates/engine/src/prayers.rs`, copied from the first video. Do not reword it.
- **The bead layout** is in `crates/engine/src/beads.rs`. Tests pin 59 beads and their places.
- **The order of prayers** is in `crates/engine/src/plan.rs`. A test checks it against the table in CLAUDE.md.
- **A new kind of beads** needs its layout and prayers checked against a source that tradition trusts, and the person asked which kind comes next. Do not add one from memory.
- **Scripture** is NLT, word for word, supplied by the person. Do not type it from memory.
- **Every setting has a default** and is described in `crates/engine/src/reference.rs`. A test fails if you add a setting without describing it. After changing settings, run `pvc options --markdown > docs/settings.md`.
- **Anything that changes how the first video looks or sounds**: ask first.
- **Anything the person reads** (an error, a note, a label in the app) says what went wrong and what to do, in plain words.

### Checking your work

```
cargo test -p pvc-engine                              the engine's tests
cargo test -p pvc-engine the_narrowest_choice_wins    one test, by name
cargo test -p pvc-engine plan::                       the tests in one file
npm run check                                         the window's type check
npm test                                              the window's tests
node --test src/lib/utils/path.test.ts                one of the window's test files
build.bat                                             everything above, then builds the app and pvc
start.bat                                             opens the app from this folder
```

There is no linter and no formatter. Do not run `cargo fmt`. It would rewrite nearly every Rust file.

To see a change to the drawing, use `pvc frame` and look at the picture.

To look at the app's window while the person has their own copy open, start a second copy with `PVC_SECOND_WINDOW=1`, its own `PVC_DATA_DIR`, and its own `WEBVIEW2_USER_DATA_FOLDER`. Without the first, the app only shows the window that is already open. Close your copy when you are done. If they started the app with `start.bat`, it rebuilds and restarts by itself each time you change the code.

To test the voice without spending credits, run the stand-in service, `node dev/stand-in-elevenlabs.mjs` (Windows only). The steps are at the top of that file. It prints what each request asked for, which is how to see that a setting reached ElevenLabs. Always set `PVC_DATA_DIR` to a scratch folder while you do, so stand-in speech is never kept as if it were the person's real voice. Pass `--to` a scratch folder too, or the test video lands in their Videos folder.

To drive the app's window, start your copy with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9347`. Then `pvc dev-shot 9347 script.js --to shot.png` runs a script in the window and takes a picture of it. Every control has a `data-testid`.

Settings for a test run, set outside the tool:

```
PVC_DATA_DIR              a different folder for setups and kept speech
PVC_ELEVENLABS_URL        where ElevenLabs is (the stand-in)
ELEVENLABS_API_KEY        a key for one run, used before the saved one
PVC_FFMPEG, PVC_BROWSER   where FFmpeg or the browser is
PVC_KEEP_WORK             keep the stills and working files after a run
PVC_SECOND_WINDOW         let a second copy of the app start
```

### Versioning

The version lives in `package.json`. When it changes, change `src-tauri/tauri.conf.json` and the root `Cargo.toml` to match. `build.bat` makes a test build named like `0.1.0-a1b2c3d`. `build.bat --release` makes the real `0.1.0` with an installer.
