use crate::game::{
    components::mafia::Mafia, controllers::ControllerID, event::on_controller_changed::OnControllerChanged, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnControllerSelectionChanged{
    id: ControllerID,
}
impl OnControllerSelectionChanged{
    pub fn new(id: ControllerID) -> Self{
        Self{id}
    }
    pub fn invoke(self, game: &mut Game){
        Mafia::on_controller_selection_changed(game, self.id.clone());
        for player in PlayerReference::all_players(game){
            player.on_controller_selection_changed(game, self.id.clone());
        }
    }
    pub(super) fn on_controller_changed(game: &mut Game, event: &OnControllerChanged, _fold: &mut (), _priority: ()){
        if
            event.new.as_ref().map(|c|c.selection()) != 
            event.old.as_ref().map(|c|c.selection())
        {
            Self::new(event.id.clone()).invoke(game);
        }
    }
}