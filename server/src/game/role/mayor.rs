
use serde::Serialize;

use crate::game::controllers::AvailableUnitSelection;
use crate::game::attack_power::DefensePower;
use crate::game::event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority};
use crate::game::components::enfranchise::Enfranchise;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;


use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, GetClientAbilityState, Role, RoleStateTrait};

#[derive(Clone, Debug, Default)]
pub struct Mayor;

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Mayor {
    type ClientAbilityState = ClientRoleState;
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: super::ControllerInput) {
        if actor_ref != input_player {return;}
        if ability_input.id() != ControllerID::role(actor_ref, Role::Mayor, 0) {
            return;
        }

        Enfranchise::enfranchise(game, actor_ref);
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _new: super::RoleState, _old: super::RoleState) {
        if actor_ref != player {return;}
        Enfranchise::unenfranchise(game, actor_ref);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Mayor, 0))
            .available_selection(AvailableUnitSelection)
            .add_grayed_out_condition(
                actor_ref.ability_deactivated_from_death(game) ||
                Enfranchise::enfranchised(game, actor_ref) || 
                PhaseType::Night == game.current_phase().phase() ||
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
        ) && Enfranchise::enfranchised(game, actor_ref) {
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