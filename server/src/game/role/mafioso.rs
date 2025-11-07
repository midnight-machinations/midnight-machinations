use serde::Serialize;
use crate::game::{components::attack::night_attack::NightAttack, prelude::*};


#[derive(Debug, Clone, Serialize, Default)]
pub struct Mafioso;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Mafioso {
    type ClientAbilityState = Mafioso;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Kill {return}
        if game.day_number() == 1 {return}
        let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Mafioso) else {return};
        NightAttack::new()
            .attackers([actor_ref])
            .grave_killer(RoleSet::Mafia)
            .attack(game, midnight_variables, target_ref);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Mafioso, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Mafioso, 0),
            true
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}