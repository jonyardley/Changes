//! M1 audio-spike content: hardcoded cadence → chord → reveal scores in E♭.
//! Exists to exercise the audio pipeline (mvp-plan M1 exit criteria), not to
//! be the theory engine — M2 replaces this with the real generator.

use crate::audio::{InstrumentRole, Score, ScoreNote};

/// Playback timings are config data, not constants scattered in code
/// (mvp-plan decision 8). Values from the design prototype, validated by
/// ear; at 60bpm one beat is one second.
pub struct SpikeTiming {
    pub tempo_bpm: f32,
    pub cadence_chord_beats: f32,
    pub cadence_resolve_beats: f32,
    pub question_beats: f32,
    pub reveal_beats: f32,
    /// Slight roll between chord tones (~35ms in the prototype).
    pub roll_beats: f32,
}

pub const TIMING: SpikeTiming = SpikeTiming {
    tempo_bpm: 60.0,
    cadence_chord_beats: 0.95,
    cadence_resolve_beats: 1.5,
    question_beats: 2.2,
    reveal_beats: 1.6,
    roll_beats: 0.035,
};

/// The three qualities the spike cycles through, all over E♭.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpikeChord {
    EbMaj7,
    EbM7,
    Eb7,
}

pub const SPIKE_CHORDS: [SpikeChord; 3] = [SpikeChord::EbMaj7, SpikeChord::EbM7, SpikeChord::Eb7];

impl SpikeChord {
    pub fn label(self) -> &'static str {
        match self {
            SpikeChord::EbMaj7 => "E♭maj7",
            SpikeChord::EbM7 => "E♭m7",
            SpikeChord::Eb7 => "E♭7",
        }
    }

    fn midi_notes(self) -> [u8; 4] {
        match self {
            // Eb3 G3 Bb3 D4
            SpikeChord::EbMaj7 => [51, 55, 58, 62],
            // Eb3 Gb3 Bb3 Db4
            SpikeChord::EbM7 => [51, 54, 58, 61],
            // Eb3 G3 Bb3 Db4
            SpikeChord::Eb7 => [51, 55, 58, 61],
        }
    }
}

fn chord(notes: &[u8], onset: f32, duration: f32, velocity: u8, out: &mut Vec<ScoreNote>) {
    for (i, &midi) in notes.iter().enumerate() {
        let roll = TIMING.roll_beats * i as f32;
        out.push(ScoreNote {
            midi,
            onset_beats: onset + roll,
            duration_beats: duration - roll,
            velocity,
            role: InstrumentRole::Piano,
        });
    }
}

/// ii–V–I in E♭ (Fm7 → B♭7 → E♭maj7) establishing the key.
pub fn cadence_score() -> Score {
    let mut notes = Vec::new();
    // Fm7: F3 Ab3 C4 Eb4
    chord(
        &[53, 56, 60, 63],
        0.0,
        TIMING.cadence_chord_beats,
        76,
        &mut notes,
    );
    // Bb7: F3 Ab3 Bb3 D4
    chord(
        &[53, 56, 58, 62],
        TIMING.cadence_chord_beats,
        TIMING.cadence_chord_beats,
        76,
        &mut notes,
    );
    // Ebmaj7: Eb3 G3 Bb3 D4
    chord(
        &[51, 55, 58, 62],
        TIMING.cadence_chord_beats * 2.0,
        TIMING.cadence_resolve_beats,
        80,
        &mut notes,
    );
    Score {
        tempo_bpm: TIMING.tempo_bpm,
        notes,
    }
}

pub fn question_score(item: SpikeChord) -> Score {
    let mut notes = Vec::new();
    chord(
        &item.midi_notes(),
        0.0,
        TIMING.question_beats,
        84,
        &mut notes,
    );
    Score {
        tempo_bpm: TIMING.tempo_bpm,
        notes,
    }
}

pub fn reveal_score(item: SpikeChord) -> Score {
    let mut notes = Vec::new();
    chord(&item.midi_notes(), 0.0, TIMING.reveal_beats, 84, &mut notes);
    Score {
        tempo_bpm: TIMING.tempo_bpm,
        notes,
    }
}
