//! The audio boundary: the core decides WHAT plays and WHEN in musical
//! terms (an abstract score); the shell owns precise realization on
//! `AVAudioEngine` (sample-accurate scheduling, session config,
//! interruptions). The shell never invents, transposes, or re-voices
//! musical content.

use crux_core::capability::Operation;
use crux_core::command::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum InstrumentRole {
    Piano,
    Bass,
    Ride,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct ScoreNote {
    pub midi: u8,
    pub onset_beats: f32,
    pub duration_beats: f32,
    pub velocity: u8,
    pub role: InstrumentRole,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct Score {
    pub tempo_bpm: f32,
    pub notes: Vec<ScoreNote>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct PlayScoreOperation {
    pub score: Score,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PlayScoreOutput {
    Finished,
    /// Real failure (engine/soundfont), surfaced to a UI state — a failed
    /// effect is never a silent success.
    Failed {
        message: String,
    },
}

impl Operation for PlayScoreOperation {
    type Output = PlayScoreOutput;
}

pub fn play_score(score: Score) -> Command<Effect, Event> {
    Command::request_from_shell(PlayScoreOperation { score }).then_send(Event::PlaybackFinished)
}
