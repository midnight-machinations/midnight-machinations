use serde::Serialize;
use crate::game::controllers::AvailablePlayerListSelection;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority};
use crate::game::phase::PhaseType;
use crate::game::role::common_role;
use crate::game::role::informant::Informant;
use crate::game::{attack_power::DefensePower, player::PlayerReference};

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};


#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cerenovous{
    pub currently_brained: Option<PlayerReference>,
    previous: Option<PlayerReference>,
    charges: u8,
    blocked: bool
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Cerenovous {
    type ClientAbilityState = Cerenovous;
    fn new_state(game: &Game) -> Self {
        Self{
            charges: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return}

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first() {
            if self.charges != 0 {
                let target_ref = visit.target;
                
                target_ref.push_night_message(midnight_variables, ChatMessageVariant::Brained);
                self.currently_brained = Some(target_ref);
                self.previous = Some(target_ref);
                self.charges = self.charges.saturating_sub(1);
            }else{
                self.previous = None;
            }
        }else{
            self.previous = None;
        }
        actor_ref.set_role_state(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Cerenovous, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|player|
                        !(!player.alive(game) || 
                        *player == actor_ref ||
                        InsiderGroupID::in_same_group(game, actor_ref, *player) ||
                        Some(*player) == self.previous)
                    )
                    .collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .night_typical(actor_ref)
            .add_grayed_out_condition(self.charges == 0)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Cerenovous, 0),
            false
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn on_player_roleblocked(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, _invisible: bool) {
        common_role::on_player_roleblocked(midnight_variables, actor_ref, player);
        if player != actor_ref {return}
        self.blocked = true;
        actor_ref.set_role_state(game, self);
    }
    fn on_visit_wardblocked(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, visit: Visit) {
        common_role::on_visit_wardblocked(midnight_variables, actor_ref, visit);
        if actor_ref != visit.visitor {return};
        self.blocked = true;
        actor_ref.set_role_state(game, self);
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: crate::game::phase::PhaseType) {
        if matches!(phase, PhaseType::Night) {
            self.blocked = false;
            self.currently_brained = None;
            actor_ref.set_role_state(game, self);
        }
    }
    fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        Informant::read_whispers(self.blocked, game, actor_ref, event, fold, priority);
    }
}
