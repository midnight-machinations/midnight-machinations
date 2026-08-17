use serde::Serialize;
use crate::game::components::hide_votes_message::HideVotesMessage;
use crate::game::prelude::*;
use crate::game::Game;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Pyrotechnician;

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Pyrotechnician {
    type ClientAbilityState = Pyrotechnician;
    fn on_midnight(self, _game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::DeleteMessages {return}

        let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::Pyrotechnician) else {return};

        target.set_night_messages(midnight_variables, vec![]);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Pyrotechnician, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Pyrotechnician, 1))
                .available_selection(AvailableBooleanSelection)
                .allow_players(vec![actor_ref])
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, _id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Pyrotechnician, 0),
            false
        )
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, _input_player: PlayerReference, ability_input: ControllerInput) {
        if ability_input.id() == ControllerID::role(actor_ref, Role::Pyrotechnician, 1) {
            HideVotesMessage::update_hidden_votes(game)
        }
    }
}