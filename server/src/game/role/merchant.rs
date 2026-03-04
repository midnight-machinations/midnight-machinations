use serde::Serialize;
use crate::{game::prelude::*, vec_set::VecSet};

#[derive(Clone, Debug, Default)]
pub struct Merchant {
    available: VecSet<MerchantOption>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MerchantOption{
    Polymorph,

    Scarecrow,
    Witch,
    // Hunter,
    PoisonVial,
}

#[derive(Clone, Serialize, Debug)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Merchant {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::default()
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){

    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){

    }
}
impl GetClientAbilityState<ClientRoleState> for Merchant {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl Merchant {
    pub fn won(&self) -> bool {
        self.available.count() == 0
    }
}