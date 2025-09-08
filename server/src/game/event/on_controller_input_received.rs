use crate::game::{
    controllers::{ControllerInput, Controllers}, event::Event,
    player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnControllerInputReceived{
    pub actor_ref: PlayerReference,
    pub input: ControllerInput,
}
impl OnControllerInputReceived{
    pub fn new(actor_ref: PlayerReference, input: ControllerInput) -> Self{
        Self{actor_ref, input}
    }
}
impl Event for OnControllerInputReceived{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Controllers::on_controller_input_received
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}