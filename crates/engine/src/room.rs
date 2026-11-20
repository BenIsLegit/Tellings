//! A room for the voice: the echo of a chapel, a church or a cathedral.
//!
//! The room is added once, to the finished voice track, so it costs no
//! credits, changes no timing, and can be changed without recording anything
//! again. The echo rings on into the pauses between prayers.
//!
//! No recording of a real building is used or shipped. A room is described by
//! what one clap would sound like in it: the clap itself, a short wait while
//! the sound crosses to the walls, then a wash of echo that fades, its high
//! notes first. That clap is worked out here from a few numbers, the same way
//! every time, and FFmpeg lays it over the voice (`audio::master`).

use crate::setup::{Room, RoomKind};
use crate::speech::SR;

/// How a room rings.
struct Shape {
    /// Seconds before the first echo comes back from the walls.
    wait: f32,
    /// Seconds for the echo to die away, to a thousandth of where it began.
    decay: f32,
    /// How bright the echo is when it starts and when it ends, in hertz. Stone
    /// swallows high notes first, and a bigger room swallows more of them.
    bright: (f32, f32),
    /// Makes each room's echo its own.
    seed: u64,
}

fn shape(kind: RoomKind) -> Option<Shape> {
    match kind {
        RoomKind::None => None,
        RoomKind::Chapel => Some(Shape { wait: 0.012, decay: 1.2, bright: (9000.0, 3000.0), seed: 1 }),
        RoomKind::Church => Some(Shape { wait: 0.025, decay: 2.5, bright: (8000.0, 2200.0), seed: 2 }),
        RoomKind::Cathedral => Some(Shape { wait: 0.045, decay: 4.5, bright: (7000.0, 1600.0), seed: 3 }),
    }
}

/// The next number from -1 to 1. It looks like chance and is not: the same
/// seed always gives the same run, so a video comes out the same twice.
fn noise(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut x = *state;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 40) as f32 / (1u64 << 23) as f32 - 1.0
}

/// How strong a sound is across the notes a voice is made of, roughly 250 to
/// 3,500 hertz. The echo is measured there, so that "as loud as the voice"
/// means what an ear would say, not what a meter counting every hiss would.
fn voice_strength(sound: &[f32]) -> f64 {
    let turn = |hertz: f32| 1.0 - (-std::f32::consts::TAU * hertz / SR as f32).exp();
    let (low_turn, high_turn) = (turn(250.0), turn(3500.0));
    let (mut below, mut kept) = (0f32, 0f32);
    let mut total = 0f64;
    // Run on a little past the end, so the last of the sound is counted too.
    for x in sound.iter().copied().chain(std::iter::repeat_n(0.0, SR as usize / 10)) {
        below += low_turn * (x - below);
        kept += high_turn * ((x - below) - kept);
        total += (kept as f64) * (kept as f64);
    }
    total
}

/// What one clap sounds like in the room, in samples: the clap, then its echo.
/// Laid over the voice, it puts the voice in that room. `None` when the setup
/// asks for no room, and then the sound is left exactly as it was.
pub fn impulse(room: &Room) -> Option<Vec<f32>> {
    let shape = shape(room.kind)?;
    if room.amount <= 0.0 {
        return None;
    }
    let rate = SR as f32;
    let mut state = shape.seed;
    let (mut soft, mut rumble) = (0f32, 0f32);
    let rumble_turn = 1.0 - (-std::f32::consts::TAU * 120.0 / rate).exp();
    let mut echo: Vec<f32> = (0..(shape.decay * rate) as usize)
        .map(|i| {
            let t = i as f32 / rate;
            // The high notes die first: the echo darkens as it fades.
            let ceiling = shape.bright.1 + (shape.bright.0 - shape.bright.1) * (-3.0 * t / shape.decay).exp();
            let hold = (-std::f32::consts::TAU * ceiling / rate).exp();
            soft = (1.0 - hold) * noise(&mut state) + hold * soft;
            // Darkening must not also quieten it. Only `decay` says how fast it fades.
            let level = soft * ((1.0 + hold) / (1.0 - hold)).sqrt();
            // No rumble: a room does not hum.
            rumble += rumble_turn * (level - rumble);
            // It swells in over a few thousandths of a second, so there is no click, then fades.
            let swell = 1.0 - (-t / 0.008).exp();
            let fade = (-(1000f32.ln()) * t / shape.decay).exp();
            (level - rumble) * swell * fade
        })
        .collect();

    // Set the echo's strength against the clap's: at 1 it is as loud as the voice.
    let mut clap = vec![0f32; 64];
    clap[0] = 1.0;
    let scale = room.amount.min(1.0) * (voice_strength(&clap) / voice_strength(&echo).max(1e-12)).sqrt() as f32;
    echo.iter_mut().for_each(|x| *x *= scale);

    let wait = ((shape.wait * rate) as usize).max(1);
    let mut out = vec![0f32; wait + echo.len()];
    out[0] = 1.0;
    out[wait..].copy_from_slice(&echo);
    Some(out)
}

