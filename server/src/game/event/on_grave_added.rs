use crate::game::{
    abilities_component::Abilities, components::graves::{grave_reference::GraveReference, Graves},
    event::EventData, modifiers::ModifierSettings, Game
};

#[must_use = "Event must be invoked"]
pub struct OnGraveAdded{
    pub grave: GraveReference,
}
impl OnGraveAdded{
    pub fn new(grave: GraveReference) -> (Self, ()) {
        (Self{grave}, ())
    }
}
impl EventData for OnGraveAdded{
    type FoldValue = ();

    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_grave_added,
        ModifierSettings::on_grave_added,
        Graves::on_grave_added
    ]}

    fn log(&self, game: &Game, _fold: &()) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "grave": self.grave.deref(game) }))
    }
}