
use serde::Serialize;

use crate::game::ability_input::{AvailableRoleListSelection, AvailableStringSelection, RoleListSelection};
use crate::game::attack_power::DefensePower;
use crate::game::chat::ChatMessageVariant;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, GetClientRoleState, Role};
use super::{RoleState, RoleStateImpl};


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Forger {
    pub forges_remaining: u8,
    pub forged_ref: Option<PlayerReference>
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState{
    forges_remaining: u8
}

impl Default for Forger {
    fn default() -> Self {
        Forger {
            forges_remaining: 3,
            forged_ref: None,
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Forger {
    type ClientRoleState = ClientRoleState;
    fn new_state(game: &Game) -> Self {
        Self{
            forges_remaining: game.num_players().div_ceil(5),
            ..Self::default()
        }
    }
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if self.forges_remaining == 0 {return}

        match priority {
            OnMidnightPriority::Deception=>{
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(visit) = actor_visits.first() else{return};

                let target_ref = visit.target;

                let fake_role = ControllerID::role(actor_ref, Role::Forger, 1)
                    .get_role_list_selection(game)
                    .and_then(|p| p.0.first().copied());

                target_ref.set_night_grave_role(midnight_variables, fake_role);


                let fake_alibi = ControllerID::role(actor_ref, Role::Forger, 2)
                    .get_string_selection(game)
                    .map(|s|s.0.clone())
                    .unwrap_or("".to_string());

                target_ref.set_night_grave_will(midnight_variables, fake_alibi);

                actor_ref.set_role_state(game, Forger { 
                    forges_remaining: self.forges_remaining.saturating_sub(1), 
                    forged_ref: Some(target_ref),
                });
            },
            OnMidnightPriority::Investigative=>{
                if let Some(forged_ref) = self.forged_ref {
                    if forged_ref.night_died(midnight_variables) {
                        actor_ref.push_night_message(midnight_variables, ChatMessageVariant::PlayerRoleAndAlibi{
                            player: forged_ref,
                            role: forged_ref.role(game),
                            will: forged_ref.alibi(game).to_string(),
                        });
                    }
                }
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            // Player
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Forger, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .add_grayed_out_condition(self.forges_remaining == 0)
                .build_map(),
            // Role
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Forger, 1))
                .available_selection(AvailableRoleListSelection{
                    available_roles: Role::values(),
                    can_choose_duplicates: false,
                    max_roles: Some(1)
                })
                .default_selection(RoleListSelection(vec!(Role::Forger)))
                .add_grayed_out_condition(
                    self.forges_remaining == 0 ||
                    actor_ref.ability_deactivated_from_death(game)
                )
                .allow_players([actor_ref])
                .build_map(),
            // Alibi
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Forger, 2))
                .available_selection(AvailableStringSelection)
                .add_grayed_out_condition(
                    self.forges_remaining == 0 ||
                    actor_ref.ability_deactivated_from_death(game)
                )
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Forger, 0),
            false
        )
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.set_role_state(game, RoleState::Forger(Forger{
            forged_ref: None,
            ..self
        }));
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}
impl GetClientRoleState<ClientRoleState> for Forger {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            forges_remaining: self.forges_remaining,
        }
    }
}