use crux_core::macros::effect;
use crux_core::render::{render, RenderOperation};
use crux_core::{App, Command};
use serde::{Deserialize, Serialize};

use crate::audio::{play_score, PlayScoreOperation, PlayScoreOutput};
use crate::session::{
    compare_side_score, generate_session, listening_score, reveal_score, Item, Rung,
};

/// Session length until the SRS queue drives it (M2b) and the pre-session
/// duration pills wire up (M3).
const ITEMS_PER_SESSION: usize = 12;
/// Starting rung until placement (M5) and per-user progress (M2b) exist.
const RUNG: Rung = Rung::DiatonicMajor;

/// Root Crux application: the Pocket Session state machine
/// (`pre → [listening → gap → reveal → (compare)]* → recap`), manually
/// paced — every `→` that involves the user is a deliberate tap (mvp-plan
/// decision 9). The design spec is `design/README.md` §Interactions.
#[derive(Default)]
pub struct Changes;

/// All events the application can process. Bridge-crossing: serialized over
/// positional bincode to the shell — field order is the wire format.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Event {
    /// Pre-session play tap. The seed is shell-provided entropy (time and
    /// randomness arrive via events — core stays deterministic).
    StartSession {
        seed: u64,
    },
    /// Gap → reveal (the open-ended thinking gap ends on this tap).
    TapReveal,
    /// Self-grade taps; each doubles as "next".
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
    /// Cadence + question note playing (level bars only — no pitch visuals).
    Listening,
    /// Open-ended thinking gap; silence on purpose.
    Gap,
    Reveal,
    /// Banacos loop: the missed color alternating with its confusable twin.
    Compare,
    Recap,
}

#[derive(Default, Debug)]
pub struct Model {
    phase: Phase,
    items: Vec<Item>,
    index: usize,
    results: Vec<bool>,
    compare_on_twin: bool,
    is_playing: bool,
    paused: bool,
    error: Option<String>,
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
    pub paused: bool,
    pub error: Option<String>,
}

/// Side effects the core requests from shells.
#[effect(facet_typegen)]
pub enum Effect {
    Render(RenderOperation),
    PlayScore(PlayScoreOperation),
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
            Event::StartSession { seed } if matches!(model.phase, Phase::Pre | Phase::Recap) => {
                model.items = generate_session(RUNG, ITEMS_PER_SESSION, seed);
                model.index = 0;
                model.results.clear();
                model.error = None;
                model.paused = false;
                start_listening(model)
            }
            Event::TapReveal if model.phase == Phase::Gap && !model.paused => {
                model.phase = Phase::Reveal;
                play_current(model, reveal_score)
            }
            Event::GradeGotIt if model.phase == Phase::Reveal && !model.paused => {
                model.results.push(true);
                advance(model)
            }
            Event::GradeMissedIt if model.phase == Phase::Reveal && !model.paused => {
                model.results.push(false);
                model.phase = Phase::Compare;
                model.compare_on_twin = false;
                play_compare_side(model)
            }
            Event::ExitCompare if model.phase == Phase::Compare => advance(model),
            Event::TapResume if model.paused => {
                model.paused = false;
                match model.phase {
                    Phase::Listening => play_current(model, listening_score),
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
                    Phase::Listening if !model.paused => {
                        model.phase = Phase::Gap;
                        render()
                    }
                    // The compare alternation is continuous until the exit
                    // tap: each side finishing starts the other.
                    Phase::Compare if !model.paused => {
                        model.compare_on_twin = !model.compare_on_twin;
                        play_compare_side(model)
                    }
                    _ => render(),
                }
            }
            Event::AudioInterrupted | Event::HeadphonesUnplugged => {
                model.paused = true;
                model.is_playing = false;
                render()
            }
            // Everything else is a tap that doesn't belong to the current
            // phase — ignored, no state change.
            _ => Command::done(),
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let key = model.current().map(|i| i.key);
        let answer = model
            .current()
            .filter(|_| matches!(model.phase, Phase::Reveal | Phase::Compare));
        ViewModel {
            phase: model.phase,
            item_number: (model.index + 1).min(model.items.len()) as u32,
            total_items: model.items.len() as u32,
            key_name: key.map(|k| k.tonic_name().to_string()).unwrap_or_default(),
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
            paused: model.paused,
            error: model.error.clone(),
        }
    }
}

