use crate::game::{
    controllers::{ControllerID, Controller, Controllers},
    event::{on_controller_selection_changed::OnControllerSelectionChanged, Event}
};

pub struct OnControllerChanged{
    pub old: Option<Controller>,
    pub new: Option<Controller>,
    pub id: ControllerID
}
impl Event for OnControllerChanged {
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            Controllers::send_controller_to_client,
            OnControllerSelectionChanged::on_controller_changed
        ]
    }
    fn initial_fold_value(&self, _game: &crate::game::Game) -> Self::FoldValue {}
}
impl OnControllerChanged{
    pub fn new(id: ControllerID, old: Option<Controller>, new: Option<Controller>)->Self{
        Self{id, old, new}
    }
}