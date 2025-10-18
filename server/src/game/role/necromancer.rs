use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::role::godfather::Godfather;
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;
use crate::game::role_list::RoleSet;
use crate::game::visit::Visit;
use crate::game::Game;
use super::{
    common_role, ControllerID,
    ControllerParametersMap, Role, RoleStateTrait
};


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Necromancer;

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;

impl RoleStateTrait for Necromancer {
    type ClientAbilityState = Necromancer;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        Godfather::night_kill_ability(game, midnight_variables, actor_ref, priority, Role::Necromancer);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Necromancer, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Necromancer, 0),
            true
        )
    }
    fn on_player_roleblocked(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, invisible: bool) {
        common_role::on_player_roleblocked(midnight_variables, actor_ref, player);
        if player != actor_ref {return}
        for seanced in Necromancer::get_seanced_targets(game, actor_ref) {
            seanced.roleblock(game, midnight_variables, invisible);
        }
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
impl Necromancer {
    pub fn get_seanced_targets(game: &Game, actor_ref: PlayerReference) -> Vec<PlayerReference> {
        if !actor_ref.alive(game) {return vec![]}
        if !(AbilityID::Role { role: Role::Necromancer, player: actor_ref }.exists(game)) {return vec![];}
        PlayerReference::all_players(game)
            .filter(|player|
                !player.alive(game) &&
                InsiderGroupID::in_same_group(game, actor_ref, *player) &&
                !RoleSet::MafiaKilling.get_roles().contains(&player.role(game)) &&
                !RoleSet::Fiends.get_roles().contains(&player.role(game)) &&
                !RoleSet::Cult.get_roles().contains(&player.role(game))
            )
            .collect()
    }
}