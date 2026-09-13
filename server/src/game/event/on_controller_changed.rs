use crate::game::{
    controllers::{ControllerID, Controller, Controllers},
    event::{on_controller_selection_changed::OnControllerSelectionChanged, EventData},
    Game
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

    /// activates on every change to a controller's value (including clienside edits a
    /// player never submits). Submitted actions are already logged in
    /// `OnControllerInputReceived`/`OnValidatedControllerInputReceived`
    fn log(&self, _game: &Game, _fold: &()) -> Option<serde_json::Value> {
        None
    }
}
impl OnControllerChanged{
    pub fn new(id: ControllerID, old: Option<Controller>, new: Option<Controller>)->(Self, ()){
        (Self{id, old, new}, ())
    }
}