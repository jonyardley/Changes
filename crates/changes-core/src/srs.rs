//! Spaced repetition: skill identity, review state, the `Scheduler` trait,
//! and the FSRS-backed implementation (docs/specs/srs-persistence.md).
//! `fsrs` types never cross this module's boundary — bridge and storage
//! carry only our own fields, so the algorithm is swappable (decision 6).

use fsrs::{MemoryState, DEFAULT_PARAMETERS, FSRS};
use serde::{Deserialize, Serialize};

use crate::theory::{Degree, Key, Mode};

const DAY_MS: i64 = 86_400_000;
/// FSRS desired retention (spec decision 5) — config, not persisted.
const DESIRED_RETENTION: f32 = 0.9;

/// One trainable skill: a degree color heard against a mode — key-agnostic
/// (spec decision 1).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct SkillId {
    pub mode: Mode,
    pub degree: Degree,
}

impl SkillId {
    /// Storage key, e.g. "major:3" — the `review_states` primary key.
    pub fn key(self) -> String {
        let mode = match self.mode {
            Mode::Major => "major",
            Mode::Minor => "minor",
        };
        format!("{mode}:{}", self.degree.semitones())
    }

    pub fn label_in(self, key: Key) -> &'static str {
        key.label_of(self.degree)
    }
}

/// The UI's honest two-button self-grade (spec decision 3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Grade {
    Got,
    Missed,
}

/// Bridge/storage-crossing review state — one row per skill.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct ReviewState {
    pub skill: SkillId,
    pub stability: f32,
    pub difficulty: f32,
    pub last_reviewed_at_ms: i64,
    pub due_at_ms: i64,
}

/// Bridge/storage-crossing history row (client-minted ulid id).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct ReviewLog {
    pub id: String,
    pub skill: SkillId,
    pub grade: Grade,
    pub reviewed_at_ms: i64,
}

/// The swap seam (spec decision 2): SM-2 or first-partied FSRS equations
/// slot in here without touching callers.
pub trait Scheduler {
    fn review(
        &self,
        prior: Option<&ReviewState>,
        skill: SkillId,
        grade: Grade,
        now_ms: i64,
    ) -> ReviewState;
}

pub struct FsrsScheduler {
    inner: Option<FSRS>,
}

impl Default for FsrsScheduler {
    fn default() -> Self {
        // Construction with the published defaults doesn't fail in
        // practice; if it ever does, `review` degrades to the deterministic
        // fallback schedule rather than panicking mid-session.
        FsrsScheduler {
            inner: FSRS::new(&DEFAULT_PARAMETERS).ok(),
        }
    }
}

impl Scheduler for FsrsScheduler {
    fn review(
        &self,
        prior: Option<&ReviewState>,
        skill: SkillId,
        grade: Grade,
        now_ms: i64,
    ) -> ReviewState {
        self.fsrs_review(prior, skill, grade, now_ms)
            .unwrap_or_else(|| fallback_review(prior, skill, grade, now_ms))
    }
}

impl FsrsScheduler {
    fn fsrs_review(
        &self,
        prior: Option<&ReviewState>,
        skill: SkillId,
        grade: Grade,
        now_ms: i64,
    ) -> Option<ReviewState> {
        let fsrs = self.inner.as_ref()?;
        let memory = prior.map(|state| MemoryState {
            stability: state.stability,
            difficulty: state.difficulty,
        });
        let elapsed_days = prior.map_or(0, |state| {
            ((now_ms - state.last_reviewed_at_ms).max(0) / DAY_MS) as u32
        });
        let next = fsrs
            .next_states(memory, DESIRED_RETENTION, elapsed_days)
            .ok()?;
        let outcome = match grade {
            Grade::Got => next.good,
            Grade::Missed => next.again,
        };
        Some(ReviewState {
            skill,
            stability: outcome.memory.stability,
            difficulty: outcome.memory.difficulty,
            last_reviewed_at_ms: now_ms,
            due_at_ms: now_ms + (outcome.interval as f64 * DAY_MS as f64) as i64,
        })
    }
}

/// Deterministic degraded-mode schedule (FSRS construction failed): missed
/// → tomorrow; got → double the prior interval, floor one day.
fn fallback_review(
    prior: Option<&ReviewState>,
    skill: SkillId,
    grade: Grade,
    now_ms: i64,
) -> ReviewState {
    let prior_interval = prior
        .map(|state| (state.due_at_ms - state.last_reviewed_at_ms).max(DAY_MS))
        .unwrap_or(DAY_MS);
    let interval = match grade {
        Grade::Missed => DAY_MS,
        Grade::Got => prior_interval * 2,
    };
    ReviewState {
        skill,
        stability: prior.map_or(1.0, |s| s.stability),
        difficulty: prior.map_or(5.0, |s| s.difficulty),
        last_reviewed_at_ms: now_ms,
        due_at_ms: now_ms + interval,
    }
}

