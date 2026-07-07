use crux_core::macros::effect;
use crux_core::render::{render, RenderOperation};
use crux_core::{App, Command};
use serde::{Deserialize, Serialize};

/// Root Crux application. Walking skeleton: `Event::Ping` proves the
/// event → update → render → view loop end-to-end before real features land.
#[derive(Default)]
pub struct Changes;

/// All events the application can process. Bridge-crossing: serialized over
/// positional bincode to the shell — field order is the wire format.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Event {
    Ping,
}

#[derive(Default, Debug)]
pub struct Model {
    pings: u32,
}

/// Bridge-crossing: what shells render.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub struct ViewModel {
    pub pong_count: u32,
}

/// Side effects the core requests from shells.
#[effect(facet_typegen)]
pub enum Effect {
    Render(RenderOperation),
}

impl App for Changes {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
    ) -> Command<Self::Effect, Self::Event> {
        match event {
            Event::Ping => {
                model.pings += 1;
                render()
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            pong_count: model.pings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_increments_and_requests_render() {
        let app = Changes;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Ping, &mut model);

        assert!(matches!(cmd.effects().next(), Some(Effect::Render(_))));
        assert!(cmd.effects().next().is_none());
        assert_eq!(app.view(&model).pong_count, 1);
    }

    #[test]
    fn pings_accumulate() {
        let app = Changes;
        let mut model = Model::default();

        for _ in 0..3 {
            let _ = app.update(Event::Ping, &mut model);
        }

        assert_eq!(app.view(&model).pong_count, 3);
    }

    // The bridge is positional bincode (non-self-describing): every
    // bridge-crossing type gets a round-trip test via the shared helper so
    // a silent wire break fails here, not as a no-op in the shell
    // (intrada #846). Effect payloads round-trip their operation types —
    // the generated `*Ffi` enum derives no PartialEq/Debug.
    #[test]
    fn event_bincode_round_trip() {
        crate::test_support::assert_bincode_round_trip(&Event::Ping);
    }

    #[test]
    fn view_model_bincode_round_trip() {
        crate::test_support::assert_bincode_round_trip(&ViewModel { pong_count: 42 });
    }

    #[test]
    fn render_operation_bincode_round_trip() {
        crate::test_support::assert_bincode_round_trip(&RenderOperation);
    }
}
