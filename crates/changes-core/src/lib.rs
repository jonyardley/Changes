//! Pure Crux core for Changes: theory engine, exercise generation, session
//! state machine, SRS scheduling, grading. No I/O — shells fulfil effects.

pub mod app;
#[cfg(test)]
pub(crate) mod test_support;

pub use app::{Changes, Effect, Event, Model, ViewModel};
