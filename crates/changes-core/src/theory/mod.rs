//! Theory engine v1 (rungs 0–2 scope): pitch classes, keys, the 12 degree
//! colors, key-establishing cadences, and resolution paths. First-party by
//! design — this is core domain, not a dependency to outsource.

pub mod cadence;
pub mod degree;
pub mod key;
pub mod pitch;
pub mod resolution;

pub use cadence::{two_five_one, Voicing};
pub use degree::Degree;
pub use key::{Key, Mode};
pub use pitch::PitchClass;
pub use resolution::resolution_path;
