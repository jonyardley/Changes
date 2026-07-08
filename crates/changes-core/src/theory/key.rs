//! Key centers. Everything is heard against one of these — functional over
//! intervallic is the product's first principle.

use serde::{Deserialize, Serialize};

use super::degree::Degree;
use super::pitch::PitchClass;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Mode {
    Major,
    /// Natural minor; harmonic/melodic colors arrive as chromatic degrees.
    Minor,
}

impl Mode {
    /// Scale tones as semitones above the tonic.
    pub fn scale_semitones(self) -> [u8; 7] {
        match self {
            Mode::Major => [0, 2, 4, 5, 7, 9, 11],
            Mode::Minor => [0, 2, 3, 5, 7, 8, 10],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct Key {
    pub tonic: PitchClass,
    pub mode: Mode,
}

impl Key {
    pub fn new(tonic: PitchClass, mode: Mode) -> Self {
        Key { tonic, mode }
    }

    /// The seven scale degrees of this key, as degrees.
    pub fn scale_degrees(self) -> [Degree; 7] {
        self.mode.scale_semitones().map(Degree::new)
    }

    pub fn is_diatonic(self, degree: Degree) -> bool {
        self.mode.scale_semitones().contains(&degree.semitones())
    }

    pub fn pitch_class_of(self, degree: Degree) -> PitchClass {
        self.tonic.transpose(degree.semitones() as i8)
    }

    pub fn degree_of(self, pitch_class: PitchClass) -> Degree {
        Degree::new(self.tonic.distance_up_to(pitch_class))
    }

    pub fn label_of(self, degree: Degree) -> &'static str {
        match self.mode {
            Mode::Major => degree.label_major(),
            Mode::Minor => degree.label_minor(),
        }
    }

    /// Key name for display ("in E♭"). Flat spellings throughout — the
    /// jazz-standard default for key badges; proper per-key spelling
    /// (sharp keys) can refine this later without changing identities.
    pub fn tonic_name(self) -> &'static str {
        const NAMES: [&str; 12] = [
            "C", "D♭", "D", "E♭", "E", "F", "G♭", "G", "A♭", "A", "B♭", "B",
        ];
        NAMES[self.tonic.value() as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eb_major() -> Key {
        Key::new(PitchClass::EB, Mode::Major)
    }

    #[test]
    fn degree_pitch_round_trip_all_keys_all_degrees() {
        for tonic in 0..12u8 {
            for mode in [Mode::Major, Mode::Minor] {
                let key = Key::new(PitchClass::new(tonic), mode);
                for degree in Degree::all() {
                    let pc = key.pitch_class_of(degree);
                    assert_eq!(key.degree_of(pc), degree);
                }
            }
        }
    }

    #[test]
    fn eb_major_scale_pitch_classes() {
        let key = eb_major();
        let classes: Vec<u8> = key
            .scale_degrees()
            .iter()
            .map(|d| key.pitch_class_of(*d).value())
            .collect();
        // Eb F G Ab Bb C D
        assert_eq!(classes, vec![3, 5, 7, 8, 10, 0, 2]);
    }

    #[test]
    fn diatonic_membership_differs_by_mode() {
        let major = eb_major();
        let minor = Key::new(PitchClass::EB, Mode::Minor);
        let flat3 = Degree::new(3);
        assert!(!major.is_diatonic(flat3));
        assert!(minor.is_diatonic(flat3));
    }

    #[test]
    fn tonic_names_use_flat_spellings() {
        assert_eq!(eb_major().tonic_name(), "E♭");
        assert_eq!(Key::new(PitchClass::new(6), Mode::Major).tonic_name(), "G♭");
    }
}
