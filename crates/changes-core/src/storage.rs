//! Typed storage effects the shell fulfils against on-device SQLite (GRDB).
//! Schema + choreography: docs/specs/srs-persistence.md. The device is the
//! only copy of practice history — a failed write resolves `Failed` and
//! surfaces; never a silent success.

use crux_core::capability::Operation;
use crux_core::command::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::srs::{ReviewLog, ReviewState};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum StorageOperation {
    LoadReviews,
    SaveReview { state: ReviewState, log: ReviewLog },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum StorageOutput {
    Reviews(Vec<ReviewState>),
    Ack,
    /// The local store failed the op — surfaced, not trusted as success.
    Failed {
        message: String,
    },
}

impl Operation for StorageOperation {
    type Output = StorageOutput;
}

pub fn load_reviews() -> Command<Effect, Event> {
    Command::request_from_shell(StorageOperation::LoadReviews).then_send(Event::ReviewsLoaded)
}

pub fn save_review(state: ReviewState, log: ReviewLog) -> Command<Effect, Event> {
    Command::request_from_shell(StorageOperation::SaveReview { state, log })
        .then_send(Event::ReviewSaved)
}
