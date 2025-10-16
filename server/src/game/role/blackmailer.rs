use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::components::night_visits::Visits;
use crate::game::controllers::AvailablePlayerListSelection;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::components::silenced::Silenced;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};
use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};


#[derive(Clone, Debug, Serialize, Default)]
pub struct Blackmailer{
    previous: Option<PlayerReference>
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Blackmailer {
    type ClientAbilityState = Blackmailer;
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return}
        if let Some(visit) = Visits::default_visit(midnight_variables, actor_ref, Role::Blackmailer) {
            let target_ref = visit.target;
    
            Silenced::silence_night(game, midnight_variables, target_ref);
            self.previous = Some(target_ref);
        }else{
            self.previous = None;
        }
        actor_ref.edit_role_ability_helper(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Blackmailer, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|player|
                        !(!player.alive(game) || 
                        *player == actor_ref ||
                        InsiderGroupID::in_same_group(game, actor_ref, *player) ||
                        Some(*player) == self.previous)
                    )
                    .collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            }).night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Blackmailer, 0),
            false
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}