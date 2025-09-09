use serde::Serialize;

use crate::game::controllers::*;
use crate::game::attack_power::DefensePower;
use crate::game::components::graves::grave::GraveInformation;
use crate::game::components::graves::grave_reference::GraveReference;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::godfather::Godfather;
use super::{Role, RoleStateTrait};


#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Impostor{
    pub backup: Option<PlayerReference>
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Impostor {
    type ClientAbilityState = Impostor;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        Godfather::night_kill_ability(game, midnight_variables, actor_ref, priority);
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Impostor, 0),
            true
        )
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Impostor, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Impostor, 1))
                .single_role_selection_typical(game, |_|true)
                .default_selection(RoleListSelection(vec!(Role::Impostor)))
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn on_grave_added(self, game: &mut Game, actor_ref: PlayerReference, grave: GraveReference) {
        let Some(RoleListSelection(roles)) = ControllerID::role(actor_ref, Role::Impostor, 1)
            .get_role_list_selection(game).cloned() else {return};
        let Some(role) = roles.first().copied() else {return};
        
        
        if grave.deref(game).player == actor_ref {
            let grave = grave.deref_mut(game);
            grave.information = match grave.information.clone() {
                GraveInformation::Obscured => GraveInformation::Obscured,
                GraveInformation::Normal {will, death_cause, death_notes, .. } => {
                    GraveInformation::Normal { role, will, death_cause, death_notes } 
                },
            };
        }
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
        Godfather::pass_role_state_down(game, actor_ref, dead_player_ref, self);
    }
     fn default_insider_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}