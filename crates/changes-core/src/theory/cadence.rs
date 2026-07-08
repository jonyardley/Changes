//! Key-establishing cadences: ii–V–I(major) / iiø–V–i(minor) as concrete
//! voicings, transposable to any key. Template shapes are the M1 spike's
//! E♭ voicings expressed as offsets from the tonic; register is anchored
//! so every key sits in the same comfortable mid-range.

use super::key::{Key, Mode};

/// One cadence chord as sounding midi notes, low to high.
pub type Voicing = Vec<u8>;

/// Anchor for the tonic's bass register (E♭3 — where the spike voicings
/// sat). `midi_near` keeps every key within a tritone of it.
const TONIC_ANCHOR_MIDI: u8 = 51;

/// Semitone offsets from the tonic for [ii, V, I].
fn template(mode: Mode) -> [[i8; 4]; 3] {
    match mode {
        // Fm7 → B♭7 → E♭maj7 relative to E♭.
        Mode::Major => [[2, 5, 9, 12], [2, 5, 7, 11], [0, 4, 7, 11]],
        // iiø7 → V7 → i m7.
        Mode::Minor => [[2, 5, 8, 12], [2, 5, 7, 11], [0, 3, 7, 10]],
    }
}

/// The three cadence voicings for a key, in playing order.
pub fn two_five_one(key: Key) -> [Voicing; 3] {
    let tonic_midi = key.tonic.midi_near(TONIC_ANCHOR_MIDI) as i16;
    template(key.mode).map(|chord| {
        chord
            .iter()
            .map(|&offset| (tonic_midi + offset as i16) as u8)
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::pitch::PitchClass;

    #[test]
    fn eb_major_cadence_matches_the_spike_voicings() {
        let key = Key::new(PitchClass::EB, Mode::Major);
        let [ii, v, i] = two_five_one(key);
        assert_eq!(ii, vec![53, 56, 60, 63]); // Fm7
        assert_eq!(v, vec![53, 56, 58, 62]); // B♭7
        assert_eq!(i, vec![51, 55, 58, 62]); // E♭maj7
    }

    #[test]
    fn every_key_lands_i_on_its_tonic_and_stays_in_register() {
        for tonic in 0..12u8 {
            for mode in [Mode::Major, Mode::Minor] {
                let key = Key::new(PitchClass::new(tonic), mode);
                let [_, _, one] = two_five_one(key);
                assert_eq!(
                    one[0] % 12,
                    key.tonic.value(),
                    "I chord roots the tonic in {key:?}"
                );
                for voicing in two_five_one(key) {
                    assert!(voicing.windows(2).all(|w| w[0] < w[1]), "low to high");
                    assert!(
                        voicing.iter().all(|&n| (40..=80).contains(&n)),
                        "mid-register in {key:?}: {voicing:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn minor_cadence_carries_the_quality_defining_tones() {
        let key = Key::new(PitchClass::C, Mode::Minor);
        let [ii, five, one] = two_five_one(key);
        // iiø over C minor: D F A♭ C.
        assert_eq!(
            ii.iter().map(|n| n % 12).collect::<Vec<_>>(),
            vec![2, 5, 8, 0]
        );
        // V7: D F G B — leading tone present.
        assert!(five.iter().any(|n| n % 12 == 11));
        // i m7: C E♭ G B♭ — minor third present.
        assert!(one.iter().any(|n| n % 12 == 3));
    }
}
