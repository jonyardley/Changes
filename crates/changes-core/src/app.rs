use std::collections::HashMap;

use crux_core::macros::effect;
use crux_core::render::{render, RenderOperation};
use crux_core::{App, Command};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::audio::{play_score, PlayScoreOperation, PlayScoreOutput};
use crate::rng::SplitMix64;
use crate::session::{
    compare_side_score, context_score, plan_session, question_score, reveal_score, Item, Rung,
};
use crate::srs::{build_queue, FsrsScheduler, Grade, ReviewLog, Scheduler, SkillId};
use crate::storage::{load_reviews, save_review, StorageOperation, StorageOutput};

/// Ceiling on any session (the full chromatic pool); the pre-session
/// duration pills choose the actual cap via `StartSession::max_items`.
const MAX_ITEMS_CEILING: usize = 24;
/// Starting rung until placement (M5) and rung gating exist.
const RUNG: Rung = Rung::DiatonicMajor;

/// Root Crux application: the Pocket Session state machine
/// (`pre → [listening → gap → reveal → (compare)]* → recap`), manually
/// paced — every `→` that involves the user is a deliberate tap (mvp-plan
/// decision 9). What each session practises comes from the SRS queue
/// (docs/specs/srs-persistence.md); grades feed back into it.
#[derive(Default)]
pub struct Changes {
    scheduler: FsrsScheduler,
}

/// All events the application can process. Bridge-crossing: serialized over
/// positional bincode to the shell — field order is the wire format.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Event {
    /// Pre-session play tap. Seed and clock are shell-provided (time and
    /// randomness arrive via events — core stays deterministic);
    /// `max_items` comes from the duration pills.
    StartSession {
        seed: u64,
        now_ms: i64,
        max_items: u32,
    },
    /// Deliberate user pause (the in-session pause button).
    TapPause,
    ReviewsLoaded(StorageOutput),
    ReviewSaved(StorageOutput),
    /// Gap → reveal (the open-ended thinking gap ends on this tap).
    TapReveal,
    /// Self-grade taps; each doubles as "next" and feeds the SRS.
    GradeGotIt,
    GradeMissedIt,
    /// Leave the compare loop ("I hear it — continue").
    ExitCompare,
    /// Resume after pause — replays the current phase's audio (no item is
    /// ever lost to an interruption).
    TapResume,
    PlaybackFinished(PlayScoreOutput),
    /// Other audio took the session (call, Siri) — shell already stopped us.
    AudioInterrupted,
    /// Route change lost the headphones — pause, never blast the speaker.
    HeadphonesUnplugged,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Phase {
    #[default]
    Pre,
    /// Cadence establishing the key (level bars only — no pitch visuals).
    Context,
    /// The item's note playing; "?" pulses.
    Question,
    /// Open-ended thinking gap; silence on purpose.
    Gap,
    Reveal,
    /// Banacos loop: the missed color alternating with its confusable twin.
    Compare,
    Recap,
}

/// Why the session is held, if it is. User pause and auto-hold look
/// different on screen (design: paused card vs amber interrupted banner).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PauseState {
    #[default]
    None,
    User,
    Interrupted,
}

#[derive(Default, Debug)]
pub struct Model {
    phase: Phase,
    items: Vec<Item>,
    index: usize,
    results: Vec<bool>,
    compare_on_twin: bool,
    is_playing: bool,
    pause: PauseState,
    error: Option<String>,
    max_items: usize,
    review_states: HashMap<String, crate::srs::ReviewState>,
    awaiting_load: bool,
    now_ms: i64,
    seed: u64,
    rng: SplitMix64,
}

