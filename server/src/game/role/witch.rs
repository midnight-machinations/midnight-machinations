use serde::Serialize;
use crate::game::prelude::*;


#[derive(Clone, Debug, Default)]
pub struct Witch{
    currently_used_player: Option<PlayerReference> 
}

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Witch {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if let Some(currently_used_player) = Possession::possess_night_action_and_steal_messages(
            actor_ref,
            game,
            midnight_variables,
            priority,
            self.currently_used_player,
            VisitTag::Ability { ability: *id, id: 0 },
            VisitTag::Ability { ability: *id, id: 1 },
        ){
            actor_ref.edit_role_ability_helper(game, Witch{
                currently_used_player: Some(currently_used_player)
            })
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Witch, 0))
            .available_selection(AvailableTwoPlayerOptionSelection {
                available_first_players: PlayerReference::all_players(game)
                    .filter(|p|p.alive(game))
                    .filter(|p|*p != actor_ref)
                    .collect(),
                available_second_players: PlayerReference::all_players(game)
                    .filter(|p|p.alive(game))
                    .collect(),
                can_choose_duplicates: true,
                can_choose_none: true
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn create_visits_initialize_night(self, game: &Game, _id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits_possession(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Witch, 0),
            VisitTag::Ability { ability: AbilityID::Role { role: Role::Witch, player: actor_ref }, id: 0 },
            VisitTag::Ability { ability: AbilityID::Role { role: Role::Witch, player: actor_ref }, id: 1 }
        )
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        if phase != PhaseType::Night {return}
        actor_ref.edit_role_ability_helper(game, Witch { currently_used_player: None });
    }
    fn on_player_roleblocked(self, _game: &mut Game, _id: &AbilityID, _event: &OnPlayerRoleblocked, _fold: &mut OnMidnightFold, _priority: ()) {}
}
impl GetClientAbilityState<ClientRoleState> for Witch {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}