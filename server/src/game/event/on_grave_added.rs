use crate::game::event::EventData;
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnGraveAdded{
    pub grave: crate::game::components::graves::grave_reference::GraveReference,
}
impl OnGraveAdded{
    pub fn new(grave: crate::game::components::graves::grave_reference::GraveReference) -> (Self, ()) {
        (Self{grave}, ())
    }
}
impl EventData for OnGraveAdded{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnGraveAdded{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::abilities_component::Abilities::on_grave_added,
        crate::game::modifiers::ModifierSettings::on_grave_added,
        crate::game::components::graves::Graves::on_grave_added
    ]}
}