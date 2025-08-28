use crate::game::{
    chat::ChatComponent, components::{
        call_witness::CallWitness, forward_messages::ForwardMessages, nomination_controller::NominationController, syndicate_gun_item::SyndicateGunItem
    }, controllers::ControllerInput, event::Event, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnValidatedControllerInputReceived{
    pub actor_ref: PlayerReference,
    pub input: ControllerInput,
}
impl OnValidatedControllerInputReceived{
    pub fn new(actor_ref: PlayerReference, input: ControllerInput) -> Self{
        Self{actor_ref, input}
    }
}
impl Event for OnValidatedControllerInputReceived{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            PlayerReference::on_validated_ability_input_received,
            SyndicateGunItem::on_validated_ability_input_received,
            ForwardMessages::on_validated_ability_input_received,
            NominationController::on_validated_ability_input_received,
            ChatComponent::on_validated_ability_input_received,
            CallWitness::on_validated_ability_input_received
        ]
    }

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}