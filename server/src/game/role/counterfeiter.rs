use serde::Serialize;

use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::components::night_visits::Visits;
use crate::game::controllers::{AvailableIntegerSelection, AvailableStringSelection, RoleListSelection};
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::ChatMessageVariant;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::role_list::RoleSet;
use crate::game::visit::Visit;

use crate::game::Game;
use super::godfather::Godfather;
use super::{
    ControllerID, ControllerParametersMap, GetClientAbilityState, IntegerSelection, Role,
    RoleStateTrait
};


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counterfeiter{
    pub forges_remaining: u8,
    pub forged_ref: Option<PlayerReference>
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState{
    pub forges_remaining: u8
}
impl Default for Counterfeiter {
    fn default() -> Self {
        Counterfeiter {
            forges_remaining: 3,
            forged_ref: None
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Counterfeiter {
    type ClientAbilityState = ClientRoleState;
    fn new_state(game: &Game) -> Self {
        Self{
            forges_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if game.day_number() <= 1 {return}

        match priority {
            OnMidnightPriority::Deception => {
                if self.forges_remaining == 0 || chose_no_forge(game, actor_ref) {return}
                let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Counterfeiter) else {return};

                let fake_role = ControllerID::role(actor_ref, Role::Counterfeiter, 1)
                    .get_role_list_selection(game)
                    .and_then(|p| p.0.first().copied());
                target_ref.set_night_grave_role(midnight_variables, fake_role);

                let fake_alibi = ControllerID::role(actor_ref, Role::Counterfeiter, 2)
                    .get_string_selection(game)
                    .map(|s|s.0.clone())
                    .unwrap_or("".to_string());

                target_ref.set_night_grave_will(midnight_variables, fake_alibi);

                actor_ref.edit_role_ability_helper(game, Counterfeiter { 
                    forges_remaining: self.forges_remaining.saturating_sub(1), 
                    forged_ref: Some(target_ref)
                });
            },
            OnMidnightPriority::Kill => {
                let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Counterfeiter) else {return};
                target_ref.try_night_kill_single_attacker(
                    actor_ref, game, midnight_variables, GraveKiller::RoleSet(RoleSet::Mafia), AttackPower::Basic, false
                );
            },
            OnMidnightPriority::Investigative => {
                if let Some(forged_ref) = self.forged_ref && forged_ref.night_died(midnight_variables) {
                    actor_ref.push_night_message(midnight_variables, ChatMessageVariant::PlayerRoleAndAlibi{
                        player: forged_ref,
                        role: forged_ref.role(game),
                        will: forged_ref.alibi(game).to_string(),
                    });
                }
            },
            _ => {}
        }
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Counterfeiter, 0),
            true
        )
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Counterfeiter, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map(),
            // Role
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Counterfeiter, 1))
                .single_role_selection_typical(game, |_|true)
                .default_selection(RoleListSelection(vec!(Role::Counterfeiter)))
                .add_grayed_out_condition(
                    self.forges_remaining == 0 ||
                    actor_ref.ability_deactivated_from_death(game)
                )
                .allow_players([actor_ref])
                .build_map(),
            // Alibi
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Counterfeiter, 2))
                .available_selection(AvailableStringSelection)
                .add_grayed_out_condition(
                    self.forges_remaining == 0 ||
                    actor_ref.ability_deactivated_from_death(game)
                )
                .allow_players([actor_ref])
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Counterfeiter, 3))
                .available_selection(AvailableIntegerSelection {
                    min: 0, max: if self.forges_remaining > 0 {1} else {0}
                })
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.edit_role_ability_helper(game, Counterfeiter{
            forged_ref: None,
            ..self
        });
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
        Godfather::pass_role_state_down(game, actor_ref, dead_player_ref, self);
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}
impl GetClientAbilityState<ClientRoleState> for Counterfeiter {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState{
            forges_remaining: self.forges_remaining
        }
    }
}

fn chose_no_forge(game: &Game, actor_ref: PlayerReference)->bool{
    if let Some(IntegerSelection(x)) = ControllerID::role(actor_ref, Role::Counterfeiter, 3).get_integer_selection(game){
        *x == 0
    }else{
        true
    }
}