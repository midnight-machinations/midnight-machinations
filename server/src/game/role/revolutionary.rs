
use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::game::abilities::role_abilities::RoleAbility;
use crate::game::abilities_component::ability::Ability;
use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::graves::grave::Grave;
use crate::game::components::tags::{TagSetID, Tags};
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority};
use crate::game::phase::{PhaseState, PhaseType};
use crate::game::player::PlayerReference;
use crate::game::role::RoleState;
use crate::game::role_list::RoleSet;
use crate::game::Game;
use super::jester::Jester;
use super::{GetClientAbilityState, Role, RoleStateTrait};


#[derive(Clone, Debug, Default)]
pub struct Revolutionary {
    target: RevolutionaryTarget,
}

#[derive(Clone, Serialize, Debug)]
pub struct ClientRoleState;

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
pub enum RevolutionaryTarget{
    Target(PlayerReference),
    Won,
}
impl RevolutionaryTarget {
    fn get_target(&self)->Option<PlayerReference>{
        if let Self::Target(p) = self {
            Some(*p)
        }else{
            None
        }
    }
}
impl Default for RevolutionaryTarget {
    fn default() -> Self {
        Self::Won
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Revolutionary {
    type ClientAbilityState = ClientRoleState;
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        if self.target == RevolutionaryTarget::Won || !actor_ref.alive(game){
            return;
        }

        match *game.current_phase() {
            PhaseState::FinalWords { player_on_trial } => {
                if Some(player_on_trial) == self.target.get_target() {
                    game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::RevolutionaryWon);
                    actor_ref.set_role_state(game, RoleState::Revolutionary(Revolutionary { target: RevolutionaryTarget::Won }));
                    actor_ref.die_and_add_grave(game, Grave::from_player_leave_town(game, actor_ref));
                }
            }
            _=>{}
        }
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority){
        if event.id.is_players_role(actor_ref, Role::Revolutionary) {
            match priority {
                OnAbilityCreationPriority::CancelOrEdit => {
                    let Ability::Role(RoleAbility(RoleState::Revolutionary(Revolutionary { target: RevolutionaryTarget::Won }))) = event.ability else {return};
                    if let Some(target) = PlayerReference::all_players(game)
                        .filter(|p|
                            RoleSet::Town
                                .get_roles()
                                .contains(&p.role(game)) &&
                                !Self::CANT_CHOOSE_ROLES.contains(&p.role(game))
                        ).collect::<Vec<PlayerReference>>()
                        .choose(&mut rand::rng())
                    {
                        fold.ability = Ability::Role(RoleAbility(RoleState::Revolutionary(
                            Revolutionary{target: RevolutionaryTarget::Target(*target)}
                        )));
                    }else{
                        fold.cancelled = true;
                        actor_ref.set_role_and_win_condition_and_revealed_group(game, RoleState::Jester(Jester::default()))
                    };
                },
                OnAbilityCreationPriority::SideEffect => {
                    if !fold.cancelled && let RevolutionaryTarget::Target(target) = self.target {
                        Tags::add_viewer(game, TagSetID::RevolutionaryTarget(actor_ref), actor_ref);
                        Tags::add_tag(game, TagSetID::RevolutionaryTarget(actor_ref), target);
                        actor_ref.reveal_players_role(game, target);
                    }
                },
                _ => {}
            }
        }

        
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
        if Some(dead_player_ref) == self.target.get_target() && self.target != RevolutionaryTarget::Won {
            actor_ref.set_role_and_win_condition_and_revealed_group(game, RoleState::Jester(Jester::default()))
        }
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Revolutionary) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        Tags::remove_viewer(game, TagSetID::RevolutionaryTarget(actor_ref), actor_ref);
    }
}
impl GetClientAbilityState<ClientRoleState> for Revolutionary {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl Revolutionary {
    pub fn won(&self)->bool{
        self.target == RevolutionaryTarget::Won
    }
    const CANT_CHOOSE_ROLES: [Role; 5] = [
        Role::Jailor,
        Role::Deputy,    
        Role::Transporter,
        Role::Mayor,
        Role::Reporter,
    ];
}