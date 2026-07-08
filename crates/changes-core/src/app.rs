use crux_core::macros::effect;
use crux_core::render::{render, RenderOperation};
use crux_core::{App, Command};
use serde::{Deserialize, Serialize};

use crate::audio::{play_score, PlayScoreOperation, PlayScoreOutput};
use crate::spike::{cadence_score, question_score, reveal_score, SPIKE_CHORDS};

/// Root Crux application. M1 audio spike: a tap-paced context → question →
/// reveal loop over hardcoded E♭ chords, exercising the `PlayScore` pipeline
/// end-to-end. M2 replaces the hardcoded content with the real session
/// engine; the event/effect shape is the durable part.
#[derive(Default)]
pub struct Changes;

/// All events the application can process. Bridge-crossing: serialized over
/// positional bincode to the shell — field order is the wire format.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Event {
    /// The one deliberate tap: starts the next phase (manual pacing,
    /// mvp-plan decision 9), or resumes after a pause by replaying the
    /// current phase — no item is ever lost to an interruption.
    TapNext,
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
    Idle,
    Context,
    Question,
    Reveal,
}

#[derive(Default, Debug)]
pub struct Model {
    phase: Phase,
    item: usize,
    is_playing: bool,
    paused: bool,
    error: Option<String>,
}

