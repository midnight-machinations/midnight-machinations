use rand::seq::SliceRandom as _;
use serde::{Deserialize, Serialize};

use crate::game::controllers::AvailableBooleanSelection;
use crate::game::attack_power::AttackPower;
use crate::game::components::night_visits::{NightVisitsIterator as _, Visits};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::phase::PhaseType;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::player::PlayerReference;

use crate::game::visit::{Visit, VisitTag};

use crate::game::Game;
use super::{common_role, BooleanSelection, ControllerID, ControllerParametersMap, GetClientAbilityState, Role, RoleState, RoleStateTrait};

#[derive(Default, Clone, Debug)]
pub struct Engineer {
    pub trap: Trap
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState {
    trap: ClientTrapState
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum ClientTrapState {
    Dismantled,
    Ready,
    Set
}

#[derive(Default, Clone, Debug)]
pub enum Trap {
    #[default]
    Dismantled,
    Ready,
    Set{target: PlayerReference}
}

impl Trap {
    fn state(&self) -> TrapState {
        match self {
            Trap::Dismantled => TrapState::Dismantled,
            Trap::Ready => TrapState::Ready,
            Trap::Set{..} => TrapState::Set
        }
    }
}
#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum TrapState {
    #[default]
    Dismantled,
    Ready,
    Set
}

//engineer prioritys
//tell player state

//Set trap / ready up / choose to unset and bring to ready
//protect, kill & investigate, dismantle



pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Engineer {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Heal => {
                //upgrade state

                if !actor_ref.night_blocked(midnight_variables) {
                    match self.trap {
                        Trap::Dismantled => {
                            actor_ref.edit_role_ability_helper(game, Engineer {trap: Trap::Ready});
                        },
                        Trap::Ready => {
                            if let Some(visit) = actor_ref.role_night_visits_cloned(midnight_variables).first(){
                                actor_ref.edit_role_ability_helper(game, Engineer {trap: Trap::Set{target: visit.target}});
                            }
                        },
                        Trap::Set { .. } => {
                            if let Some(BooleanSelection(true)) = ControllerID::role(actor_ref, Role::Engineer, 1).get_boolean_selection(game){
                                actor_ref.edit_role_ability_helper(game, Engineer {trap: Trap::Ready});
                            }
                        }
                    }
                }
    
                if let RoleState::Engineer(Engineer{trap: Trap::Set{target, ..}}) = actor_ref.role_state(game).clone(){
                    actor_ref.guard_player(game, midnight_variables, target);
                }
            }
            OnMidnightPriority::Kill => {
                if let Trap::Set { target, .. } = self.trap {
                    for visit in Visits::into_iter(midnight_variables) {
                        if 
                            visit.attack &&
                            visit.target == target &&
                            visit.visitor != actor_ref &&
                            !visit.indirect
                        {
                            visit.visitor.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Engineer), AttackPower::ArmorPiercing, false);
                        }
                    }
                }
            }
            OnMidnightPriority::Investigative => {
                if
                    let Trap::Set { target, .. } = self.trap &&
                    (
                        target.night_attacked(midnight_variables) ||
                        Visits::into_iter(midnight_variables)
                            .with_target(target)
                            .with_direct()
                            .map_visitor()
                            .any(|p|p!=actor_ref)
                    )
                {
                    actor_ref.edit_role_ability_helper(game, RoleState::Engineer(Engineer {trap: Trap::Dismantled}));
                    let mut visit_tags: Vec<VisitTag> = Visits::into_iter(midnight_variables)
                        .with_investigatable()
                        .with_target(target)
                        .filter(|v|v.tag != VisitTag::Role { role: Role::Engineer, id: 0 } || v.visitor != actor_ref)
                        .map_tag()
                        .collect();
                    visit_tags.shuffle(&mut rand::rng());

                    actor_ref.push_night_message(midnight_variables, ChatMessageVariant::SpyBug { visit_tags } );
                }
                
                if let RoleState::Engineer(Engineer { trap }) = actor_ref.role_state(game){
                    actor_ref.push_night_message(midnight_variables, ChatMessageVariant::TrapStateEndOfNight { state: trap.state() });
                }
            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        match self.trap {
            Trap::Ready => {
                ControllerParametersMap::builder(game)
                    .id(ControllerID::role(actor_ref, Role::Engineer, 0))
                    .single_player_selection_typical(actor_ref, false, true)
                    .night_typical(actor_ref)
                    
                    .build_map()
            },
            Trap::Set { .. } => {
                ControllerParametersMap::builder(game)
                    .id(ControllerID::role(actor_ref, Role::Engineer, 1))
                    .available_selection(AvailableBooleanSelection)
                    .night_typical(actor_ref)
                    
                    .build_map()
            }
            _ => {
                ControllerParametersMap::default()
            }
        }
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref, 
            ControllerID::role(actor_ref, Role::Engineer, 0),
            false
        )
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {
                actor_ref.add_private_chat_message(game, ChatMessageVariant::TrapState { state: self.trap.state() });
            }
            _ => {}
        }
    }

}
impl GetClientAbilityState<ClientRoleState> for Engineer {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            trap: match self.trap {
                Trap::Dismantled => ClientTrapState::Dismantled,
                Trap::Ready => ClientTrapState::Ready,
                Trap::Set {..} => ClientTrapState::Set,
            }
        }
    }
}