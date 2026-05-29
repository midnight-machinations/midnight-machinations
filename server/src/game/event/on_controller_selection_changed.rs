use crate::game::{
    abilities_component::Abilities, components::mafia::Mafia, controllers::ControllerID, event::EventData,
};
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnControllerSelectionChanged{
    pub id: ControllerID,
}
impl OnControllerSelectionChanged{
    pub fn new(id: ControllerID) -> (Self, ()){
        (Self{id}, ())
    }
}
impl EventData for OnControllerSelectionChanged{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnControllerSelectionChanged{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        Mafia::on_controller_selection_changed,
        Abilities::on_controller_selection_changed
    ]}
}