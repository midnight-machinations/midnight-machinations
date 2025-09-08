use crate::game::{
    abilities_component::Abilities, components::graves::grave_reference::GraveReference,
    event::Event, modifiers::ModifierSettings, Game
};


#[must_use = "Event must be invoked"]
pub struct OnGraveAdded{
    pub grave: GraveReference,
}
impl OnGraveAdded{
    pub fn new(grave: GraveReference) -> Self{
        Self{grave}
    }
}
impl Event for OnGraveAdded{
    type FoldValue = ();

    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_grave_added,
        ModifierSettings::on_grave_added,
        Game::on_grave_added
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}