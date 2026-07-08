//! Score builders: turn items into the abstract scores the shell realizes.
//! Context (cadence) and Question (the note) are separate scores so the UI
//! can show distinct states; each score is one sample-accurate schedule.

use crate::audio::{InstrumentRole, Score, ScoreNote};
use crate::theory::{resolution_path, two_five_one, Degree};

use super::item::Item;
use super::timing::TIMING;

/// Register anchor for question/reveal melody notes (around E♭4): above
/// the cadence voicings, comfortable singing range.
const MELODY_ANCHOR_MIDI: u8 = 63;

fn push_chord(notes: &mut Vec<ScoreNote>, midi: &[u8], onset: f32, duration: f32, velocity: u8) {
    for (i, &m) in midi.iter().enumerate() {
        let roll = TIMING.roll_beats * i as f32;
        notes.push(ScoreNote {
            midi: m,
            onset_beats: onset + roll,
            duration_beats: duration - roll,
            velocity,
            role: InstrumentRole::Piano,
        });
    }
}

fn push_note(notes: &mut Vec<ScoreNote>, midi: u8, onset: f32, duration: f32, velocity: u8) {
    notes.push(ScoreNote {
        midi,
        onset_beats: onset,
        duration_beats: duration,
        velocity,
        role: InstrumentRole::Piano,
    });
}

fn melody_midi(item: Item, degree: Degree) -> u8 {
    item.key
        .pitch_class_of(degree)
        .midi_near(MELODY_ANCHOR_MIDI)
}

/// Context: the cadence establishing the key, plus the pre-question breath
/// (as trailing silence — the phase's audio owns its own gap, so the
/// Question phase starts exactly when its note sounds).
pub fn context_score(item: Item) -> Score {
    let mut notes = Vec::new();
    let [ii, v, i] = two_five_one(item.key);
    push_chord(&mut notes, &ii, 0.0, TIMING.cadence_chord_beats, 76);
    push_chord(
        &mut notes,
        &v,
        TIMING.cadence_chord_beats,
        TIMING.cadence_chord_beats,
        76,
    );
    let resolve_at = TIMING.cadence_chord_beats * 2.0;
    push_chord(&mut notes, &i, resolve_at, TIMING.cadence_resolve_beats, 80);
    Score {
        tempo_bpm: TIMING.tempo_bpm,
        notes,
    }
}

/// Question: the item's note alone, played once.
pub fn question_score(item: Item) -> Score {
    let mut notes = Vec::new();
    push_note(
        &mut notes,
        melody_midi(item, item.degree),
        // The breath between cadence and question lives here as onset
        // offset, keeping it one sample-accurate schedule.
        TIMING.pre_question_rest_beats,
        TIMING.question_beats,
        88,
    );
    Score {
        tempo_bpm: TIMING.tempo_bpm,
        notes,
    }
}

/// The aural reveal: the degree walks its resolution path home
/// (mvp-plan decision 5 — the reveal is this playback plus the on-screen
/// answer, never voice).
pub fn reveal_score(item: Item) -> Score {
    let mut notes = Vec::new();
    let path = resolution_path(item.key, item.degree);
    let mut onset = 0.0;
    for (index, degree) in path.iter().enumerate() {
        let last = index == path.len() - 1;
        let duration = if last {
            TIMING.reveal_home_beats
        } else {
            TIMING.reveal_step_beats
        };
        push_note(&mut notes, melody_midi(item, *degree), onset, duration, 84);
        onset += duration;
    }
    Score {
        tempo_bpm: TIMING.tempo_bpm,
        notes,
    }
}

/// One side of the compare alternation: the degree sounding, then its
/// first resolution step (the moving tone the teaching line points at).
pub fn compare_side_score(item: Item, degree: Degree) -> Score {
    let mut notes = Vec::new();
    let path = resolution_path(item.key, degree);
    let sustain = TIMING.compare_side_beats * 0.6;
    push_note(&mut notes, melody_midi(item, degree), 0.0, sustain, 86);
    if let Some(next) = path.get(1) {
        push_note(
            &mut notes,
            melody_midi(item, *next),
            sustain,
            TIMING.compare_side_beats - sustain,
            80,
        );
    }
    Score {
        tempo_bpm: TIMING.tempo_bpm,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::{Key, Mode, PitchClass};

    fn item() -> Item {
        Item {
            key: Key::new(PitchClass::EB, Mode::Major),
            degree: Degree::new(4), // 3 of E♭ = G
        }
    }

    #[test]
    fn context_score_is_the_cadence_alone() {
        let score = context_score(item());
        // 3 rolled 4-note chords, no melody note.
        assert_eq!(score.notes.len(), 12);
    }

    #[test]
    fn question_score_is_the_note_after_a_breath() {
        let score = question_score(item());
        assert_eq!(score.notes.len(), 1);
        let question = score.notes.last().expect("question note");
        assert_eq!(question.midi % 12, 7); // G
        assert_eq!(question.onset_beats, TIMING.pre_question_rest_beats);
        assert_eq!(question.duration_beats, TIMING.question_beats);
    }

    #[test]
    fn reveal_walks_home_and_lands_on_the_tonic() {
        let score = reveal_score(item());
        // 3 → 2 → 1.
        assert_eq!(score.notes.len(), 3);
        let last = score.notes.last().expect("home note");
        assert_eq!(last.midi % 12, 3); // E♭
        assert_eq!(last.duration_beats, TIMING.reveal_home_beats);
        // Strictly descending walk for a falling path.
        assert!(score.notes.windows(2).all(|w| w[0].midi > w[1].midi));
    }

    #[test]
    fn compare_side_plays_the_color_then_its_moving_tone() {
        let score = compare_side_score(item(), Degree::new(4));
        assert_eq!(score.notes.len(), 2);
        assert_eq!(score.notes[0].midi % 12, 7); // G…
        assert_eq!(score.notes[1].midi % 12, 5); // …moves to F (degree 2)
    }

    #[test]
    fn tonic_compare_side_is_just_the_home_note() {
        let tonic_item = Item {
            degree: Degree::TONIC,
            ..item()
        };
        let score = compare_side_score(tonic_item, Degree::TONIC);
        assert_eq!(score.notes.len(), 1);
    }
}
