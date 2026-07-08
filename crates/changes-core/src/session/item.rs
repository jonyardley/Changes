//! Exercise items and their generator. An item is a degree heard against a
//! key; the session is a pure function of (rung, count, seed) — SRS state
//! joins the inputs in M2b.

use serde::{Deserialize, Serialize};

use crate::rng::SplitMix64;
use crate::theory::{Degree, Key, Mode, PitchClass};

/// Curriculum rungs the generator understands so far (RESEARCH.md ladder).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Rung {
    /// Tonic orientation: strong scale anchors, is-it-home hearing.
    Orientation,
    /// Diatonic degrees in major keys.
    DiatonicMajor,
    /// Minor keys join, all 12 chromatic colors.
    MinorAndChromatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub key: Key,
    pub degree: Degree,
}

impl Item {
    /// The confusable twin for the Banacos compare loop: fixed
    /// adjacent-semitone pairs — each chromatic color with the diatonic
    /// neighbour it masquerades as, and 1↔5 (the classic orientation
    /// confusion). An involution over the 12 colors.
    pub fn confusion_twin(self) -> Degree {
        let twin = match self.degree.semitones() {
            0 => 7,
            7 => 0,
            s @ (1 | 3 | 5 | 8 | 10) => s + 1,
            s @ (2 | 4 | 6 | 9 | 11) => s - 1,
            s => s,
        };
        Degree::new(twin)
    }
}

fn degree_pool(rung: Rung) -> Vec<Degree> {
    match rung {
        Rung::Orientation => [0u8, 4, 7].map(Degree::new).to_vec(),
        Rung::DiatonicMajor => Mode::Major.scale_semitones().map(Degree::new).to_vec(),
        Rung::MinorAndChromatic => Degree::all().to_vec(),
    }
}

fn key_pool(rung: Rung) -> Vec<Key> {
    // Flat-side jazz keys first; every key is fair game by rung 2.
    let tonics = [0u8, 5, 10, 3, 8, 7, 2];
    let mut keys: Vec<Key> = tonics
        .iter()
        .map(|&t| Key::new(PitchClass::new(t), Mode::Major))
        .collect();
    if rung == Rung::MinorAndChromatic {
        keys.extend(
            tonics
                .iter()
                .map(|&t| Key::new(PitchClass::new(t), Mode::Minor)),
        );
    }
    keys
}

/// Deterministic session plan: same (rung, count, seed) → same items.
/// One key per session (the design's key badge is a session-level fact);
/// no immediate degree repeats.
pub fn generate_session(rung: Rung, count: usize, seed: u64) -> Vec<Item> {
    let mut rng = SplitMix64::new(seed);
    let keys = key_pool(rung);
    let key = keys[rng.next_below(keys.len())];
    let pool = degree_pool(rung);

    let mut items = Vec::with_capacity(count);
    let mut previous: Option<Degree> = None;
    for _ in 0..count {
        let degree = loop {
            let candidate = pool[rng.next_below(pool.len())];
            if pool.len() == 1 || Some(candidate) != previous {
                break candidate;
            }
        };
        previous = Some(degree);
        items.push(Item { key, degree });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_session() {
        assert_eq!(
            generate_session(Rung::DiatonicMajor, 12, 99),
            generate_session(Rung::DiatonicMajor, 12, 99)
        );
        assert_ne!(
            generate_session(Rung::DiatonicMajor, 12, 99),
            generate_session(Rung::DiatonicMajor, 12, 100)
        );
    }

    #[test]
    fn rung_pools_match_the_curriculum() {
        for item in generate_session(Rung::Orientation, 50, 1) {
            assert!([0, 4, 7].contains(&item.degree.semitones()));
            assert_eq!(item.key.mode, Mode::Major);
        }
        for item in generate_session(Rung::DiatonicMajor, 50, 2) {
            assert!(item.key.is_diatonic(item.degree));
            assert_eq!(item.key.mode, Mode::Major);
        }
    }

    #[test]
    fn chromatic_rung_reaches_minor_keys_and_all_colors() {
        let mut minor_seen = false;
        let mut colors = std::collections::HashSet::new();
        for seed in 0..40u64 {
            for item in generate_session(Rung::MinorAndChromatic, 24, seed) {
                minor_seen |= item.key.mode == Mode::Minor;
                colors.insert(item.degree.semitones());
            }
        }
        assert!(minor_seen);
        assert_eq!(colors.len(), 12);
    }

    #[test]
    fn no_back_to_back_repeats_and_one_key_per_session() {
        for seed in 0..20u64 {
            let items = generate_session(Rung::DiatonicMajor, 30, seed);
            for pair in items.windows(2) {
                assert_ne!(pair[0].degree, pair[1].degree);
                assert_eq!(pair[0].key, pair[1].key);
            }
        }
    }

    #[test]
    fn confusion_twin_is_an_involution() {
        for degree in Degree::all() {
            let key = Key::new(PitchClass::C, Mode::Major);
            let item = Item { key, degree };
            let twin = Item {
                key,
                degree: item.confusion_twin(),
            };
            assert_eq!(twin.confusion_twin(), degree);
            assert_ne!(item.confusion_twin(), degree);
        }
    }
}
