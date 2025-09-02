use serde::Serialize;
use crate::game::controllers::{AvailableRoleListSelection, ControllerID, RoleListSelection};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::role::common_role;
use crate::game::visit::Visit;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::Game;
use super::{ControllerParametersMap, GetClientRoleState, Role, RoleStateImpl};

#[derive(Clone, Debug)]
pub struct Steward {
    self_heals_remaining: u8,
    target_healed_refs: Vec<PlayerReference>,
    previous_input: RoleListSelection
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState {
    steward_protects_remaining: u8,
    previous_role_chosen: RoleListSelection
}

impl Default for Steward {
    fn default() -> Self {
        Self { 
            self_heals_remaining: 1,
            target_healed_refs: vec![],
            previous_input: RoleListSelection(vec![])
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Steward {
    type ClientRoleState = ClientRoleState;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {

        if actor_ref.night_blocked(midnight_variables) {return}
        if actor_ref.ability_deactivated_from_death(game) {return}

        match priority {
            OnMidnightPriority::Heal => {
                let Some(RoleListSelection(roles)) = ControllerID::role(actor_ref, Role::Steward, 0)
                    .get_role_list_selection(game)
                    .cloned()
                else {return};

                self.target_healed_refs = vec![];
                
                for &role in roles.iter(){
                    for player in PlayerReference::all_players(game){
                        if role != player.role(game) {continue;}
    
                        player.increase_defense_to(game, midnight_variables, DefensePower::Protected);
                        self.target_healed_refs.push(player);
                    }
                }

                self.self_heals_remaining = self.self_heals_remaining
                    .saturating_sub(
                        if roles.contains(&Role::Steward){1}else{0}
                    );
                
                actor_ref.set_role_state(game, Steward{
                    previous_input: RoleListSelection(
                        vec![roles.first(), roles.get(1)]
                            .into_iter()
                            .flatten()
                            .copied()
                            .collect()
                    ), //updates here
                    ..self
                });
            }
            OnMidnightPriority::DeleteMessages => {
                for target_healed_ref in self.target_healed_refs{
                    if target_healed_ref.night_attacked(midnight_variables){

                        target_healed_ref.set_night_messages(midnight_variables, 
                            target_healed_ref.night_messages(midnight_variables)
                                .iter()
                                .filter(|m|
                                    !matches!(m,ChatMessageVariant::YouWereGuarded|ChatMessageVariant::YouSurvivedAttack)
                                )
                                .cloned()
                                .collect()
                        );
                    }
                }
            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Steward, 0))
            .available_selection(AvailableRoleListSelection{
                available_roles: game.settings.enabled_roles.clone()
                    .into_iter()
                    .filter(|role|self.self_heals_remaining > 0 || role != &Role::Steward)
                    .filter(|role|!self.previous_input.0.contains(role))
                    .collect(),
                can_choose_duplicates: false,
                max_roles: Some(2)
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        let mut out = common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Steward, 0),
            false
        );
        out.iter_mut().for_each(|v|{
            v.transport_immune = true;
            v.indirect = true;
        });
        out
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.set_role_state(game, Steward{
            self_heals_remaining: self.self_heals_remaining,
            target_healed_refs: vec![],
            previous_input: self.previous_input
        });
    }
}
impl GetClientRoleState<ClientRoleState> for Steward {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            steward_protects_remaining: self.self_heals_remaining,
            previous_role_chosen: self.previous_input
        }
    }
}