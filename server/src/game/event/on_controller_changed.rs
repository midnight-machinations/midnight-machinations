use crate::game::{
    controllers::{ControllerID, Controller, Controllers},
    event::{on_controller_selection_changed::OnControllerSelectionChanged, EventData}
};

pub struct OnControllerChanged{
    pub old: Option<Controller>,
    pub new: Option<Controller>,
    pub id: ControllerID
}
impl EventData for OnControllerChanged {
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
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