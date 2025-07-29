mod controller_parameters_map; pub use controller_parameters_map::*;
pub mod builder;

use crate::{game::{ability_input::{ability_selection::AbilitySelection, AvailableAbilitySelection}, phase::PhaseType, player::PlayerReference, Game}, vec_set::VecSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControllerParameters{
    pub (super) available: AvailableAbilitySelection,
    grayed_out: bool,
    reset_on_phase_start: Option<PhaseStartCondition>,
    dont_save: bool,
    default_selection: AbilitySelection,
    allowed_players: VecSet<PlayerReference>
}
impl ControllerParameters{
    pub fn new(
        game: &Game,
        available: AvailableAbilitySelection,
        grayed_out: bool,
        reset_on_phase_start: Option<PhaseStartCondition>,
        dont_save: bool,
        default_selection: AbilitySelection,
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
    
    pub fn validate_selection(&self, game: &Game, selection: &AbilitySelection)->bool{
        self.available.validate_selection(game, selection)
    }
    pub fn default_selection(&self)->&AbilitySelection{
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
    pub fn reset_on_phase_start(&self)->Option<PhaseStartCondition>{
        self.reset_on_phase_start.clone()
    }
    pub fn allowed_players(&self)->&VecSet<PlayerReference>{
        &self.allowed_players
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseStartCondition {
    pub phase: PhaseType,
    pub condition: fn(&Game) -> bool
}