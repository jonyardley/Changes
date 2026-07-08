//! Playback timings are config data, not constants scattered in code
//! (mvp-plan decision 8). Values from the design prototype, validated by
//! ear; at 60bpm one beat is one second. The prototype's thinking-gap and
//! auto-continue values are deliberately absent — manual pacing (decision
//! 9): the gap is open-ended, transitions wait for taps.

pub struct SessionTiming {
    pub tempo_bpm: f32,
    /// ii and V of the establishing cadence.
    pub cadence_chord_beats: f32,
    /// The I chord, slightly longer — let the key land.
    pub cadence_resolve_beats: f32,
    /// Silence between cadence and the question note.
    pub pre_question_rest_beats: f32,
    /// The question note's sustain.
    pub question_beats: f32,
    /// Each passing step of the aural reveal's resolution walk.
    pub reveal_step_beats: f32,
    /// The arrival on the tonic.
    pub reveal_home_beats: f32,
    /// One side of the compare (Banacos) alternation (~1.7s).
    pub compare_side_beats: f32,
    /// Slight roll between chord tones (~35ms in the prototype).
    pub roll_beats: f32,
}

pub const TIMING: SessionTiming = SessionTiming {
    tempo_bpm: 60.0,
    cadence_chord_beats: 0.95,
    cadence_resolve_beats: 1.5,
    pre_question_rest_beats: 0.6,
    question_beats: 2.2,
    reveal_step_beats: 0.55,
    reveal_home_beats: 1.4,
    compare_side_beats: 1.7,
    roll_beats: 0.035,
};
