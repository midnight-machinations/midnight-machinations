use serde::Serialize;
use crate::game::components::night_visits::Visits;
use crate::game::controllers::AvailableBooleanSelection;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::graves::grave::{Grave, GraveDeathCause, GraveInformation, GraveKiller};
use crate::game::components::graves::grave_reference::GraveReference;
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::abilities_component::ability_id::AbilityID;

use crate::game::role::BooleanSelection;
use crate::game::visit::Visit;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, Role, RoleState, RoleStateTrait};

#[derive(PartialEq, Eq, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Martyr {
    pub state: MartyrState
}


#[derive(PartialEq, Eq, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum MartyrState {
    Won,
    StillPlaying {
        bullets: u8
    },
    Ascension
}

impl Default for Martyr {
    fn default() -> Self {
        Self{
            state: MartyrState::StillPlaying { bullets: 3 }
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Martyr {
    // More information is being sent than needed by the client.
    // This should be fixed later
    type ClientAbilityState = Martyr;
    fn new_state(game: &mut Game) -> Self {
        Self{
            state: MartyrState::StillPlaying { bullets: crate::game::role::common_role::standard_charges(game) }
        }
    }
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Kill {return}
        let MartyrState::StillPlaying { bullets } = self.state else {return};
        if bullets == 0 {return}
        if let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Martyr) {

            self.state = MartyrState::StillPlaying { bullets: bullets.saturating_sub(1) };

            if target_ref == actor_ref {
                if target_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Suicide, AttackPower::Basic, true) {
                    self.state = MartyrState::Won;
                }
            } else {
                target_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Martyr), AttackPower::Basic, true);
            }
        };

        actor_ref.edit_role_ability_helper(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Martyr, 0))
            .available_selection(AvailableBooleanSelection)
            .night_typical(actor_ref)
            .add_grayed_out_condition(
                game.day_number() <= 1 || match self.state {
                    MartyrState::StillPlaying { bullets } => bullets == 0,
                    _ => true
                }
            )
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        let Some(BooleanSelection(true)) = ControllerID::role(actor_ref, Role::Martyr, 0).get_boolean_selection(game) else {return Vec::new()};
        vec![Visit::new_role(actor_ref, actor_ref, true, Role::Martyr, 0)]
    }
    fn on_phase_start(self,  game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        if phase == PhaseType::Obituary && matches!(self.state, MartyrState::StillPlaying {..}) {
            game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::MartyrFailed);
        }

        if phase == PhaseType::Obituary && actor_ref.alive(game) && matches!(self.state, MartyrState::StillPlaying { bullets: 0 }) {
            actor_ref.die_and_add_grave(game, Grave::from_player_suicide(game, actor_ref));
        }
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Martyr) || fold.cancelled {return}
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::MartyrRevealed { martyr: actor_ref });
        for player in PlayerReference::all_players(game){
            player.reveal_players_role(game, actor_ref);
        }
        
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference) {
        let left_town = GraveReference::all_graves(game).any(|grave| 
            grave.deref(game).player == dead_player_ref &&
            if let GraveInformation::Normal { death_cause, .. } = &grave.deref(game).information {
                *death_cause == GraveDeathCause::Ascension
            } else {false}
        );

        if dead_player_ref == actor_ref && !left_town {
            game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::MartyrWon);
            
            for player in PlayerReference::all_players(game) {
                if player == actor_ref {continue}
                if !player.alive(game) {continue}
                if player.normal_defense(game).can_block(AttackPower::ProtectionPiercing) {continue}
                player.die_and_add_grave(game, Grave::from_player_suicide(game, player));
            }
    
            actor_ref.edit_role_ability_helper(game, RoleState::Martyr(Martyr {
                state: MartyrState::Won
            }));
        }
    }
}

impl Martyr{
    pub fn won(&self)->bool{
        self.state == MartyrState::Won
    }
}
