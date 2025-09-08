use crate::game::{
    abilities_component::Abilities, components::mafia::Mafia, controllers::ControllerID, event::{on_controller_changed::OnControllerChanged, Event}, Game
};

#[must_use = "Event must be invoked"]
pub struct OnControllerSelectionChanged{
    pub id: ControllerID,
}
impl OnControllerSelectionChanged{
    pub fn new(id: ControllerID) -> Self{
        Self{id}
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
impl Event for OnControllerSelectionChanged{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Mafia::on_controller_selection_changed,
        Abilities::on_controller_selection_changed
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}