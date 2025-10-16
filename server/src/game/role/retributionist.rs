use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;

use crate::game::controllers::AvailableTwoPlayerOptionSelection;
use crate::game::components::graves::grave_reference::GraveReference;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::possession::Possession;
use crate::game::role_list::RoleSet;
use crate::game::{attack_power::DefensePower, phase::PhaseType};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use super::{
    common_role, ControllerID,
    ControllerParametersMap, GetClientAbilityState, Role, RoleStateTrait
};


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Default)]
pub struct Retributionist { 
    used_bodies: Vec<PlayerReference>, 
    currently_used_player: Option<PlayerReference> 
}

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;

impl RoleStateTrait for Retributionist {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if let Some(currently_used_player) = Possession::possess_night_action(actor_ref, game, midnight_variables, priority, self.currently_used_player, Role::Retributionist){
            let mut used_bodies = self.used_bodies;
            used_bodies.push(currently_used_player);

            actor_ref.edit_role_ability_helper(game, Retributionist{
                used_bodies,
                currently_used_player: Some(currently_used_player)
            })
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Retributionist, 0))
            .available_selection(AvailableTwoPlayerOptionSelection {
                available_first_players: PlayerReference::all_players(game)
                    .filter(|p|!p.alive(game))
                    .filter(|target|
                        GraveReference::all_graves(game).any(|grave|
                            grave.deref(game).player == *target && 
                            if let Some(role) = grave.deref(game).role(){
                                RoleSet::Town.get_roles().contains(&role)
                            }else{false}
                        ))
                    .filter(|target|
                        (self.used_bodies.iter().filter(|p| **p == *target).count() < 2)
                    )
                    .filter(|p|*p != actor_ref)
                    .collect(),
                available_second_players: PlayerReference::all_players(game)
                    .filter(|p|p.alive(game))
                    .collect(),
                can_choose_duplicates: true,
                can_choose_none: true,
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits_possession(
            game, actor_ref, ControllerID::role(actor_ref, Role::Retributionist, 0)
        )
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        if phase == PhaseType::Night {
            actor_ref.edit_role_ability_helper(game, Retributionist { currently_used_player: None, ..self });
        }
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}
impl GetClientAbilityState<ClientRoleState> for Retributionist {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}