/// Bridge-crossing: what shells render.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct ViewModel {
    pub phase: Phase,
    /// 1-based, only meaningful outside `Idle`.
    pub item_number: u32,
    /// Shown at `Reveal` only — sound before sign.
    pub answer: Option<String>,
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
            Event::TapNext => {
                if !model.paused {
                    model.phase = match model.phase {
                        Phase::Idle => Phase::Context,
                        Phase::Context => Phase::Question,
                        Phase::Question => Phase::Reveal,
                        Phase::Reveal => {
                            model.item += 1;
                            Phase::Context
                        }
                    };
                }
                model.paused = false;
                model.is_playing = true;
                let chord = SPIKE_CHORDS[model.item % SPIKE_CHORDS.len()];
                let score = match model.phase {
                    // Unreachable in practice (the match above always leaves
                    // Idle); playing the context is the safe fallback.
                    Phase::Idle | Phase::Context => cadence_score(),
                    Phase::Question => question_score(chord),
                    Phase::Reveal => reveal_score(chord),
                };
                render().and(play_score(score))
            }
            Event::PlaybackFinished(output) => {
                model.is_playing = false;
                if let PlayScoreOutput::Failed { message } = output {
                    model.error = Some(message);
                }
                render()
            }
            Event::AudioInterrupted | Event::HeadphonesUnplugged => {
                model.paused = true;
                model.is_playing = false;
                render()
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let chord = SPIKE_CHORDS[model.item % SPIKE_CHORDS.len()];
        ViewModel {
            phase: model.phase,
            item_number: model.item as u32 + 1,
            answer: (model.phase == Phase::Reveal).then(|| chord.label().to_string()),
            is_playing: model.is_playing,
            paused: model.paused,
            error: model.error.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::spike::{SpikeChord, TIMING};

    use super::*;

    fn play_effects(cmd: &mut Command<Effect, Event>) -> Vec<PlayScoreOperation> {
        cmd.effects()
            .filter_map(|e| match e {
                Effect::PlayScore(req) => Some(req.operation.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn first_tap_plays_the_cadence() {
        let app = Changes;
        let mut model = Model::default();

        let mut cmd = app.update(Event::TapNext, &mut model);

        let plays = play_effects(&mut cmd);
        assert_eq!(plays.len(), 1);
        assert_eq!(plays[0].score, cadence_score());
        let view = app.view(&model);
        assert_eq!(view.phase, Phase::Context);
        assert!(view.is_playing);
        assert_eq!(view.answer, None);
    }

    #[test]
    fn tap_sequence_walks_context_question_reveal_then_next_item() {
        let app = Changes;
        let mut model = Model::default();

        let scores: Vec<_> = (0..4)
            .map(|_| {
                let mut cmd = app.update(Event::TapNext, &mut model);
                play_effects(&mut cmd).remove(0).score
            })
            .collect();

        assert_eq!(scores[0], cadence_score());
        assert_eq!(scores[1], question_score(SpikeChord::EbMaj7));
        assert_eq!(scores[2], reveal_score(SpikeChord::EbMaj7));
        // Fourth tap: next item's context.
        assert_eq!(scores[3], cadence_score());
        assert_eq!(app.view(&model).item_number, 2);
    }

    #[test]
    fn answer_is_shown_at_reveal_only() {
        let app = Changes;
        let mut model = Model::default();

        for _ in 0..2 {
            let _ = app.update(Event::TapNext, &mut model);
            assert_eq!(app.view(&model).answer, None, "sound before sign");
        }
        let _ = app.update(Event::TapNext, &mut model);
        assert_eq!(app.view(&model).answer, Some("E♭maj7".to_string()));
    }

    #[test]
    fn interruption_pauses_and_the_next_tap_replays_the_same_phase() {
        let app = Changes;
        let mut model = Model::default();
        let _ = app.update(Event::TapNext, &mut model);
        let _ = app.update(Event::TapNext, &mut model); // Question, item 1

        let _ = app.update(Event::AudioInterrupted, &mut model);
        let view = app.view(&model);
        assert!(view.paused);
        assert!(!view.is_playing);

        // Resume tap replays the question — the item is not lost.
        let mut cmd = app.update(Event::TapNext, &mut model);
        let plays = play_effects(&mut cmd);
        assert_eq!(plays[0].score, question_score(SpikeChord::EbMaj7));
        assert_eq!(app.view(&model).phase, Phase::Question);
        assert!(!app.view(&model).paused);
    }

    #[test]
    fn headphones_unplugged_pauses_playback() {
        let app = Changes;
        let mut model = Model::default();
        let _ = app.update(Event::TapNext, &mut model);

        let _ = app.update(Event::HeadphonesUnplugged, &mut model);

        assert!(app.view(&model).paused);
    }

    #[test]
    fn playback_failure_is_surfaced_never_swallowed() {
        let app = Changes;
        let mut model = Model::default();
        let _ = app.update(Event::TapNext, &mut model);

        let _ = app.update(
            Event::PlaybackFinished(PlayScoreOutput::Failed {
                message: "no soundfont".into(),
            }),
            &mut model,
        );

        assert_eq!(app.view(&model).error, Some("no soundfont".to_string()));
    }

    #[test]
    fn cadence_timing_matches_the_prototype_spec() {
        let score = cadence_score();
        assert_eq!(score.tempo_bpm, TIMING.tempo_bpm);
        // Three rolled 4-note chords.
        assert_eq!(score.notes.len(), 12);
        let last_onset = score
            .notes
            .iter()
            .map(|n| n.onset_beats)
            .fold(f32::MIN, f32::max);
        assert!(last_onset < TIMING.cadence_chord_beats * 2.0 + TIMING.roll_beats * 4.0);
    }

    // The bridge is positional bincode (non-self-describing): every
    // bridge-crossing type gets a round-trip test via the shared helper so
    // a silent wire break fails here, not as a no-op in the shell
    // (intrada #846). Effect payloads round-trip their operation types —
    // the generated `*Ffi` enum derives no PartialEq/Debug.
    #[test]
    fn event_bincode_round_trip() {
        crate::test_support::assert_bincode_round_trip(&Event::TapNext);
        crate::test_support::assert_bincode_round_trip(&Event::PlaybackFinished(
            PlayScoreOutput::Failed {
                message: "engine".into(),
            },
        ));
        crate::test_support::assert_bincode_round_trip(&Event::AudioInterrupted);
        crate::test_support::assert_bincode_round_trip(&Event::HeadphonesUnplugged);
    }

    #[test]
    fn view_model_bincode_round_trip() {
        crate::test_support::assert_bincode_round_trip(&ViewModel {
            phase: Phase::Reveal,
            item_number: 3,
            answer: Some("E♭7".into()),
            is_playing: false,
            paused: true,
            error: None,
        });
    }

    #[test]
    fn effect_operation_bincode_round_trips() {
        crate::test_support::assert_bincode_round_trip(&RenderOperation);
        crate::test_support::assert_bincode_round_trip(&PlayScoreOperation {
            score: cadence_score(),
        });
        crate::test_support::assert_bincode_round_trip(&PlayScoreOutput::Finished);
    }
}
