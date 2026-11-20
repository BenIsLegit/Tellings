//! Taking turns.
//!
//! A prayer said 53 times does not have to sound the same 53 times. Two things
//! can take turns: the voices that lead, and the recordings ("takes") of one
//! prayer by one voice. Both are worked out here, the same way every time, so
//! the video a person looked at is the video that gets made.

use sha2::{Digest, Sha256};

/// Hands out the next choice from a fixed number of them.
///
/// In turn: 0, 1, 2, 0, 1, 2 …
///
/// Mixed up: the first round still goes 0, 1, 2, so the choices are used in the
/// order they were given and nothing is recorded that is not needed. After
/// that each round is in a different order. Every choice still comes up once a
/// round, and the same one never comes twice running.
#[derive(Clone, Debug)]
pub struct Turns {
    /// How many choices. 0 means no end to them: every pick is a new one.
    count: u32,
    shuffle: bool,
    seed: String,
    given: u64,
    order: Vec<u32>,
    last: Option<u32>,
}

impl Turns {
    pub fn new(count: u32, shuffle: bool, seed: &str) -> Self {
        Turns { count: count.max(1), shuffle, seed: seed.to_string(), given: 0, order: Vec::new(), last: None }
    }

    /// Never the same choice twice: 0, 1, 2, 3 … for as long as it is asked.
    pub fn endless() -> Self {
        Turns { count: 0, shuffle: false, seed: String::new(), given: 0, order: Vec::new(), last: None }
    }

    /// The next choice, counting from 0.
    pub fn next(&mut self) -> u32 {
        let pick = if self.count == 0 {
            self.given as u32
        } else {
            let place = (self.given % self.count as u64) as usize;
            if place == 0 {
                self.order = self.round(self.given / self.count as u64);
            }
            self.order[place]
        };
        self.given += 1;
        self.last = Some(pick);
        pick
    }

    /// The choice handed out last, without moving on. The first one if none has been.
    pub fn again(&self) -> u32 {
        self.last.unwrap_or(0)
    }

    fn round(&self, number: u64) -> Vec<u32> {
        let mut order: Vec<u32> = (0..self.count).collect();
        if !self.shuffle || number == 0 || self.count < 2 {
            return order;
        }
        // The mix comes from the seed, not from chance, so it never changes between runs.
        let digest = Sha256::digest(format!("{}|{number}", self.seed).as_bytes());
        let mut state = u64::from_le_bytes(digest[..8].try_into().expect("eight bytes")) | 1;
        let mut roll = |below: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % below
        };
        for i in (1..order.len()).rev() {
            order.swap(i, roll(i as u64 + 1) as usize);
        }
        if Some(order[0]) == self.last {
            let other = 1 + roll(order.len() as u64 - 1) as usize;
            order.swap(0, other);
        }
        order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(mut t: Turns, n: usize) -> Vec<u32> {
        (0..n).map(|_| t.next()).collect()
    }

    #[test]
    fn in_turn_goes_round_and_round() {
        assert_eq!(run(Turns::new(3, false, "x"), 8), [0, 1, 2, 0, 1, 2, 0, 1]);
        assert_eq!(run(Turns::new(1, false, "x"), 3), [0, 0, 0]);
        assert_eq!(run(Turns::new(1, true, "x"), 3), [0, 0, 0]);
    }

    #[test]
    fn endless_never_repeats() {
        assert_eq!(run(Turns::endless(), 5), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn mixed_up_is_fair_and_never_says_the_same_one_twice_running() {
        for count in 2..=7u32 {
            let picks = run(Turns::new(count, true, "hail mary"), count as usize * 12);
            // The first round is in the order given.
            assert_eq!(picks[..count as usize], (0..count).collect::<Vec<_>>()[..]);
            // Every round uses every choice once.
            for round in picks.chunks(count as usize) {
                let mut sorted = round.to_vec();
                sorted.sort();
                assert_eq!(sorted, (0..count).collect::<Vec<_>>(), "count {count}");
            }
            for pair in picks.windows(2) {
                assert_ne!(pair[0], pair[1], "count {count}: {picks:?}");
            }
        }
        // And it really is mixed: with five choices, some later round differs from the first.
        let picks = run(Turns::new(5, true, "hail mary"), 50);
        assert!(picks.chunks(5).skip(1).any(|r| r != [0, 1, 2, 3, 4]));
    }

    #[test]
    fn the_mix_is_the_same_every_time_and_differs_by_seed() {
        let a = run(Turns::new(5, true, "one"), 40);
        assert_eq!(a, run(Turns::new(5, true, "one"), 40));
        assert_ne!(a, run(Turns::new(5, true, "two"), 40));
    }

    #[test]
    fn again_repeats_the_last_pick_without_moving_on() {
        let mut t = Turns::new(3, false, "x");
        assert_eq!(t.again(), 0);
        t.next();
        t.next();
        assert_eq!(t.again(), 1);
        assert_eq!(t.again(), 1);
        assert_eq!(t.next(), 2);
    }
}
