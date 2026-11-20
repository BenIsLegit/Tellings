---
name: make-video
description: Make a follow-along prayer video with the pvc tool in this repo. Use when the person asks to make, render or remake a video (a rosary, a set of mysteries, "today's", one decade), to change how a video looks or sounds (voice, pictures, screen size, pace, words), to save or reuse a setup, or to get a sound-only file, captions or the tap-through guide.
---

# Make a prayer video

Everything goes through `pvc` (`.\pvc.bat` on Windows, `./pvc` elsewhere). Never edit code to make a video. The full guide is [AGENTS.md](../../../AGENTS.md). This is the short form.

1. Pick a setup and changes. `pvc list` shows the presets and saved setups. `pvc options` shows every setting.
2. Run `pvc show <setup> [changes]`. Read the `Credits:` line.
3. If it is not "none", tell the person the number and wait for a yes. Do not skip this, even if they approved a different video earlier.
4. Run `pvc make <setup> [changes]`, adding `--approve-credits N` only after that yes.
5. Reply in a few short lines: where the file is, how long, what was spent, any `Note:` lines.

Common changes:

```
--mysteries joyful | sorrowful | glorious | luminous | today
--voice "Name"            --no-voice            --screen tv-1080p
--set words.follow=true   --to "C:\folder"
--rule "mystery=3: voice.name=Bella"        one mystery
--rule "prayer=ourfather: voice.name=George" one kind of prayer
--rule "bead=d3b10: voice.name=Sarah"        one bead
```

Variety, so repeated prayers do not all sound alike. It costs more credits, so run `pvc show` again and ask about the new number:

```
--takes 5                                 five recordings of each prayer take turns
--voice "Bella + George"                  the voices take turns
--set voice.freshEachTime=true            a new recording every time (costs the most)
--set voice.shuffle=true                  mixed-up order
--rule "prayer=hailmary: voice.takes=5"   only the Hail Marys
```

A different ElevenLabs model, or direction for the voice. `pvc models` lists the models and their prices (free). A different model records the prayers again, so run `pvc show` again and ask about the new number:

```
--model eleven_v4                                        the whole video
--rule "prayer=hailmary: voice.model=eleven_flash_v2_5"  part of it
--set "voice.direction=reverent, unhurried"              how to say it (Eleven v3 and v4 only)
```

One recording came out wrong: `pvc rerecord <setup> [changes] --scene d1b3`, then make the video again.

To check a look before making the whole video: `pvc frame <setup> --scene d3b10 --to still.png`, then open the picture.

Never ask for the ElevenLabs key in the conversation. If none is saved, the person adds it in the app under Settings or runs `pvc key set` themselves.

The person reads slowly. Keep replies short and plain.
