//! Pure Crux core for Changes: theory engine, exercise generation, session
//! state machine, SRS scheduling, grading. No I/O — shells fulfil effects.

pub mod app;
pub mod audio;
pub mod rng;
pub mod session;
#[cfg(test)]
pub(crate) mod test_support;
pub mod theory;

pub use app::{
    AnswerView, Changes, CompareView, Effect, Event, Model, Phase, RecapView, ViewModel,
};
pub use audio::{PlayScoreOperation, PlayScoreOutput, Score, ScoreNote};
