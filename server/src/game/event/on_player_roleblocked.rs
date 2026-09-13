use crate::game::{
    abilities_component::Abilities, components::{blocked::BlockedComponent, mafia::Mafia}, event::EventData, player::PlayerReference, Game
};

use super::on_midnight::OnMidnightFold;

#[must_use = "Event must be invoked"]
pub struct OnPlayerRoleblocked{
    pub player: PlayerReference,
    pub invisible: bool,
}
impl OnPlayerRoleblocked{
    pub fn new(player: PlayerReference, invisible: bool) -> Self{
        Self{player, invisible}
    }
}
impl EventData for OnPlayerRoleblocked{
    type FoldValue = OnMidnightFold;
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_player_roleblocked,
        Mafia::on_player_roleblocked,
        BlockedComponent::on_player_roleblocked,
    ]}

    fn log(&self, _game: &Game, _fold: &OnMidnightFold) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "player": self.player, "invisible": self.invisible }))
    }
}