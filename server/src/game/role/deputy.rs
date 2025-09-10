
use serde::Serialize;

use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::graves::grave::{Grave, GraveDeathCause, GraveInformation, GraveKiller};
use crate::game::game_conclusion::GameConclusion;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;


use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, PlayerListSelection, Role, RoleStateTrait};




#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deputy {
    bullets_remaining: u8,
}

impl Default for Deputy {
    fn default() -> Self {
        Self { bullets_remaining: 1 }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Deputy {
    type ClientAbilityState = Deputy;
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: super::ControllerInput) {
        
        if actor_ref != input_player {return;}
        let Some(PlayerListSelection(target_ref)) = ability_input.get_player_list_selection_if_id(
            ControllerID::role(actor_ref, Role::Deputy, 0)
        )else{return};
        let Some(shot) = target_ref.first() else {return};
        
        
        shot.add_private_chat_message(game, ChatMessageVariant::DeputyShotYou);
        if shot.normal_defense(game).can_block(AttackPower::Basic) {
            shot.add_private_chat_message(game, ChatMessageVariant::YouSurvivedAttack);
            actor_ref.add_private_chat_message(game, ChatMessageVariant::SomeoneSurvivedYourAttack);

        }else{
            game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::DeputyKilled{shot: *shot});
            
            
            let mut grave = Grave::from_player_lynch(game, *shot);
            if let GraveInformation::Normal{death_cause, ..} = &mut grave.information {
                *death_cause = GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Deputy)]);
            }
            shot.die_and_add_grave(game, grave);
            

            if shot.win_condition(game).is_loyalist_for(GameConclusion::Town) {
                actor_ref.die_and_add_grave(game, Grave::from_player_leave_town(game, actor_ref));
            }
        }

        actor_ref.set_role_state(game, Deputy{bullets_remaining:self.bullets_remaining.saturating_sub(1)});
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Deputy, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .add_grayed_out_condition(
                actor_ref.ability_deactivated_from_death(game) ||
                self.bullets_remaining == 0 || 
                game.day_number() <= 1 || 
                !matches!(game.current_phase().phase(), PhaseType::Discussion | PhaseType::Nomination | PhaseType::Adjournment | PhaseType::Dusk)
            )
            .dont_save()
            .allow_players([actor_ref])
            .build_map()
    }
}