/// The echo adds to how loud the whole thing is. This is how much to turn the
/// voice down, in decibels, so a video with a room is no louder than one without.
pub fn makes_louder_by(room: &Room) -> f64 {
    let amount = room.amount.clamp(0.0, 1.0) as f64;
    10.0 * (1.0 + amount * amount).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room(kind: RoomKind, amount: f32) -> Room {
        Room { kind, amount }
    }

    fn strength(sound: &[f32]) -> f64 {
        sound.iter().map(|x| (*x as f64) * (*x as f64)).sum()
    }

    #[test]
    fn no_room_means_nothing_is_added() {
        assert_eq!(impulse(&Room::default()), None);
        assert_eq!(impulse(&room(RoomKind::None, 1.0)), None);
        // A room turned all the way down is no room.
        assert_eq!(impulse(&room(RoomKind::Church, 0.0)), None);
    }

    #[test]
    fn a_room_sounds_the_same_every_time() {
        for kind in [RoomKind::Chapel, RoomKind::Church, RoomKind::Cathedral] {
            assert_eq!(impulse(&room(kind, 0.3)), impulse(&room(kind, 0.3)));
        }
        // Each room is its own.
        assert_ne!(impulse(&room(RoomKind::Chapel, 0.3)).unwrap()[..40_000], impulse(&room(RoomKind::Church, 0.3)).unwrap()[..40_000]);
    }

    #[test]
    fn the_voice_comes_first_then_a_wait_then_the_echo() {
        for (kind, wait, decay) in [(RoomKind::Chapel, 0.012, 1.2), (RoomKind::Church, 0.025, 2.5), (RoomKind::Cathedral, 0.045, 4.5)] {
            let clap = impulse(&room(kind, 0.3)).unwrap();
            // The voice itself passes through untouched.
            assert_eq!(clap[0], 1.0);
            // Nothing comes back until the sound has crossed to the walls.
            let first = (wait * SR as f32) as usize;
            assert!(clap[1..first].iter().all(|x| *x == 0.0), "{kind:?}");
            assert!(clap[first..first + 2000].iter().any(|x| *x != 0.0), "{kind:?}");
            // It lasts as long as the room rings, and no longer.
            let seconds = clap.len() as f32 / SR as f32;
            assert!((seconds - (wait + decay)).abs() < 0.001, "{kind:?} lasts {seconds}");
            // No single echo is anywhere near as loud as the voice: it is a wash, not a slap.
            assert!(clap[1..].iter().all(|x| x.abs() < 0.2), "{kind:?}");
        }
    }

    #[test]
    fn the_echo_dies_away() {
        for (kind, decay) in [(RoomKind::Chapel, 1.2f32), (RoomKind::Church, 2.5), (RoomKind::Cathedral, 4.5)] {
            let clap = impulse(&room(kind, 0.5)).unwrap();
            let echo = &clap[1..];
            // Tenth by tenth of the room's ring, each stretch is quieter than the one before.
            let step = echo.len() / 10;
            let parts: Vec<f64> = echo.chunks(step).take(10).map(strength).collect();
            assert!(parts.windows(2).skip(1).all(|w| w[1] < w[0]), "{kind:?}: {parts:?}");
            // By the end it has all but gone: the last tenth holds less than a
            // ten-thousandth of what the loudest tenth held.
            let loudest = parts.iter().cloned().fold(0.0, f64::max);
            assert!(parts[9] < loudest / 10_000.0, "{kind:?}: {parts:?}");
            // Half-way through its ring it is down to about a thirtieth (30 decibels).
            let at = |seconds: f32| -> f64 {
                let mid = (seconds * SR as f32) as usize;
                (strength(&echo[mid - 2400..mid + 2400]) / 4800.0).sqrt()
            };
            let fall = 20.0 * (at(decay * 0.6) / at(decay * 0.1)).log10();
            assert!((-36.0..=-24.0).contains(&fall), "{kind:?} falls {fall:.1} dB over half its ring");
        }
    }

    #[test]
    fn the_high_notes_die_first() {
        // How much of a stretch of echo is fast wiggle: the jump from one sample to the next.
        let wiggle = |part: &[f32]| -> f64 { part.windows(2).map(|w| ((w[1] - w[0]) as f64).powi(2)).sum::<f64>() / strength(part) };
        let clap = impulse(&room(RoomKind::Church, 0.5)).unwrap();
        let echo = &clap[1200..];
        let (early, late) = (&echo[2400..24_000], &echo[72_000..96_000]);
        assert!(wiggle(late) < wiggle(early) * 0.8, "early {:.4}, late {:.4}", wiggle(early), wiggle(late));
    }

    #[test]
    fn more_room_is_more_echo_and_one_is_as_loud_as_the_voice() {
        let echo_of = |amount: f32| -> f64 {
            let clap = impulse(&room(RoomKind::Church, amount)).unwrap();
            voice_strength(&clap[1..])
        };
        let mut voice = vec![0f32; 64];
        voice[0] = 1.0;
        let voice = voice_strength(&voice);
        // At 1 the echo carries as much as the voice does, across the notes a voice is made of.
        assert!((echo_of(1.0) / voice - 1.0).abs() < 0.01, "{}", echo_of(1.0) / voice);
        // At 0.3 it carries 0.3 x 0.3 of it: about a tenth, ten decibels down.
        assert!((echo_of(0.3) / voice - 0.09).abs() < 0.001, "{}", echo_of(0.3) / voice);
        assert!(echo_of(0.6) > echo_of(0.3) * 3.9);
        // And the whole is turned down by as much as the echo adds, so it is no louder.
        assert!((makes_louder_by(&room(RoomKind::Church, 1.0)) - 3.0103).abs() < 0.001);
        assert!((makes_louder_by(&room(RoomKind::Church, 0.3)) - 0.3743).abs() < 0.001);
        assert_eq!(makes_louder_by(&room(RoomKind::Church, 0.0)), 0.0);
    }
}
