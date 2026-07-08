//! Resolution paths — the Benbassat move that names a degree by how it
//! wants to resolve: stepwise along the scale to the nearest tonic. This is
//! the aural reveal's raw material (mvp-plan decision 5).
//!
//! Direction rule: shortest way home by semitone distance — below the
//! tritone falls, the tritone and above rises (design's one documented
//! example, "♭3 resolves down to do", holds). A chromatic note's first
//! step lands on the adjacent scale tone in that direction, then the walk
//! is diatonic. The rule lives in one function (`resolves_upward`) so
//! dogfooding can retune it (e.g. classical le→sol) without touching
//! callers.

use super::degree::Degree;
use super::key::Key;

/// The degrees traversed from `start` to the tonic, inclusive of both ends.
/// The tonic itself yields `[1]` — it is already home (rung 0: resolve any
/// note to Do).
pub fn resolution_path(key: Key, start: Degree) -> Vec<Degree> {
    let scale = key.mode.scale_semitones();
    let mut path = vec![start];
    let mut current = start.semitones();

    if current == 0 {
        return path;
    }

    let upward = resolves_upward(current);

    // First step off a chromatic note lands on the neighbouring scale tone.
    if !scale.contains(&current) {
        current = if upward {
            scale.iter().copied().find(|&s| s > current).unwrap_or(0)
        } else {
            scale
                .iter()
                .rev()
                .copied()
                .find(|&s| s < current)
                .unwrap_or(0)
        };
        path.push(Degree::new(current));
        if current == 0 {
            return path;
        }
    }

    // Walk scale positions to the tonic (position 7 ≡ upper Do, 0 ≡ Do).
    let mut position = scale.iter().position(|&s| s == current).unwrap_or_default() as i8;
    loop {
        position += if upward { 1 } else { -1 };
        if !(1..=6).contains(&position) {
            path.push(Degree::TONIC);
            return path;
        }
        path.push(Degree::new(scale[position as usize]));
    }
}

/// Shortest way home: 1–5 semitones fall, 6–11 rise. The single seam for
/// retuning resolution feel after dogfooding.
fn resolves_upward(semitones: u8) -> bool {
    semitones >= 6
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::key::Mode;
    use crate::theory::pitch::PitchClass;

    fn major() -> Key {
        Key::new(PitchClass::C, Mode::Major)
    }

    fn labels(key: Key, start: u8) -> Vec<&'static str> {
        resolution_path(key, Degree::new(start))
            .iter()
            .map(|d| key.label_of(*d))
            .collect()
    }

    #[test]
    fn tonic_is_already_home() {
        assert_eq!(labels(major(), 0), vec!["1"]);
    }

    #[test]
    fn lower_half_falls_to_do() {
        assert_eq!(labels(major(), 2), vec!["2", "1"]);
        assert_eq!(labels(major(), 4), vec!["3", "2", "1"]);
        assert_eq!(labels(major(), 5), vec!["4", "3", "2", "1"]);
    }

    #[test]
    fn upper_half_rises_to_do() {
        assert_eq!(labels(major(), 7), vec!["5", "6", "7", "1"]);
        assert_eq!(labels(major(), 9), vec!["6", "7", "1"]);
        assert_eq!(labels(major(), 11), vec!["7", "1"]);
    }

    #[test]
    fn lowered_chromatics_below_the_tritone_fall() {
        assert_eq!(labels(major(), 1), vec!["♭2", "1"]);
        assert_eq!(labels(major(), 3), vec!["♭3", "2", "1"]);
    }

    #[test]
    fn tritone_and_above_rise_home() {
        assert_eq!(labels(major(), 6), vec!["♯4", "5", "6", "7", "1"]);
        assert_eq!(labels(major(), 8), vec!["♭6", "6", "7", "1"]);
        assert_eq!(labels(major(), 10), vec!["♭7", "7", "1"]);
    }

    #[test]
    fn every_color_resolves_home_in_every_key() {
        for tonic in 0..12u8 {
            for mode in [Mode::Major, Mode::Minor] {
                let key = Key::new(PitchClass::new(tonic), mode);
                for degree in Degree::all() {
                    let path = resolution_path(key, degree);
                    assert_eq!(path.first(), Some(&degree));
                    assert_eq!(path.last(), Some(&Degree::TONIC), "{degree:?} in {key:?}");
                    assert!(path.len() <= 7, "paths are short walks, not tours");
                    // After the first step everything is diatonic.
                    for step in &path[1..] {
                        assert!(key.is_diatonic(*step), "{step:?} in {key:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn minor_paths_walk_the_minor_scale() {
        let key = Key::new(PitchClass::C, Mode::Minor);
        // Minor's own 3rd falls 3-2-1.
        assert_eq!(labels(key, 3), vec!["3", "2", "1"]);
        // Minor 6th (8 semitones) is diatonic and rises through minor 7.
        assert_eq!(labels(key, 8), vec!["6", "7", "1"]);
        // ♮7 (11) is chromatic in natural minor and rises home.
        assert_eq!(labels(key, 11), vec!["♮7", "1"]);
    }
}
