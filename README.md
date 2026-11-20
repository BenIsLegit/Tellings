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

## Where it stands

- **Made so far:** one video, the Luminous Mysteries of the rosary. It runs 13 minutes 10 seconds.
- **In this repo:** the rough scripts that made that video, in [`prototype/`](prototype/). They are a reference. They only know one kind of rosary.
- **Not built yet:** the tool itself.