fn resolution_text(item: Item) -> String {
    crate::theory::resolution_path(item.key, item.degree)
        .iter()
        .map(|d| item.key.label_of(*d))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn start_listening(model: &mut Model) -> Command<Effect, Event> {
    model.phase = Phase::Listening;
    play_current(model, listening_score)
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
        start_listening(model)
    }
}

#[cfg(test)]
mod tests {
    use crate::session::{reveal_score as reveal_for, TIMING};
    use crate::theory::Degree;

    use super::*;

    const SEED: u64 = 7;

    fn start() -> (Changes, Model) {
        let app = Changes;
        let mut model = Model::default();
        let _ = app.update(Event::StartSession { seed: SEED }, &mut model);
        (app, model)
    }

    fn finish_playback(app: &Changes, model: &mut Model) {
        let _ = app.update(Event::PlaybackFinished(PlayScoreOutput::Finished), model);
    }

    fn plays(cmd: &mut Command<Effect, Event>) -> Vec<PlayScoreOperation> {
        cmd.effects()
            .filter_map(|e| match e {
                Effect::PlayScore(req) => Some(req.operation.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn start_plays_the_listening_score_of_a_seeded_session() {
        let app = Changes;
        let mut model = Model::default();
        let mut cmd = app.update(Event::StartSession { seed: SEED }, &mut model);

        let expected = generate_session(RUNG, ITEMS_PER_SESSION, SEED);
        assert_eq!(
            plays(&mut cmd),
            vec![PlayScoreOperation {
                score: listening_score(expected[0])
            }]
        );
        let view = app.view(&model);
        assert_eq!(view.phase, Phase::Listening);
        assert_eq!(view.total_items, ITEMS_PER_SESSION as u32);
        assert!(!view.key_name.is_empty());
    }

    #[test]
    fn listening_finish_opens_the_gap_and_waits() {
        let (app, mut model) = start();
        finish_playback(&app, &mut model);
        let view = app.view(&model);
        assert_eq!(view.phase, Phase::Gap);
        assert!(!view.is_playing);
        assert_eq!(view.answer, None, "silence on purpose — no hints");
    }

    #[test]
    fn reveal_waits_for_the_tap_and_plays_the_resolution() {
        let (app, mut model) = start();
        finish_playback(&app, &mut model);

        let mut cmd = app.update(Event::TapReveal, &mut model);

        let item = model.current().expect("current item");
        assert_eq!(plays(&mut cmd)[0].score, reveal_for(item));
        let view = app.view(&model);
        assert_eq!(view.phase, Phase::Reveal);
        let answer = view.answer.expect("answer shown at reveal");
        assert_eq!(answer.label, item.key.label_of(item.degree));
        assert!(answer.resolution.ends_with('1'));
    }

    #[test]
    fn got_it_advances_to_the_next_items_listening() {
        let (app, mut model) = start();
        finish_playback(&app, &mut model);
        let _ = app.update(Event::TapReveal, &mut model);

        let mut cmd = app.update(Event::GradeGotIt, &mut model);

        assert_eq!(app.view(&model).phase, Phase::Listening);
        assert_eq!(app.view(&model).item_number, 2);
        assert_eq!(plays(&mut cmd).len(), 1);
    }

    #[test]
    fn missed_it_enters_the_compare_loop_which_alternates_until_exit() {
        let (app, mut model) = start();
        finish_playback(&app, &mut model);
        let _ = app.update(Event::TapReveal, &mut model);

        let mut cmd = app.update(Event::GradeMissedIt, &mut model);
        let item = model.current().expect("item");
        assert_eq!(
            plays(&mut cmd)[0].score,
            compare_side_score(item, item.degree),
            "compare starts on the missed color"
        );
        assert!(!app.view(&model).compare.expect("compare view").playing_twin);

        // Each side finishing starts the other — the Banacos alternation.
        let mut cmd = app.update(
            Event::PlaybackFinished(PlayScoreOutput::Finished),
            &mut model,
        );
        assert_eq!(
            plays(&mut cmd)[0].score,
            compare_side_score(item, item.confusion_twin())
        );
        assert!(app.view(&model).compare.expect("compare view").playing_twin);

        // Exit resumes the session on the next item.
        let _ = app.update(Event::ExitCompare, &mut model);
        assert_eq!(app.view(&model).phase, Phase::Listening);
        assert_eq!(app.view(&model).item_number, 2);
    }

    #[test]
    fn a_full_session_reaches_the_recap_with_honest_counts() {
        let (app, mut model) = start();
        for i in 0..ITEMS_PER_SESSION {
            finish_playback(&app, &mut model);
            let _ = app.update(Event::TapReveal, &mut model);
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
        assert_eq!(recap.got, 9);
        assert_eq!(recap.missed, 3);
        // And a new session can start from the recap.
        let _ = app.update(Event::StartSession { seed: 99 }, &mut model);
        assert_eq!(app.view(&model).phase, Phase::Listening);
    }

    #[test]
    fn interruption_pauses_and_resume_replays_the_current_phase() {
        let (app, mut model) = start();
        let _ = app.update(Event::AudioInterrupted, &mut model);
        assert!(app.view(&model).paused);

        // A grade tap while paused is ignored.
        let _ = app.update(Event::GradeGotIt, &mut model);
        assert_eq!(app.view(&model).item_number, 1);

        let mut cmd = app.update(Event::TapResume, &mut model);
        let item = model.current().expect("item");
        assert_eq!(plays(&mut cmd)[0].score, listening_score(item));
        assert!(!app.view(&model).paused);
    }

    #[test]
    fn pause_stops_the_compare_alternation() {
        let (app, mut model) = start();
        finish_playback(&app, &mut model);
        let _ = app.update(Event::TapReveal, &mut model);
        let _ = app.update(Event::GradeMissedIt, &mut model);
        let _ = app.update(Event::HeadphonesUnplugged, &mut model);

        // The in-flight side resolving must NOT start the next side.
        let mut cmd = app.update(
            Event::PlaybackFinished(PlayScoreOutput::Finished),
            &mut model,
        );
        assert!(plays(&mut cmd).is_empty());
        assert!(app.view(&model).paused);
    }

    #[test]
    fn out_of_phase_taps_are_ignored() {
        let app = Changes;
        let mut model = Model::default();
        for event in [Event::TapReveal, Event::GradeGotIt, Event::ExitCompare] {
            let _ = app.update(event, &mut model);
            assert_eq!(app.view(&model).phase, Phase::Pre);
        }
    }

    #[test]
    fn playback_failure_is_surfaced_never_swallowed() {
        let (app, mut model) = start();
        let _ = app.update(
            Event::PlaybackFinished(PlayScoreOutput::Failed {
                message: "no soundfont".into(),
            }),
            &mut model,
        );
        assert_eq!(app.view(&model).error, Some("no soundfont".to_string()));
    }

    #[test]
    fn reveal_home_note_gets_the_long_duration() {
        let (_, model) = start();
        let item = model.current().expect("item");
        if item.degree != Degree::TONIC {
            let score = reveal_for(item);
            assert_eq!(
                score.notes.last().map(|n| n.duration_beats),
                Some(TIMING.reveal_home_beats)
            );
        }
    }

    // The bridge is positional bincode (non-self-describing): every
    // bridge-crossing type gets a round-trip test via the shared helper so
    // a silent wire break fails here, not as a no-op in the shell
    // (intrada #846).
    #[test]
    fn event_bincode_round_trips() {
        for event in [
            Event::StartSession { seed: 42 },
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
    fn view_model_bincode_round_trips() {
        crate::test_support::assert_bincode_round_trip(&ViewModel {
            phase: Phase::Compare,
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
            paused: false,
            error: None,
        });
    }

    #[test]
    fn effect_operation_bincode_round_trips() {
        let (_, model) = start();
        crate::test_support::assert_bincode_round_trip(&crux_core::render::RenderOperation);
        crate::test_support::assert_bincode_round_trip(&PlayScoreOperation {
            score: listening_score(model.current().expect("item")),
        });
    }
}
