//! Exercise items and session planning. Since M2b the SRS queue decides
//! WHAT is practised (docs/specs/srs-persistence.md); this module decides
//! HOW it lands in a session: one key, shuffled presentation, still a pure
//! function of its inputs.

use serde::{Deserialize, Serialize};

use crate::rng::SplitMix64;
use crate::srs::SkillId;
use crate::theory::{Degree, Key, Mode, PitchClass};

/// Curriculum rungs the planner understands so far (RESEARCH.md ladder).
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

impl Rung {
    /// The rung's skill pool in curriculum order — the SRS queue's "unseen"
    /// ordering (spec decision 6).
    pub fn skill_pool(self) -> Vec<SkillId> {
        let major = |semis: &[u8]| {
            semis
                .iter()
                .map(|&s| SkillId {
                    mode: Mode::Major,
                    degree: Degree::new(s),
                })
                .collect::<Vec<_>>()
        };
        match self {
            Rung::Orientation => major(&[0, 4, 7]),
            Rung::DiatonicMajor => major(&Mode::Major.scale_semitones()),
            Rung::MinorAndChromatic => {
                let mut pool = major(&core::array::from_fn::<u8, 12, _>(|i| i as u8));
                pool.extend(Degree::all().map(|degree| SkillId {
                    mode: Mode::Minor,
                    degree,
                }));
                pool
            }
        }
    }
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

/// Flat-side jazz keys first; every key is fair game.
fn key_pool(mode: Mode) -> Vec<Key> {
    let tonics = [0u8, 5, 10, 3, 8, 7, 2];
    tonics
        .iter()
        .map(|&t| Key::new(PitchClass::new(t), mode))
        .collect()
}

/// Turn the SRS queue into a session plan: sessions are single-key hence
/// single-mode (the key badge is a session-level fact), so the plan takes
/// the mode of the queue's head — the highest-priority skill — and keeps
/// only that mode's skills; presentation order is shuffled (a queue in
/// curriculum order would let the user predict the answer), keys chosen by
/// seed. Pure function of (queue, seed).
pub fn plan_session(queue: &[SkillId], seed: u64) -> Vec<Item> {
    let mut rng = SplitMix64::new(seed);
    let Some(head) = queue.first() else {
        return Vec::new();
    };
    let mode = head.mode;
    let keys = key_pool(mode);
    let key = keys[rng.next_below(keys.len())];

    let mut degrees: Vec<Degree> = queue
        .iter()
        .filter(|skill| skill.mode == mode)
        .map(|skill| skill.degree)
        .collect();
    // Fisher–Yates with the session rng.
    for i in (1..degrees.len()).rev() {
        degrees.swap(i, rng.next_below(i + 1));
    }
    degrees
        .into_iter()
        .map(|degree| Item { key, degree })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skills(mode: Mode, semis: &[u8]) -> Vec<SkillId> {
        semis
            .iter()
            .map(|&s| SkillId {
                mode,
                degree: Degree::new(s),
            })
            .collect()
    }

    #[test]
    fn same_seed_same_plan() {
        let queue = skills(Mode::Major, &[0, 2, 4, 5, 7, 9, 11]);
        assert_eq!(plan_session(&queue, 9), plan_session(&queue, 9));
        assert_ne!(plan_session(&queue, 9), plan_session(&queue, 10));
    }

    #[test]
    fn plan_preserves_queue_content_and_uses_one_key() {
        let queue = skills(Mode::Major, &[0, 2, 4, 5, 7]);
        let plan = plan_session(&queue, 3);
        assert_eq!(plan.len(), 5);
        let mut planned: Vec<u8> = plan.iter().map(|i| i.degree.semitones()).collect();
        planned.sort_unstable();
        assert_eq!(planned, vec![0, 2, 4, 5, 7]);
        assert!(plan.windows(2).all(|w| w[0].key == w[1].key));
    }

    #[test]
    fn plan_filters_to_the_head_skills_mode() {
        let mut queue = skills(Mode::Minor, &[3]);
        queue.extend(skills(Mode::Major, &[4, 7]));
        let plan = plan_session(&queue, 1);
        assert_eq!(plan.len(), 1, "major skills wait for a major session");
        assert_eq!(plan[0].key.mode, Mode::Minor);
    }

    #[test]
    fn empty_queue_plans_nothing() {
        assert!(plan_session(&[], 1).is_empty());
    }

    #[test]
    fn rung_pools_are_curriculum_ordered_and_sized() {
        assert_eq!(Rung::Orientation.skill_pool().len(), 3);
        assert_eq!(Rung::DiatonicMajor.skill_pool().len(), 7);
        assert_eq!(Rung::MinorAndChromatic.skill_pool().len(), 24);
        assert_eq!(
            Rung::DiatonicMajor.skill_pool()[0].degree.semitones(),
            0,
            "curriculum order starts at the tonic"
        );
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
