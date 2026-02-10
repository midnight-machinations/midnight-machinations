use serde::{Deserialize, Serialize};

use crate::{game::{controllers::{ControllerID, ControllerSelection},
    event::{
        on_controller_input_received::OnControllerInputReceived, AsInvokable as _,
        Invokable as _
    },
    player::PlayerReference, Game
}};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ControllerInput{
    pub id: ControllerID, 
    pub selection: ControllerSelection
}
impl ControllerInput{
    pub fn new(id: ControllerID, selection: impl Into<ControllerSelection>)->Self{
        Self{id, selection: selection.into()}
    }
    pub fn id(&self)->ControllerID{
        self.id.clone()
    }
    pub fn selection(&self)->ControllerSelection{
        self.selection.clone()
    }
    pub fn id_and_selection(&self)->(ControllerID, ControllerSelection){
        (self.id(), self.selection())
    }
}
impl ControllerInput{
    pub fn on_client_message(self, game: &mut Game, actor_ref: PlayerReference){
        OnControllerInputReceived::new(actor_ref, self.clone()).as_invokable().invoke(game);
    }
}