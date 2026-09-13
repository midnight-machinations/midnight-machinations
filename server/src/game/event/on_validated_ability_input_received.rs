use crate::game::{
    abilities_component::Abilities, chat::ChatComponent,
    components::{
        call_witness::CallWitness, forward_messages::ForwardMessages, nomination_controller::NominationController,
    },
    controllers::ControllerInput, event::EventData, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnValidatedControllerInputReceived{
    pub actor_ref: PlayerReference,
    pub input: ControllerInput,
}
impl OnValidatedControllerInputReceived{
    pub fn new(actor_ref: PlayerReference, input: ControllerInput) -> (Self, ()) {
        (Self{actor_ref, input}, ())
    }
}
impl EventData for OnValidatedControllerInputReceived{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            Abilities::on_validated_ability_input_received,
            ForwardMessages::on_validated_ability_input_received,
            NominationController::on_validated_ability_input_received,
            ChatComponent::on_validated_ability_input_received,
            CallWitness::on_validated_ability_input_received
        ]
    }

    fn log(&self, _game: &Game, _fold: &()) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "actor": self.actor_ref, "input": self.input.clone() }))
    }
}