//! Where every bead sits.
//!
//! The one rule of this project: someone holding the beads must be able to see
//! which bead they are on. So the drawing has to match the real thing, bead for
//! bead. A standard five-decade rosary has 59 beads:
//!
//! - a crucifix
//! - five beads above it: one big, three small, one big
//! - the medal
//! - five sets of ten small beads, with one big bead between sets
//!
//! The numbers below are the first video's, unchanged (`prototype/core.js`).

use crate::setup::{BeadKind, Direction};
use serde::Serialize;

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SpotKind {
    Cross,
    Big,
    Small,
    /// A stretch of chain with no bead, where the Glory Be is said.
    Gap,
    Medal,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Spot {
    pub id: String,
    pub kind: SpotKind,
    pub x: f32,
    pub y: f32,
    /// For a stretch of chain: the two beads it runs between, in the order you
    /// reach them. The mark for it follows the chain from one to the other.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub between: Option<[String; 2]>,
}

/// Everything needed to draw the beads, in the drawing's own units.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Drawing {
    /// x, y, width, height of the drawing area.
    pub view_box: [f32; 4],
    pub loop_cx: f32,
    pub loop_cy: f32,
    pub loop_r: f32,
    /// The strand from the medal down to the cross: x, top y, bottom y.
    pub strand: [f32; 3],
    pub spots: Vec<Spot>,
}

const CX: f32 = 180.0;
const CY: f32 = 160.0;
const R: f32 = 135.0;
const U_SMALL: f32 = 1.0;
const U_BIG: f32 = 1.8;
const U_MEDAL: f32 = 2.2;

fn round1(v: f64) -> f32 {
    ((v * 10.0).round() / 10.0) as f32
}

pub fn drawing(kind: BeadKind, direction: Direction) -> Drawing {
    match kind {
        BeadKind::FiveDecadeRosary => five_decade(direction),
    }
}

fn five_decade(direction: Direction) -> Drawing {
    let total = (2.0 * U_MEDAL + 45.0 * U_SMALL + 8.0 * U_BIG) as f64;
    let deg = 360.0 / total;
    let side = if direction == Direction::RightFirst { 1.0 } else { -1.0 };
    // Position `u` units of chain along the loop, starting from the medal at the bottom.
    let at = |u: f64| -> (f32, f32) {
        let a = (90.0 - u * deg).to_radians();
        (round1(CX as f64 + side * R as f64 * a.cos()), round1(CY as f64 + R as f64 * a.sin()))
    };
    let mut spots = Vec::with_capacity(72);
    let mut push = |id: String, kind: SpotKind, (x, y): (f32, f32)| spots.push(Spot { id, kind, x, y, between: None });

    // The strand, from the cross up to the medal.
    push("cross".into(), SpotKind::Cross, (CX, 445.0));
    push("p1".into(), SpotKind::Big, (CX, 401.0));
    push("p2".into(), SpotKind::Small, (CX, 382.0));
    push("p3".into(), SpotKind::Small, (CX, 368.0));
    push("p4".into(), SpotKind::Small, (CX, 354.0));
    push("g0".into(), SpotKind::Gap, (CX, 340.5));
    push("p5".into(), SpotKind::Big, (CX, 325.0));
    push("medal".into(), SpotKind::Medal, at(0.0));

    // The loop: five decades, a big bead before each one after the first.
    let mut u = U_MEDAL as f64;
    for d in 1..=5 {
        if d > 1 {
            push(format!("L{d}"), SpotKind::Big, at(u));
            u += U_BIG as f64;
        }
        for b in 1..=10 {
            push(format!("d{d}b{b}"), SpotKind::Small, at(u));
            if b < 10 {
                u += U_SMALL as f64;
            }
        }
        let (gap_len, frac) = if d < 5 { (U_BIG as f64, 0.45) } else { (U_MEDAL as f64, 0.36) };
        push(format!("g{d}"), SpotKind::Gap, at(u + gap_len * frac));
        u += gap_len;
    }

    // Each stretch of chain runs between two beads. Name them, so the mark for a
    // prayer said on the chain can follow the chain instead of looking like a bead.
    for spot in spots.iter_mut().filter(|s| s.kind == SpotKind::Gap) {
        let d: u32 = spot.id[1..].parse().expect("gaps are g0 to g5");
        spot.between = Some(match d {
            0 => ["p4".into(), "p5".into()],
            5 => ["d5b10".into(), "medal".into()],
            _ => [format!("d{d}b10"), format!("L{}", d + 1)],
        });
    }

    Drawing { view_box: [30.0, 10.0, 300.0, 478.0], loop_cx: CX, loop_cy: CY, loop_r: R, strand: [CX, 295.0, 418.0], spots }
}

