//! The 12 chromatic degree "colors" against a key center (Bruce Arnold's
//! One Note frame; rungs 0–2 curriculum). A degree is identity-by-function,
//! not by spelling: semitones above the tonic, labelled per mode.

use serde::{Deserialize, Serialize};

/// Semitones above the tonic (0–11), the degree's functional identity.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct Degree(u8);

impl Degree {
    pub const TONIC: Degree = Degree(0);

    pub fn new(semitones_above_tonic: u8) -> Self {
        Degree(semitones_above_tonic % 12)
    }

    pub fn semitones(self) -> u8 {
        self.0
    }

    /// All 12 colors, tonic first.
    pub fn all() -> [Degree; 12] {
        core::array::from_fn(|i| Degree(i as u8))
    }

    /// Conventional jazz label relative to a major key center
    /// (♭2 ♯4-as-♯4 ♭6 ♭7 spellings; the tension palette's vocabulary).
    pub fn label_major(self) -> &'static str {
        match self.0 {
            0 => "1",
            1 => "♭2",
            2 => "2",
            3 => "♭3",
            4 => "3",
            5 => "4",
            6 => "♯4",
            7 => "5",
            8 => "♭6",
            9 => "6",
            10 => "♭7",
            11 => "7",
            // Unreachable: constructor wraps mod 12.
            _ => "?",
        }
    }

    /// Label against a natural-minor key center: the minor scale's own
    /// tones read plainly (3 means ♭3's pitch), alterations marked.
    pub fn label_minor(self) -> &'static str {
        match self.0 {
            0 => "1",
            1 => "♭2",
            2 => "2",
            3 => "3",
            4 => "♮3",
            5 => "4",
            6 => "♯4",
            7 => "5",
            8 => "6",
            9 => "♮6",
            10 => "7",
            11 => "♮7",
            _ => "?",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_distinct_colors() {
        let all = Degree::all();
        assert_eq!(all.len(), 12);
        for (i, d) in all.iter().enumerate() {
            assert_eq!(d.semitones(), i as u8);
        }
    }

    #[test]
    fn major_labels_match_jazz_convention() {
        assert_eq!(Degree::new(0).label_major(), "1");
        assert_eq!(Degree::new(3).label_major(), "♭3");
        assert_eq!(Degree::new(6).label_major(), "♯4");
        assert_eq!(Degree::new(10).label_major(), "♭7");
    }

    #[test]
    fn minor_labels_read_the_minor_scale_plainly() {
        assert_eq!(Degree::new(3).label_minor(), "3");
        assert_eq!(Degree::new(4).label_minor(), "♮3");
        assert_eq!(Degree::new(8).label_minor(), "6");
        assert_eq!(Degree::new(10).label_minor(), "7");
    }
}
