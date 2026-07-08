//! Pure Crux core for Changes: theory engine, exercise generation, session
//! state machine, SRS scheduling, grading. No I/O — shells fulfil effects.

pub mod app;
pub mod audio;
pub mod spike;
#[cfg(test)]
pub(crate) mod test_support;
pub mod theory;

pub use app::{Changes, Effect, Event, Model, Phase, ViewModel};
pub use audio::{PlayScoreOperation, PlayScoreOutput, Score, ScoreNote};
