mod controller_parameters_map; pub use controller_parameters_map::*;
pub mod builder;

use serde::{Deserialize, Serialize};

use crate::{game::{controllers::{controller_selection::ControllerSelection, AvailableControllerSelection}, phase::PhaseType, player::PlayerReference, Game}, vec_set::VecSet};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerParameters{
    available: AvailableControllerSelection,
    grayed_out: bool,
    reset_on_phase_start: Option<PhaseType>,
    dont_save: bool,
    default_selection: ControllerSelection,
    allowed_players: VecSet<PlayerReference>
}
impl ControllerParameters{
    pub fn new(
        game: &Game,
        available: AvailableControllerSelection,
        grayed_out: bool,
        reset_on_phase_start: Option<PhaseType>,
        dont_save: bool,
        default_selection: ControllerSelection,
        allowed_players: VecSet<PlayerReference>
    )->Option<Self>{
        if available.validate_selection(game, &default_selection) {
            Some(
                Self{
                    available,
                    grayed_out,
                    reset_on_phase_start,
                    default_selection,
                    dont_save,
                    allowed_players
                }
            )
        }else{
            None
        }
    }
    
    pub fn validate_selection(&self, game: &Game, selection: &ControllerSelection)->bool{
        self.available.validate_selection(game, selection)
    }
    pub fn default_selection(&self)->&ControllerSelection{
        &self.default_selection
    }
    pub fn grayed_out(&self)->bool{
        self.grayed_out
    }
    pub fn dont_save(&self)->bool{
        self.dont_save
    }
    pub fn set_grayed_out(&mut self, grayed_out: bool){
        self.grayed_out = grayed_out;
    }
    pub fn reset_on_phase_start(&self)->Option<PhaseType>{
        self.reset_on_phase_start
    }
    pub fn allowed_players(&self)->&VecSet<PlayerReference>{
        &self.allowed_players
    }
}