use serde::{Deserialize, Serialize};

use crate::game::{controllers::{ControllerSelection, ControllerParameters}, phase::PhaseType};



#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Controller{
    pub(super) selection: ControllerSelection,
    pub(super) parameters: ControllerParameters
}
impl Controller{
    pub(super) fn new(selection: ControllerSelection, available_ability_data: ControllerParameters)->Self{
        Self{selection, parameters: available_ability_data}
    }
    pub fn selection(&self)->&ControllerSelection{
        &self.selection
    }
    pub fn reset_on_phase_start(&mut self, phase: PhaseType){
        if let Some(reset_phase) = self.parameters.reset_on_phase_start() && phase == reset_phase{
            self.selection = self.parameters.default_selection().clone();
        }
    }
}