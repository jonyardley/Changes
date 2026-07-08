//! Pitch fundamentals. A `PitchClass` is 0–11 (C = 0); a concrete sounding
//! note is a midi number. Spelling (E♭ vs D♯) is decided by the key at
//! label time, not stored in the pitch.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct PitchClass(u8);

impl PitchClass {
    pub const C: PitchClass = PitchClass(0);
    pub const EB: PitchClass = PitchClass(3);
    pub const F: PitchClass = PitchClass(5);
    pub const G: PitchClass = PitchClass(7);
    pub const BB: PitchClass = PitchClass(10);

    pub fn new(semitones: u8) -> Self {
        PitchClass(semitones % 12)
    }

    pub fn value(self) -> u8 {
        self.0
    }

    pub fn transpose(self, semitones: i8) -> Self {
        PitchClass((self.0 as i16 + semitones as i16).rem_euclid(12) as u8)
    }

    /// Semitones from `self` up to `other` (0–11).
    pub fn distance_up_to(self, other: PitchClass) -> u8 {
        (other.0 as i16 - self.0 as i16).rem_euclid(12) as u8
    }

    /// The midi note for this class nearest to `reference` (ties resolve
    /// downward, keeping voicings in register).
    pub fn midi_near(self, reference: u8) -> u8 {
        let base = reference as i16;
        let mut best = i16::MAX;
        let mut best_dist = i16::MAX;
        for octave in -1..=10 {
            let candidate = octave * 12 + self.0 as i16;
            if !(0..=127).contains(&candidate) {
                continue;
            }
            let dist = (candidate - base).abs();
            if dist < best_dist || (dist == best_dist && candidate < best) {
                best = candidate;
                best_dist = dist;
            }
        }
        best as u8
    }
}

impl From<u8> for PitchClass {
    fn from(midi: u8) -> Self {
        PitchClass(midi % 12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transpose_wraps_both_directions() {
        assert_eq!(PitchClass::C.transpose(3), PitchClass::EB);
        assert_eq!(PitchClass::C.transpose(-2), PitchClass::BB);
        assert_eq!(PitchClass::new(11).transpose(2), PitchClass::new(1));
    }

    #[test]
    fn distance_up_is_mod_twelve() {
        assert_eq!(PitchClass::C.distance_up_to(PitchClass::EB), 3);
        assert_eq!(PitchClass::EB.distance_up_to(PitchClass::C), 9);
        assert_eq!(PitchClass::G.distance_up_to(PitchClass::G), 0);
    }

    #[test]
    fn midi_near_picks_the_closest_octave() {
        // Eb nearest middle C (60) is Eb4 (63), not Eb3 (51).
        assert_eq!(PitchClass::EB.midi_near(60), 63);
        // G nearest C4: G3 (55) at distance 5 beats G4 (67) at 7.
        assert_eq!(PitchClass::G.midi_near(60), 55);
    }

    #[test]
    fn midi_near_tie_resolves_downward() {
        // F# is 6 semitones from C either way — take the lower.
        assert_eq!(PitchClass::new(6).midi_near(60), 54);
    }
}
