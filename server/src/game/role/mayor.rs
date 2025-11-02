
use serde::Serialize;
use crate::game::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct Mayor;

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Mayor {
    type ClientAbilityState = ClientRoleState;
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: super::ControllerInput) {
        if BlockedComponent::blocked(game, actor_ref) {return}
        if actor_ref != input_player {return;}
        if ability_input.id() != ControllerID::role(actor_ref, Role::Mayor, 0) {
            return;
        }

        EnfranchiseComponent::enfranchise(game, actor_ref, 3);
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Mayor) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        EnfranchiseComponent::unenfranchise(game, actor_ref);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Mayor, 0))
            .available_selection(AvailableUnitSelection)
            .add_grayed_out_condition(
                actor_ref.ability_deactivated_from_death(game) ||
                EnfranchiseComponent::enfranchised(game, actor_ref) || 
                PhaseType::Night == game.current_phase().phase() ||
                BlockedComponent::blocked(game, actor_ref) ||
                PhaseType::Briefing == game.current_phase().phase()
            )
            .dont_save()
            .allow_players([actor_ref])
            .build_map()
    }
    fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        if priority == WhisperPriority::Cancel && (
            event.sender == actor_ref || 
            event.receiver == actor_ref
        ) && EnfranchiseComponent::enfranchised(game, actor_ref) {
            fold.cancelled = true;
            fold.hide_broadcast = true;
        }
    }
}
impl GetClientAbilityState<ClientRoleState> for Mayor {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}