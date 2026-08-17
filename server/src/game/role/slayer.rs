
use serde::Serialize;
use crate::game::components::attack::normal_attack::Attack;
use crate::game::prelude::*;
use crate::vec_set;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slayer {
    bullets_remaining: u8,
}

impl Default for Slayer {
    fn default() -> Self {
        Self { bullets_remaining: 1 }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Slayer {
    type ClientAbilityState = Slayer;
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: super::ControllerInput) {
        if BlockedComponent::blocked(game, actor_ref) {return}
        if actor_ref != input_player {return;}
        let Some(PlayerListSelection(target_ref)) = ability_input.get_player_list_selection_if_id(
            ControllerID::role(actor_ref, Role::Slayer, 0)
        )else{return};
        let Some(shot) = target_ref.first() else {return};


        if !shot.win_condition(game).is_loyalist_for(GameConclusion::Town) {
            let mut grave = Grave::from_player_lynch(game, *shot);
            if let GraveInformation::Normal{death_causes: death_cause, ..} = &mut grave.information {
                *death_cause = vec![GraveDeathCause::Role(Role::Slayer)];
            }

            let killed = Attack::new_attack(
                game,
                true,
                *shot,
                vec_set!(actor_ref),
                AttackPower::ArmorPiercing,
                grave
            );
            
            if killed {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::DeputyKilled{shot: *shot});
            }
        }
        

        actor_ref.edit_role_ability_helper(game, Slayer{bullets_remaining:self.bullets_remaining.saturating_sub(1)});
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Slayer, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .add_grayed_out_condition(
                actor_ref.ability_deactivated_from_death(game) ||
                self.bullets_remaining == 0 || 
                game.day_number() <= 1 || 
                BlockedComponent::blocked(game, actor_ref) ||
                !matches!(game.current_phase().phase(), PhaseType::Discussion | PhaseType::Nomination | PhaseType::Adjournment | PhaseType::Dusk)
            )
            .dont_save()
            .allow_players([actor_ref])
            .build_map()
    }
}