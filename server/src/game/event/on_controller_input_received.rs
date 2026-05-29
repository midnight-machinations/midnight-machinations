use crate::game::{
    controllers::{ControllerInput, Controllers},
    player::PlayerReference,
};
use crate::game::event::EventData;
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnControllerInputReceived{
    pub actor_ref: PlayerReference,
    pub input: ControllerInput,
}
impl OnControllerInputReceived{
    pub fn new(actor_ref: PlayerReference, input: ControllerInput) -> (Self, ()){
        (Self{actor_ref, input}, ())
    }
}
impl EventData for OnControllerInputReceived{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnControllerInputReceived{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        Controllers::on_controller_input_received
    ]}
}