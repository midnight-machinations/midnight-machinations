
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
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return};
        if let Some(new_target) = Self::random_valid_target(game, self.get_target(), actor_ref) {
            self.set_target(game, actor_ref, new_target);
        } else {
            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::RevolutionaryRefreshFailed);
        };
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        if self.target == RevolutionaryTarget::Won || !actor_ref.alive(game) {
            return;
        }

        match *game.current_phase() {
            PhaseState::FinalWords { player_on_trial } => {
                if Some(player_on_trial) == self.get_target() {
                    game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::RevolutionaryWon);
                    actor_ref.set_role_state(game, RoleState::Revolutionary(Revolutionary { target: RevolutionaryTarget::Won }));
                    actor_ref.die_and_add_grave(game, Grave::from_player_leave_town(game, actor_ref));
                }
            }
            _=>{}
        }
    }
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
        Tags::add_viewer(game, TagSetID::RevolutionaryTarget(actor_ref), actor_ref);
        Self::random_valid_target(game, None, actor_ref)
            .inspect(|t|self.set_target(game, actor_ref, *t));
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
    pub fn random_valid_target(game: &Game, excluding: Option<PlayerReference>, actor_ref: PlayerReference) -> Option<PlayerReference> {
        PlayerReference::all_players(game)
            .filter(|p|
                p.alive(game) &&
                *p != actor_ref &&
                excluding.is_none_or(|e|*p != e) &&
                p.win_condition(game).is_loyalist_for(GameConclusion::Town)
            )
            .collect::<Vec<PlayerReference>>()
            .choose(&mut rand::rng())
            .copied()
    }
    pub fn set_target(self, game: &mut Game, actor_ref: PlayerReference, target: PlayerReference) {
        if let Some(old_target) = self.get_target() {
            Self::conceal_target(game, actor_ref, old_target);
        }
        Tags::add_tag(game, TagSetID::RevolutionaryTarget(actor_ref), target);
        actor_ref.set_role_state(game, RoleState::Revolutionary(Revolutionary{target: RevolutionaryTarget::Target(target)}));
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