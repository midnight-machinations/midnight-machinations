use crate::game::{
    abilities_component::Abilities, components::{graves::{Graves, grave_reference::GraveReference}, synopsis::SynopsisTracker}, event::EventData, modifiers::ModifierSettings
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
        Graves::on_grave_added,
        SynopsisTracker::on_grave_added
    ]}
}