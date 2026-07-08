use crux_core::{
    bridge::{Bridge, EffectId},
    Core,
};

use crate::Changes;

#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("core bridge error: {0}")]
    Bridge(String),
}

/// The single object the Swift shell talks to. All payloads are positional
/// bincode; the Swift side deserializes with the generated `SharedTypes`.
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct CoreFFI {
    core: Bridge<Changes>,
}

impl Default for CoreFFI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl CoreFFI {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new() -> Self {
        Self {
            core: Bridge::new(Core::new()),
        }
    }

    pub fn update(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .update(data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

    pub fn resolve(&self, id: u32, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .resolve(EffectId(id), data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

    pub fn view(&self) -> Result<Vec<u8>, CoreError> {
        let mut view = Vec::new();
        self.core
            .view(&mut view)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    use changes_core::{Event, ViewModel};

    use super::*;

    // The real-bridge round trip: an event in over the wire format, effects
    // + ViewModel back out — the same path the shell takes.
    #[test]
    fn start_session_round_trips_through_the_bridge() {
        let core = CoreFFI::new();

        let event = bincode::serialize(&Event::StartSession { seed: 42 }).unwrap();
        let effects = core.update(&event).unwrap();
        assert!(
            !effects.is_empty(),
            "StartSession must request render + play effects"
        );

        let view: ViewModel = bincode::deserialize(&core.view().unwrap()).unwrap();
        assert_eq!(view.phase, changes_core::Phase::Listening);
        assert!(view.is_playing);
        assert!(!view.key_name.is_empty());
    }
}
