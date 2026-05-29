use crate::game::{
    controllers::{ControllerID, Controller, Controllers},
    event::EventData
};
use super::{EventListenerFunction, LegacyEventData, on_controller_selection_changed::OnControllerSelectionChanged};

pub struct OnControllerChanged{
    pub old: Option<Controller>,
    pub new: Option<Controller>,
    pub id: ControllerID
}
impl EventData for OnControllerChanged {
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnControllerChanged {
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {
        vec![
            Controllers::send_controller_to_client,
            OnControllerSelectionChanged::on_controller_changed
        ]
    }
}
impl OnControllerChanged{
    pub fn new(id: ControllerID, old: Option<Controller>, new: Option<Controller>)->(Self, ()){
        (Self{id, old, new}, ())
    }
}