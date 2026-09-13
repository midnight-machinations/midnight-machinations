use crate::game::{
    controllers::{ControllerInput, Controllers}, event::EventData,
    player::PlayerReference, Game
};

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
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Controllers::on_controller_input_received
    ]}

    fn log(&self, _game: &Game, _fold: &()) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "actor": self.actor_ref, "input": self.input.clone() }))
    }
}