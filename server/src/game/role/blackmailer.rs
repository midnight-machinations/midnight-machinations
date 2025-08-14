use serde::Serialize;

use crate::game::ability_input::AvailablePlayerListSelection;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::components::silenced::Silenced;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};


#[derive(Clone, Debug, Serialize, Default)]
pub struct Blackmailer{
    previous: Option<PlayerReference>
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Blackmailer {
    type ClientRoleState = Blackmailer;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return}
        

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first(){
            let target_ref = visit.target;
    
            Silenced::silence_night(game, midnight_variables, target_ref);
            self.previous = Some(target_ref);
        }else{
            self.previous = None;
        }
        actor_ref.set_role_state(game, self);
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