/// True for any id a five-decade rosary uses.
pub fn is_bead_id(id: &str) -> bool {
    five_decade(Direction::RightFirst).spots.iter().any(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rosary() -> Drawing {
        drawing(BeadKind::FiveDecadeRosary, Direction::RightFirst)
    }

    #[test]
    fn a_rosary_has_59_beads() {
        let d = rosary();
        let count = |k: SpotKind| d.spots.iter().filter(|s| s.kind == k).count();
        assert_eq!(count(SpotKind::Small), 53, "three above the cross and five sets of ten");
        assert_eq!(count(SpotKind::Big), 6, "two above the cross and four between the sets");
        assert_eq!(count(SpotKind::Small) + count(SpotKind::Big), 59);
        assert_eq!(count(SpotKind::Cross), 1);
        assert_eq!(count(SpotKind::Medal), 1);
        assert_eq!(count(SpotKind::Gap), 6, "one stretch of chain for each Glory Be");
    }

    #[test]
    fn the_strand_is_one_big_three_small_one_big() {
        let d = rosary();
        let kinds: Vec<SpotKind> = ["p1", "p2", "p3", "p4", "p5"].iter().map(|id| d.spots.iter().find(|s| s.id == *id).unwrap().kind).collect();
        assert_eq!(kinds, [SpotKind::Big, SpotKind::Small, SpotKind::Small, SpotKind::Small, SpotKind::Big]);
        // The cross hangs lowest, the medal joins the strand to the loop.
        let y = |id: &str| d.spots.iter().find(|s| s.id == id).unwrap().y;
        assert!(y("cross") > y("p1") && y("p1") > y("p2") && y("p4") > y("p5") && y("p5") > y("medal"));
    }

    #[test]
    fn beads_sit_where_they_did_in_the_first_video() {
        let d = rosary();
        let at = |id: &str| {
            let s = d.spots.iter().find(|s| s.id == id).unwrap();
            (s.x, s.y)
        };
        // Worked out by hand from prototype/core.js: total = 63.8 units, 5.6426 degrees each.
        assert_eq!(at("medal"), (180.0, 295.0));
        assert_eq!(at("cross"), (180.0, 445.0));
        assert_eq!(at("g0"), (180.0, 340.5));
        let (x, y) = at("d1b1");
        assert!((x - 209.0).abs() < 0.11 && (y - 291.8).abs() < 0.11, "d1b1 at {x},{y}");
        // Decade 1 climbs the right-hand side.
        assert!(at("d1b10").0 > 180.0 && at("d1b10").1 < at("d1b1").1);
        // Decade 3 sits across the top, decade 5 comes down the left.
        assert!(at("d3b5").1 < 40.0);
        assert!(at("d5b10").0 < 180.0);
    }

    #[test]
    fn each_stretch_of_chain_runs_between_two_beads() {
        let d = rosary();
        let at = |id: &str| d.spots.iter().find(|s| s.id == id).unwrap_or_else(|| panic!("no spot called {id}"));
        for gap in d.spots.iter().filter(|s| s.kind == SpotKind::Gap) {
            let [a, b] = gap.between.clone().unwrap_or_else(|| panic!("{} does not say which beads it joins", gap.id));
            let (a, b) = (at(&a), at(&b));
            assert!(a.kind != SpotKind::Gap && b.kind != SpotKind::Gap);
            // The chain's own point lies between its two beads, nearer to both than they are to each other.
            let dist = |p: &Spot, q: &Spot| ((p.x - q.x).powi(2) + (p.y - q.y).powi(2)).sqrt();
            assert!(dist(a, gap) < dist(a, b) && dist(gap, b) < dist(a, b), "{} is not between {} and {}", gap.id, a.id, b.id);
        }
        // Only the chain has this. A bead is a bead.
        assert!(d.spots.iter().filter(|s| s.kind != SpotKind::Gap).all(|s| s.between.is_none()));
        // The Glory Be after decade 2 is said between its tenth bead and the next big bead.
        assert_eq!(at("g2").between, Some(["d2b10".to_string(), "L3".to_string()]));
        assert_eq!(at("g0").between, Some(["p4".to_string(), "p5".to_string()]));
        assert_eq!(at("g5").between, Some(["d5b10".to_string(), "medal".to_string()]));
    }

    #[test]
    fn going_the_other_way_mirrors_the_loop() {
        let r = rosary();
        let l = drawing(BeadKind::FiveDecadeRosary, Direction::LeftFirst);
        for (a, b) in r.spots.iter().zip(l.spots.iter()) {
            assert_eq!(a.id, b.id);
            assert!((a.x + b.x - 360.0).abs() < 0.11, "{} {} {}", a.id, a.x, b.x);
            assert_eq!(a.y, b.y);
        }
    }

    #[test]
    fn no_two_beads_touch() {
        let d = rosary();
        let beads: Vec<&Spot> = d.spots.iter().filter(|s| matches!(s.kind, SpotKind::Small | SpotKind::Big)).collect();
        for (i, a) in beads.iter().enumerate() {
            for b in beads.iter().skip(i + 1) {
                let dist = ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
                assert!(dist > 11.0, "{} and {} are {dist} apart", a.id, b.id);
            }
        }
    }
}