/// Session queue (spec decision 6): most-overdue due skills, then unseen
/// skills in curriculum order, then the soonest-due remainder — deduped,
/// capped at `count`. `pool` is the current rung's skill pool in
/// curriculum order.
pub fn build_queue(
    states: &[ReviewState],
    pool: &[SkillId],
    count: usize,
    now_ms: i64,
) -> Vec<SkillId> {
    let mut queue: Vec<SkillId> = Vec::with_capacity(count);

    let mut due: Vec<&ReviewState> = states
        .iter()
        .filter(|s| s.due_at_ms <= now_ms && pool.contains(&s.skill))
        .collect();
    due.sort_by_key(|s| s.due_at_ms);
    queue.extend(due.iter().map(|s| s.skill));

    let seen: Vec<SkillId> = states.iter().map(|s| s.skill).collect();
    queue.extend(pool.iter().filter(|s| !seen.contains(s)).copied());

    if queue.len() < count {
        let mut upcoming: Vec<&ReviewState> = states
            .iter()
            .filter(|s| s.due_at_ms > now_ms && pool.contains(&s.skill))
            .collect();
        upcoming.sort_by_key(|s| s.due_at_ms);
        queue.extend(upcoming.iter().map(|s| s.skill));
    }

    let mut deduped: Vec<SkillId> = Vec::with_capacity(count);
    for skill in queue {
        if !deduped.contains(&skill) {
            deduped.push(skill);
        }
        if deduped.len() == count {
            break;
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000_000;

    fn skill(semitones: u8) -> SkillId {
        SkillId {
            mode: Mode::Major,
            degree: Degree::new(semitones),
        }
    }

    #[test]
    fn skill_key_format_is_the_storage_contract() {
        assert_eq!(skill(3).key(), "major:3");
        assert_eq!(
            SkillId {
                mode: Mode::Minor,
                degree: Degree::new(10)
            }
            .key(),
            "minor:10"
        );
    }

    #[test]
    fn first_got_schedules_days_out_first_miss_schedules_sooner() {
        let scheduler = FsrsScheduler::default();
        let got = scheduler.review(None, skill(4), Grade::Got, NOW);
        let missed = scheduler.review(None, skill(4), Grade::Missed, NOW);
        assert!(got.due_at_ms > NOW + DAY_MS, "good ≥ a day out");
        assert!(missed.due_at_ms < got.due_at_ms, "again comes back sooner");
        assert_eq!(got.last_reviewed_at_ms, NOW);
    }

    #[test]
    fn repeated_got_grows_the_interval() {
        let scheduler = FsrsScheduler::default();
        let mut state = scheduler.review(None, skill(4), Grade::Got, NOW);
        let mut previous_interval = state.due_at_ms - NOW;
        for _ in 0..4 {
            let now = state.due_at_ms;
            state = scheduler.review(Some(&state), skill(4), Grade::Got, now);
            let interval = state.due_at_ms - now;
            assert!(interval > previous_interval, "spacing must grow");
            previous_interval = interval;
        }
    }

    #[test]
    fn miss_shrinks_the_next_interval() {
        let scheduler = FsrsScheduler::default();
        let mut state = scheduler.review(None, skill(4), Grade::Got, NOW);
        for _ in 0..3 {
            let now = state.due_at_ms;
            state = scheduler.review(Some(&state), skill(4), Grade::Got, now);
        }
        let now = state.due_at_ms;
        let grown = state.due_at_ms - state.last_reviewed_at_ms;
        let missed = scheduler.review(Some(&state), skill(4), Grade::Missed, now);
        assert!((missed.due_at_ms - now) < grown);
    }

    #[test]
    fn scheduling_is_deterministic() {
        let scheduler = FsrsScheduler::default();
        let a = scheduler.review(None, skill(2), Grade::Got, NOW);
        let b = scheduler.review(None, skill(2), Grade::Got, NOW);
        assert_eq!(a, b);
    }

    #[test]
    fn fallback_schedule_is_sane() {
        let state = fallback_review(None, skill(0), Grade::Got, NOW);
        assert_eq!(state.due_at_ms, NOW + 2 * DAY_MS);
        let missed = fallback_review(Some(&state), skill(0), Grade::Missed, state.due_at_ms);
        assert_eq!(missed.due_at_ms - state.due_at_ms, DAY_MS);
    }

    #[test]
    fn queue_orders_due_then_unseen_then_upcoming() {
        let pool: Vec<SkillId> = [0u8, 2, 4, 5, 7].iter().map(|&s| skill(s)).collect();
        let states = vec![
            ReviewState {
                skill: skill(4),
                stability: 1.0,
                difficulty: 5.0,
                last_reviewed_at_ms: NOW - 3 * DAY_MS,
                due_at_ms: NOW - 2 * DAY_MS, // most overdue
            },
            ReviewState {
                skill: skill(2),
                stability: 1.0,
                difficulty: 5.0,
                last_reviewed_at_ms: NOW - DAY_MS,
                due_at_ms: NOW - 1, // just due
            },
            ReviewState {
                skill: skill(0),
                stability: 1.0,
                difficulty: 5.0,
                last_reviewed_at_ms: NOW,
                due_at_ms: NOW + DAY_MS, // upcoming
            },
        ];
        let queue = build_queue(&states, &pool, 5, NOW);
        assert_eq!(
            queue,
            vec![skill(4), skill(2), skill(5), skill(7), skill(0)],
            "due (overdue first), unseen in curriculum order, then upcoming"
        );
    }

    #[test]
    fn queue_caps_and_dedupes() {
        let pool: Vec<SkillId> = (0..12u8).map(skill).collect();
        let queue = build_queue(&[], &pool, 5, NOW);
        assert_eq!(queue.len(), 5);
        let unique: std::collections::HashSet<_> = queue.iter().map(|s| s.key()).collect();
        assert_eq!(unique.len(), 5);
    }
}
