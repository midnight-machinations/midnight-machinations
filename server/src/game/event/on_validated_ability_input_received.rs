use crate::game::{
    controllers::ControllerInput, event::EventData, player::PlayerReference,
};

#[must_use = "Event must be invoked"]
pub struct OnValidatedControllerInputReceived{
    pub actor_ref: PlayerReference,
    pub input: ControllerInput,
}
impl OnValidatedControllerInputReceived{
    pub fn new(actor_ref: PlayerReference, input: ControllerInput) -> Self {
        Self{actor_ref, input}
    }
}
impl EventData for OnValidatedControllerInputReceived{
    type FoldValue = ();
}