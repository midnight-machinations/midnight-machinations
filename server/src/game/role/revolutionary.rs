
use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::ability_input::{AvailableBooleanSelection, ControllerID, ControllerParametersMap};
use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::graves::grave::Grave;
use crate::game::components::tags::{TagSetID, Tags};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::game_conclusion::GameConclusion;
use crate::game::grave::Grave;
use crate::game::phase::{PhaseState, PhaseType};
use crate::game::player::PlayerReference;
use crate::game::role::RoleState;
use crate::game::Game;
use super::{GetClientRoleState, Role, RoleStateImpl};


#[derive(Clone, Debug, Default)]
pub struct Revolutionary {
    target: RevolutionaryTarget,
    old_targets: Vec<PlayerReference>,
}

#[derive(Clone, Serialize, Debug)]
pub struct ClientRoleState;

#[derive(Clone, Serialize, Debug, Default, PartialEq, Eq)]
pub enum RevolutionaryTarget{
    Target(PlayerReference),
    #[default]
    Won,
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateImpl for Revolutionary {
    type ClientRoleState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return};
        if ControllerID::role(actor_ref, Role::Revolutionary, 0)
            .get_boolean_selection(game)
            .is_some_and(|s|s.0) || 
            self.get_target().is_some_and(|t|t==actor_ref)
        {
            if let Some(new_target) = Self::random_valid_target(game, &self.old_targets, actor_ref, self.get_target()) 
            {
                self.set_target(game, actor_ref, new_target);
            }
        }
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        if self.target == RevolutionaryTarget::Won || !actor_ref.alive(game) {
            return;
        }

        match *game.current_phase() {
            PhaseState::FinalWords { player_on_trial } => {
                if Some(player_on_trial) == self.get_target() {
                    game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::RevolutionaryWon);
                    actor_ref.set_role_state(game, RoleState::Revolutionary(Revolutionary { target: RevolutionaryTarget::Won, ..self}));
                    actor_ref.die_and_add_grave(game, Grave::from_player_leave_town(game, actor_ref));
                }
            }
            _=>{}
        }
    }
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
        //it can only get itself as a target in a game where there is no town at the very start,
        //which is not a game a revolutionary should've been in to begin with.
        //if anyone gets converted to town in that scenario, the revolutionary will automatically 
        //swap to targeting them at the end of the next night
        //
        //The other options would be: 
        //  1: to keep the jester behavior in this very weird and specific circumstance but that would've been a 
        //      weird edge case that would confuse people
        //  2: make it automatically win, which would also be a confusing edge case and while I originally thought
        //      it would be the best option, the issue comes where all wildcard games are even worse now because 
        //      everyone just chooses revolutionary and wins unless someone with a lower number than them chooses 
        //      town and having games be decided by player # isn't fun
        self.set_target(game, actor_ref, Self::random_valid_target(game, &[], actor_ref, None).unwrap_or(actor_ref));
        Tags::add_viewer(game, TagSetID::RevolutionaryTarget(actor_ref), actor_ref);
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _new: RoleState, _old: RoleState) {
        if actor_ref != player {return}
        if let RevolutionaryTarget::Target(old_target) = self.target {
            Self::conceal_target(game, actor_ref, old_target);
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Revolutionary, 0))
            .available_selection(AvailableBooleanSelection)
            .add_grayed_out_condition(self.won())
            .allow_players([actor_ref])
            .build_map()
    }
}

impl GetClientRoleState<ClientRoleState> for Revolutionary {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl Revolutionary {
    pub fn won(&self)->bool{
        self.target == RevolutionaryTarget::Won
    }
    pub fn random_valid_target(game: &Game, avoid: &[PlayerReference], actor_ref: PlayerReference, cur_target: Option<PlayerReference>) -> Option<PlayerReference> {
        PlayerReference::all_players(game)
            .filter(|p|
                p.alive(game) &&
                *p != actor_ref &&
                p.win_condition(game).is_loyalist_for(GameConclusion::Town) &&
                !avoid.contains(p) &&
                cur_target.is_none_or(|t|*p!=t)
            )
            .collect::<Vec<PlayerReference>>()
            .choose(&mut rand::rng())
            .copied()
            .or_else(|| 
                if !avoid.is_empty() {
                    Self::random_valid_target(game, &[], actor_ref, cur_target)
                } else {
                    None
                }
            )
    }

    pub fn set_target(self, game: &mut Game, actor_ref: PlayerReference, target: PlayerReference) {
        if let Some(old_target) = self.get_target() {
            Self::conceal_target(game, actor_ref, old_target);
        }
        let mut old_targets = self.old_targets;
        old_targets.push(target);
        Tags::add_tag(game, TagSetID::RevolutionaryTarget(actor_ref), target);
        actor_ref.set_role_state(game, RoleState::Revolutionary(Revolutionary{target: RevolutionaryTarget::Target(target), old_targets}));
        actor_ref.reveal_players_role(game, target);
    }
    pub fn conceal_target(game: &mut Game, actor_ref: PlayerReference, target: PlayerReference) {
        Tags::remove_tag(game, TagSetID::RevolutionaryTarget(actor_ref), target);
        actor_ref.conceal_players_role(game, target);
    }
    pub fn get_target(&self)->Option<PlayerReference>{
        if let RevolutionaryTarget::Target(p) = self.target {
            Some(p)
        }else{
            None
        }
    }
}