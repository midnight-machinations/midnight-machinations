use serde::Serialize;
use crate::game::{components::attack::night_attack::NightAttack, prelude::*};


#[derive(Clone, Debug, Default, Serialize)]
pub struct Zealot;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Zealot {
    type ClientAbilityState = Zealot;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if !matches!(priority, OnMidnightPriority::Kill) {return}
        let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::Zealot) else {return};
        if !Cult::can_kill_tonight(game) {return}

        NightAttack::new()
            .attackers([actor_ref])
            .grave_killer(Role::Zealot)
            .attack(game, midnight_variables, target);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Zealot, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1 || !Cult::can_kill_tonight(game))
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Zealot, 0),
            true
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Cult
        ].into_iter().collect()
    }
}
