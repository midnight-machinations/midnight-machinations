
use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::controllers::AvailablePlayerListSelection;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::{PhaseType, PhaseState};
use crate::game::player::PlayerReference;

use crate::game::verdict::Verdict;

use crate::game::Game;
use super::{
    ControllerID, ControllerParametersMap, GetClientRoleState, Role, RoleStateImpl
};

#[derive(Clone, Debug, Default)]
pub struct Jester {
    executed_yesterday: bool,
    won: bool,
}

#[derive(Clone, Serialize, Debug)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Jester {
    type ClientRoleState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::TopPriority {return;}
        if actor_ref.alive(game) {return;}
        if !self.executed_yesterday {return}

        let target_ref = if let Some(target_ref) = ControllerID::role(actor_ref, Role::Jester, 0)
            .get_player_list_selection(game)
            .and_then(|s|s.0.first())
        {
            *target_ref
        }else{
            let all_killable_players: Vec<PlayerReference> = PlayerReference::all_players(game)
                .filter(|player_ref|{
                    player_ref.alive(game) &&
                    *player_ref != actor_ref &&
                    player_ref.verdict(game) != Verdict::Innocent
                })
                .collect();

            let Some(target_ref) = all_killable_players
                .choose(&mut rand::rng()) else {return};
            
            *target_ref
        };
        
        
        target_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables,
            GraveKiller::Role(super::Role::Jester), AttackPower::ProtectionPiercing, true
        );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Jester, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|p| *p != actor_ref)
                    .filter(|player| 
                        player.alive(game) &&
                        player.verdict(game) != Verdict::Innocent
                    )
                    .collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .add_grayed_out_condition(!self.executed_yesterday)
            .reset_on_phase_start(PhaseType::Obituary)
            .allow_players([actor_ref])
            .build_map()
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        match game.current_phase() {
            PhaseState::FinalWords { player_on_trial } => {
                if *player_on_trial == actor_ref {
                    actor_ref.set_role_state(game, Jester { 
                        executed_yesterday: true,
                        won: true
                    });
                }
            }
            PhaseState::Obituary { .. } => {
                actor_ref.set_role_state(game, Jester { 
                    executed_yesterday: false,
                    ..self
                });
            }
            _ => {}
        }
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
        if 
            actor_ref == dead_player_ref && 
            game.current_phase().phase() == PhaseType::FinalWords
        {
            game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::JesterWon);
        }
    }
}
impl GetClientRoleState<ClientRoleState> for Jester {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl Jester {
    pub fn won(&self) -> bool {
        self.won
    }
}