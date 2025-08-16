use crate::game::{
    components::graves::grave_reference::GraveReference, modifiers::Modifiers, player::PlayerReference, Game
};


#[must_use = "Event must be invoked"]
pub struct OnGraveAdded{
    grave: GraveReference,
}
impl OnGraveAdded{
    pub fn new(grave: GraveReference) -> Self{
        Self{grave}
    }
    pub fn invoke(self, game: &mut Game){
        for player in PlayerReference::all_players(game){
            player.on_grave_added(game, self.grave)
        }

        Modifiers::on_grave_added(game, self.grave);

        game.on_grave_added(self.grave);
    }
}