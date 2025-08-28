use crate::game::{
    controllers::{Controllers, ControllerInput},
    modifiers::ModifierSettings, 
    player::PlayerReference,
    Game
};

#[must_use = "Event must be invoked"]
pub struct OnControllerInputReceived{
    actor_ref: PlayerReference,
    input: ControllerInput,
}
impl OnControllerInputReceived{
    pub fn new(actor_ref: PlayerReference, input: ControllerInput) -> Self{
        Self{actor_ref, input}
    }
    pub fn invoke(self, game: &mut Game){
        for player_ref in PlayerReference::all_players(game){
            player_ref.on_ability_input_received(game, self.actor_ref, self.input.clone())
        }
        ModifierSettings::on_ability_input_received(game, self.actor_ref, self.input.clone());
        Controllers::on_ability_input_received(game, self.actor_ref, self.input);
    }
}