# Prototype

These are the rough scripts that made the first video. They are here as a reference for what worked. They are not the tool, and the tool does not have to be built from them.

They only know one thing: a standard five-decade rosary.

## The first video

- **What it is:** the Luminous Mysteries, the whole rosary, 13 minutes 10 seconds.
- **Screen:** 2076 x 2152, the inner screen of a Pixel 9 Pro Fold.
- **Voice:** "Imperial - Male" from my ElevenLabs account, sped up to 1.75x.
- **Look:** dark blue background. The rosary is on the left with a gold bead marking the place. The prayer is on the right in large type.
- **Pictures:** one painting per mystery, shown in a circle inside the loop of beads, with the artist credited at the bottom.

| Mystery | Picture |
| --- | --- |
| The Baptism in the Jordan | Piero della Francesca, The Baptism of Christ, c. 1450 |
| The Wedding at Cana | Giotto, Marriage at Cana (detail), Scrovegni Chapel, c. 1305 |
| The Proclamation of the Kingdom | Carl Bloch, The Sermon on the Mount, 1877 |
| The Transfiguration | Raphael, The Transfiguration (detail), 1516-20 |
| The Institution of the Eucharist | Juan de Juanes, The Last Supper, c. 1562 |

## What we learned

- **The voice was too slow.** It was told to speak slowly. I had it sped up to 1.75x. Next time, record at a natural pace.
- **Each prayer was recorded once.** The same Hail Mary plays 53 times. That saved credits and kept the voice even.
- **The audio had to be passed by hand.** The place this was made could not download from ElevenLabs, so I downloaded three files and handed them over. The tool should fetch its own audio.
- **Nobody checked the sound by ear while making it.** The cuts between prayers were checked by measuring the pauses. Listen before trusting a new recording.
- **The Giotto picture is a close-up.** It shows Mary, the steward tasting the wine, and the jars. Christ is not in it. A fuller picture of the scene might be better.
- **The painting inside the loop works.** It keeps the picture and the beads in one place.

## What is in this folder

| File | What it is |
| --- | --- |
| `rosary-guide.html` | The tap-through guide that came first. Open it in a browser. |
| `core.js` | The prayer wording, the four sets of mysteries, the order of all 75 prayers, and where every bead sits. |
| `frame.html` | One frame of the video, drawn at full size. |
| `art.js` | Which picture goes with which mystery, how each is cropped, and its credit. |
| `shoot.py` | Takes a still picture of every frame. |
| `build_audio.py` | Cuts the voice files into prayers, speeds them up and lines them up with the frames. |
| `render.py` | Turns the stills and the sound into the finished video. |

The pictures, the voice files and the finished video are not in this repo.