impl Model {
    fn current(&self) -> Option<Item> {
        self.items.get(self.index).copied()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct AnswerView {
    /// The degree label ("♭3").
    pub label: String,
    /// The resolution walk ("♭3 · 2 · 1") shown alongside the aural reveal.
    pub resolution: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct CompareView {
    pub missed: String,
    pub twin: String,
    /// Which card is sounding right now (visual highlight syncs to audio).
    pub playing_twin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct RecapView {
    pub got: u32,
    pub missed: u32,
}

/// Bridge-crossing: what shells render. Strings are precomputed here —
/// shells are dumb pipes and make no musical decisions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct ViewModel {
    pub phase: Phase,
    /// True between the start tap and the SRS queue arriving.
    pub is_loading: bool,
    /// 1-based position and total, for "7 / 22"-style counters.
    pub item_number: u32,
    pub total_items: u32,
    /// Key badge ("in E♭"), fixed per session.
    pub key_name: String,
    /// Present at reveal and during compare — sound before sign.
    pub answer: Option<AnswerView>,
    pub compare: Option<CompareView>,
    pub recap: Option<RecapView>,
    pub is_playing: bool,
    pub pause: PauseState,
    pub error: Option<String>,
}

/// Side effects the core requests from shells.
#[effect(facet_typegen)]
pub enum Effect {
    Render(RenderOperation),
    PlayScore(PlayScoreOperation),
    Storage(StorageOperation),
}

impl App for Changes {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
    ) -> Command<Self::Effect, Self::Event> {
        match event {
            Event::StartSession {
                seed,
                now_ms,
                max_items,
            } if matches!(model.phase, Phase::Pre | Phase::Recap) && !model.awaiting_load => {
                model.seed = seed;
                model.now_ms = now_ms;
                model.max_items = (max_items as usize).clamp(1, MAX_ITEMS_CEILING);
                model.rng = SplitMix64::new(seed ^ 0x5EED);
                model.awaiting_load = true;
                model.error = None;
                model.pause = PauseState::None;
                render().and(load_reviews())
            }
            Event::ReviewsLoaded(output) if model.awaiting_load => {
                model.awaiting_load = false;
                let states: Vec<crate::srs::ReviewState> = match output {
                    StorageOutput::Reviews(states) => states,
                    StorageOutput::Failed { message } => {
                        // Degrade gracefully: practice is never blocked by a
                        // broken DB — all-new queue, error surfaced.
                        model.error = Some(message);
                        Vec::new()
                    }
                    StorageOutput::Ack => Vec::new(),
                };
                model.review_states = states.iter().map(|s| (s.skill.key(), s.clone())).collect();
                let queue = build_queue(&states, &RUNG.skill_pool(), model.max_items, model.now_ms);
                model.items = plan_session(&queue, model.seed);
                model.index = 0;
                model.results.clear();
                if model.items.is_empty() {
                    model.error = Some("nothing to practise — queue was empty".into());
                    model.phase = Phase::Pre;
                    return render();
                }
                start_context(model)
            }
            Event::ReviewSaved(output) => {
                if let StorageOutput::Failed { message } = output {
                    model.error = Some(message);
                    return render();
                }
                Command::done()
            }
            Event::TapReveal if model.phase == Phase::Gap && model.pause == PauseState::None => {
                model.phase = Phase::Reveal;
                play_current(model, reveal_score)
            }
            Event::GradeGotIt
                if model.phase == Phase::Reveal && model.pause == PauseState::None =>
            {
                model.results.push(true);
                let persist = self.record_grade(model, Grade::Got);
                advance(model).and(persist)
            }
            Event::GradeMissedIt
                if model.phase == Phase::Reveal && model.pause == PauseState::None =>
            {
                model.results.push(false);
                let persist = self.record_grade(model, Grade::Missed);
                model.phase = Phase::Compare;
                model.compare_on_twin = false;
                play_compare_side(model).and(persist)
            }
            Event::ExitCompare if model.phase == Phase::Compare => advance(model),
            Event::TapPause
                if model.pause == PauseState::None
                    && !matches!(model.phase, Phase::Pre | Phase::Recap) =>
            {
                // Shell stops audio on seeing the paused ViewModel.
                model.pause = PauseState::User;
                model.is_playing = false;
                render()
            }
            Event::TapResume if model.pause != PauseState::None => {
                model.pause = PauseState::None;
                match model.phase {
                    Phase::Context => play_current(model, context_score),
                    Phase::Question => play_current(model, question_score),
                    Phase::Reveal => play_current(model, reveal_score),
                    Phase::Compare => play_compare_side(model),
                    // Gap/Pre/Recap hold no audio — just unfreeze.
                    _ => render(),
                }
            }
            Event::PlaybackFinished(output) => {
                model.is_playing = false;
                if let PlayScoreOutput::Failed { message } = output {
                    model.error = Some(message);
                    return render();
                }
                match model.phase {
                    // Context → Question is playback chaining, not user
                    // pacing — the item's audio continues seamlessly.
                    Phase::Context if model.pause == PauseState::None => {
                        model.phase = Phase::Question;
                        play_current(model, question_score)
                    }
                    Phase::Question if model.pause == PauseState::None => {
                        model.phase = Phase::Gap;
                        render()
                    }
                    // The compare alternation is continuous until the exit
                    // tap: each side finishing starts the other.
                    Phase::Compare if model.pause == PauseState::None => {
                        model.compare_on_twin = !model.compare_on_twin;
                        play_compare_side(model)
                    }
                    _ => render(),
                }
            }
            Event::AudioInterrupted | Event::HeadphonesUnplugged => {
                model.pause = PauseState::Interrupted;
                model.is_playing = false;
                render()
            }
            // Everything else is a tap that doesn't belong to the current
            // phase — ignored, no state change.
            _ => Command::done(),
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let answer = model
            .current()
            .filter(|_| matches!(model.phase, Phase::Reveal | Phase::Compare));
        ViewModel {
            phase: model.phase,
            is_loading: model.awaiting_load,
            item_number: (model.index + 1).min(model.items.len()) as u32,
            total_items: model.items.len() as u32,
            key_name: model
                .current()
                .map(|i| i.key.tonic_name().to_string())
                .unwrap_or_default(),
            answer: answer.map(|item| AnswerView {
                label: item.key.label_of(item.degree).to_string(),
                resolution: resolution_text(item),
            }),
            compare: (model.phase == Phase::Compare)
                .then(|| model.current())
                .flatten()
                .map(|item| CompareView {
                    missed: item.key.label_of(item.degree).to_string(),
                    twin: item.key.label_of(item.confusion_twin()).to_string(),
                    playing_twin: model.compare_on_twin,
                }),
            recap: (model.phase == Phase::Recap).then(|| {
                let got = model.results.iter().filter(|&&g| g).count() as u32;
                RecapView {
                    got,
                    missed: model.results.len() as u32 - got,
                }
            }),
            is_playing: model.is_playing,
            pause: model.pause,
            error: model.error.clone(),
        }
    }
}

impl Changes {
    /// Grade the current item into the SRS: update the in-memory state and
    /// emit the write (fire-and-forget for UI flow; a Failed output
    /// surfaces via `ReviewSaved`).
    fn record_grade(&self, model: &mut Model, grade: Grade) -> Command<Effect, Event> {
        let Some(item) = model.current() else {
            return Command::done();
        };
        let skill = SkillId {
            mode: item.key.mode,
            degree: item.degree,
        };
        let prior = model.review_states.get(&skill.key()).cloned();
        let state = self
            .scheduler
            .review(prior.as_ref(), skill, grade, model.now_ms);
        model.review_states.insert(skill.key(), state.clone());
        let log = ReviewLog {
            // Client-minted, deterministic given (now, seed) — canonical id.
            id: Ulid::from_parts(
                model.now_ms as u64,
                (model.rng.next_u64() as u128) << 64 | model.rng.next_u64() as u128,
            )
            .to_string(),
            skill,
            grade,
            reviewed_at_ms: model.now_ms,
        };
        save_review(state, log)
    }
}

fn resolution_text(item: Item) -> String {
    crate::theory::resolution_path(item.key, item.degree)
        .iter()
        .map(|d| item.key.label_of(*d))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn start_context(model: &mut Model) -> Command<Effect, Event> {
    model.phase = Phase::Context;
    play_current(model, context_score)
}

fn play_current(
    model: &mut Model,
    score_for: fn(Item) -> crate::audio::Score,
) -> Command<Effect, Event> {
    match model.current() {
        Some(item) => {
            model.is_playing = true;
            render().and(play_score(score_for(item)))
        }
        None => render(),
    }
}

fn play_compare_side(model: &mut Model) -> Command<Effect, Event> {
    match model.current() {
        Some(item) => {
            let degree = if model.compare_on_twin {
                item.confusion_twin()
            } else {
                item.degree
            };
            model.is_playing = true;
            render().and(play_score(compare_side_score(item, degree)))
        }
        None => render(),
    }
}

fn advance(model: &mut Model) -> Command<Effect, Event> {
    model.index += 1;
    if model.index >= model.items.len() {
        model.phase = Phase::Recap;
        model.is_playing = false;
        render()
    } else {
        start_context(model)
    }
}

#[cfg(test)]
mod tests {
    use crate::srs::ReviewState;

    use super::*;

    const SEED: u64 = 7;
    const NOW: i64 = 1_800_000_000_000;
    const DAY_MS: i64 = 86_400_000;

    fn plays(cmd: &mut Command<Effect, Event>) -> Vec<PlayScoreOperation> {
        cmd.effects()
            .filter_map(|e| match e {
                Effect::PlayScore(req) => Some(req.operation.clone()),
                _ => None,
            })
            .collect()
    }

    fn storage_ops(cmd: &mut Command<Effect, Event>) -> Vec<StorageOperation> {
        cmd.effects()
            .filter_map(|e| match e {
                Effect::Storage(req) => Some(req.operation.clone()),
                _ => None,
            })
            .collect()
    }

    const MAX_ITEMS: u32 = 12;

    fn start_event() -> Event {
        Event::StartSession {
            seed: SEED,
            now_ms: NOW,
            max_items: MAX_ITEMS,
        }
    }

    fn start_with(states: Vec<ReviewState>) -> (Changes, Model) {
        let app = Changes::default();
        let mut model = Model::default();
        let mut cmd = app.update(start_event(), &mut model);
        assert!(matches!(
            storage_ops(&mut cmd)[..],
            [StorageOperation::LoadReviews]
        ));
        let _ = app.update(
            Event::ReviewsLoaded(StorageOutput::Reviews(states)),
            &mut model,
        );
        (app, model)
    }

    fn to_reveal(app: &Changes, model: &mut Model) {
        // Context finishes → Question plays → finishes → Gap.
        let _ = app.update(Event::PlaybackFinished(PlayScoreOutput::Finished), model);
        let _ = app.update(Event::PlaybackFinished(PlayScoreOutput::Finished), model);
        let _ = app.update(Event::TapReveal, model);
    }

    #[test]
    fn start_loads_reviews_before_any_audio() {
        let app = Changes::default();
        let mut model = Model::default();
        let mut cmd = app.update(start_event(), &mut model);
        assert!(plays(&mut cmd).is_empty(), "no audio before the queue");
        assert!(app.view(&model).is_loading);
        assert_eq!(app.view(&model).phase, Phase::Pre);
    }

    #[test]
    fn loaded_reviews_start_the_session_from_the_queue() {
        let (app, model) = start_with(Vec::new());
        let view = app.view(&model);
        assert_eq!(view.phase, Phase::Context);
        assert!(!view.is_loading);
        // Fresh user: the whole rung pool, in some shuffled order.
        assert_eq!(view.total_items, RUNG.skill_pool().len() as u32);
    }

    #[test]
    fn context_chains_into_question_then_waits_in_the_gap() {
        let (app, mut model) = start_with(Vec::new());
        let item = model.current().expect("item");

        let mut cmd = app.update(
            Event::PlaybackFinished(PlayScoreOutput::Finished),
            &mut model,
        );
        assert_eq!(app.view(&model).phase, Phase::Question);
        assert_eq!(
            plays(&mut cmd)[0].score,
            crate::session::question_score(item)
        );

        let _ = app.update(
            Event::PlaybackFinished(PlayScoreOutput::Finished),
            &mut model,
        );
        assert_eq!(app.view(&model).phase, Phase::Gap);
        assert!(!app.view(&model).is_playing);
    }

    #[test]
    fn max_items_caps_the_session() {
        let app = Changes::default();
        let mut model = Model::default();
        let _ = app.update(
            Event::StartSession {
                seed: SEED,
                now_ms: NOW,
                max_items: 3,
            },
            &mut model,
        );
        let _ = app.update(
            Event::ReviewsLoaded(StorageOutput::Reviews(Vec::new())),
            &mut model,
        );
        assert_eq!(app.view(&model).total_items, 3);
    }

    #[test]
    fn user_pause_differs_from_interruption_and_resume_replays_the_phase() {
        let (app, mut model) = start_with(Vec::new());
        let _ = app.update(Event::TapPause, &mut model);
        assert_eq!(app.view(&model).pause, PauseState::User);

        let mut cmd = app.update(Event::TapResume, &mut model);
        let item = model.current().expect("item");
        assert_eq!(plays(&mut cmd)[0].score, context_score(item));
        assert_eq!(app.view(&model).pause, PauseState::None);

        let _ = app.update(Event::AudioInterrupted, &mut model);
        assert_eq!(app.view(&model).pause, PauseState::Interrupted);
    }

    #[test]
    fn overdue_skills_are_in_the_session() {
        let scheduler = FsrsScheduler::default();
        let skill = SkillId {
            mode: crate::theory::Mode::Major,
            degree: crate::theory::Degree::new(9),
        };
        let mut overdue = scheduler.review(None, skill, Grade::Got, NOW - 40 * DAY_MS);
        overdue.due_at_ms = NOW - DAY_MS;
        let (app, model) = start_with(vec![overdue]);
        assert!(model.items.iter().any(|i| i.degree.semitones() == 9));
        assert_eq!(app.view(&model).phase, Phase::Context);
    }

    #[test]
    fn a_grade_updates_srs_state_and_persists_state_plus_log() {
        let (app, mut model) = start_with(Vec::new());
        to_reveal(&app, &mut model);
        let item = model.current().expect("item");

        let mut cmd = app.update(Event::GradeGotIt, &mut model);

        let ops = storage_ops(&mut cmd);
        let [StorageOperation::SaveReview { state, log }] = &ops[..] else {
            panic!("expected one SaveReview, got {ops:?}");
        };
        assert_eq!(state.skill.degree, item.degree);
        assert_eq!(state.last_reviewed_at_ms, NOW);
        assert!(state.due_at_ms > NOW, "scheduled into the future");
        assert_eq!(log.grade, Grade::Got);
        assert_eq!(log.reviewed_at_ms, NOW);
        assert!(!log.id.is_empty());
        assert!(
            model.review_states.contains_key(&state.skill.key()),
            "model mirrors the write"
        );
    }

    #[test]
    fn a_miss_persists_and_enters_compare() {
        let (app, mut model) = start_with(Vec::new());
        to_reveal(&app, &mut model);

        let mut cmd = app.update(Event::GradeMissedIt, &mut model);

        // effects() drains — collect once, then partition.
        let effects: Vec<Effect> = cmd.effects().collect();
        let saves = effects
            .iter()
            .filter(|e| matches!(e, Effect::Storage(_)))
            .count();
        let plays = effects
            .iter()
            .filter(|e| matches!(e, Effect::PlayScore(_)))
            .count();
        assert_eq!(saves, 1);
        assert_eq!(plays, 1, "compare side plays");
        assert_eq!(app.view(&model).phase, Phase::Compare);
    }

    #[test]
    fn review_log_ids_are_unique_and_deterministic() {
        let run = || {
            let (app, mut model) = start_with(Vec::new());
            let mut ids = Vec::new();
            for _ in 0..3 {
                to_reveal(&app, &mut model);
                let mut cmd = app.update(Event::GradeGotIt, &mut model);
                if let [StorageOperation::SaveReview { log, .. }] = &storage_ops(&mut cmd)[..] {
                    ids.push(log.id.clone());
                }
            }
            ids
        };
        let a = run();
        let b = run();
        assert_eq!(a.len(), 3);
        assert_eq!(a, b, "same seed + clock → same ids (replayable)");
        let unique: std::collections::HashSet<_> = a.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn storage_load_failure_degrades_to_a_fresh_queue_with_surfaced_error() {
        let app = Changes::default();
        let mut model = Model::default();
        let _ = app.update(start_event(), &mut model);
        let _ = app.update(
            Event::ReviewsLoaded(StorageOutput::Failed {
                message: "disk full".into(),
            }),
            &mut model,
        );
        let view = app.view(&model);
        assert_eq!(view.phase, Phase::Context, "practice is never blocked");
        assert_eq!(view.error, Some("disk full".into()));
    }

    #[test]
    fn save_failure_surfaces_but_does_not_interrupt_the_session() {
        let (app, mut model) = start_with(Vec::new());
        to_reveal(&app, &mut model);
        let _ = app.update(Event::GradeGotIt, &mut model);

        let _ = app.update(
            Event::ReviewSaved(StorageOutput::Failed {
                message: "write failed".into(),
            }),
            &mut model,
        );

        let view = app.view(&model);
        assert_eq!(view.error, Some("write failed".into()));
        assert_eq!(view.phase, Phase::Context, "session continues");
    }

    #[test]
    fn full_session_still_reaches_an_honest_recap() {
        let (app, mut model) = start_with(Vec::new());
        let total = model.items.len();
        for i in 0..total {
            to_reveal(&app, &mut model);
            if i % 4 == 3 {
                let _ = app.update(Event::GradeMissedIt, &mut model);
                let _ = app.update(Event::ExitCompare, &mut model);
            } else {
                let _ = app.update(Event::GradeGotIt, &mut model);
            }
        }
        let view = app.view(&model);
        assert_eq!(view.phase, Phase::Recap);
        let recap = view.recap.expect("recap");
        assert_eq!((recap.got + recap.missed) as usize, total);
    }

    #[test]
    fn second_session_sees_the_first_sessions_grades() {
        let (app, mut model) = start_with(Vec::new());
        let total = model.items.len();
        for _ in 0..total {
            to_reveal(&app, &mut model);
            let _ = app.update(Event::GradeGotIt, &mut model);
        }
        assert_eq!(app.view(&model).phase, Phase::Recap);

        // Next morning: the shell would hand back what it persisted; here
        // the model's mirror stands in for it.
        let states: Vec<ReviewState> = model.review_states.values().cloned().collect();
        let tomorrow = NOW + DAY_MS;
        let (app2, mut model2) = (Changes::default(), Model::default());
        let _ = app2.update(
            Event::StartSession {
                seed: SEED + 1,
                now_ms: tomorrow,
                max_items: MAX_ITEMS,
            },
            &mut model2,
        );
        let _ = app2.update(
            Event::ReviewsLoaded(StorageOutput::Reviews(states)),
            &mut model2,
        );
        // Everything was graded Got with multi-day intervals — nothing due
        // tomorrow, nothing unseen: the queue is only the soonest-due tail.
        assert!(
            !model2.items.is_empty(),
            "upcoming skills fill an otherwise-empty session"
        );
    }

    #[test]
    fn interruption_pause_resume_still_replays_the_phase() {
        let (app, mut model) = start_with(Vec::new());
        let _ = app.update(Event::AudioInterrupted, &mut model);
        assert_eq!(app.view(&model).pause, PauseState::Interrupted);
        let mut cmd = app.update(Event::TapResume, &mut model);
        assert_eq!(plays(&mut cmd).len(), 1);
    }

    #[test]
    fn interruption_mid_context_does_not_chain_into_question() {
        let (app, mut model) = start_with(Vec::new());
        let _ = app.update(Event::AudioInterrupted, &mut model);

        // The stopped score resolving must not start the question.
        let mut cmd = app.update(
            Event::PlaybackFinished(PlayScoreOutput::Finished),
            &mut model,
        );
        assert!(plays(&mut cmd).is_empty());
        assert_eq!(app.view(&model).phase, Phase::Context);
    }

    // The bridge is positional bincode (non-self-describing): every
    // bridge-crossing type gets a round-trip test via the shared helper so
    // a silent wire break fails here, not as a no-op in the shell
    // (intrada #846).
    #[test]
    fn event_bincode_round_trips() {
        let skill = SkillId {
            mode: crate::theory::Mode::Major,
            degree: crate::theory::Degree::new(3),
        };
        let state = ReviewState {
            skill,
            stability: 2.5,
            difficulty: 5.1,
            last_reviewed_at_ms: NOW,
            due_at_ms: NOW + DAY_MS,
        };
        for event in [
            Event::StartSession {
                seed: 42,
                now_ms: NOW,
                max_items: 12,
            },
            Event::TapPause,
            Event::ReviewsLoaded(StorageOutput::Reviews(vec![state.clone()])),
            Event::ReviewsLoaded(StorageOutput::Failed {
                message: "io".into(),
            }),
            Event::ReviewSaved(StorageOutput::Ack),
            Event::TapReveal,
            Event::GradeGotIt,
            Event::GradeMissedIt,
            Event::ExitCompare,
            Event::TapResume,
            Event::PlaybackFinished(PlayScoreOutput::Finished),
            Event::AudioInterrupted,
            Event::HeadphonesUnplugged,
        ] {
            crate::test_support::assert_bincode_round_trip(&event);
        }
    }

    #[test]
    fn storage_operation_bincode_round_trips() {
        let skill = SkillId {
            mode: crate::theory::Mode::Minor,
            degree: crate::theory::Degree::new(10),
        };
        let state = ReviewState {
            skill,
            stability: 1.0,
            difficulty: 4.0,
            last_reviewed_at_ms: NOW,
            due_at_ms: NOW + DAY_MS,
        };
        crate::test_support::assert_bincode_round_trip(&StorageOperation::LoadReviews);
        crate::test_support::assert_bincode_round_trip(&StorageOperation::SaveReview {
            state: state.clone(),
            log: ReviewLog {
                id: "01J0000000000000000000000".into(),
                skill,
                grade: Grade::Missed,
                reviewed_at_ms: NOW,
            },
        });
        crate::test_support::assert_bincode_round_trip(&StorageOutput::Reviews(vec![state]));
    }

    #[test]
    fn view_model_bincode_round_trips() {
        crate::test_support::assert_bincode_round_trip(&ViewModel {
            phase: Phase::Compare,
            is_loading: false,
            item_number: 3,
            total_items: 12,
            key_name: "E♭".into(),
            answer: Some(AnswerView {
                label: "♭3".into(),
                resolution: "♭3 · 2 · 1".into(),
            }),
            compare: Some(CompareView {
                missed: "♭3".into(),
                twin: "3".into(),
                playing_twin: true,
            }),
            recap: Some(RecapView { got: 9, missed: 3 }),
            is_playing: false,
            pause: PauseState::Interrupted,
            error: None,
        });
    }
}
