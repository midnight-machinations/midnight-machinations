use rand::seq::IteratorRandom;
use serde::Serialize;

use crate::game::ability_input::{AvailablePlayerListSelection, ControllerID};
use crate::game::attack_power::AttackPower;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::role_list::RoleSet;
use crate::game::grave::GraveKiller;

use crate::game::components::win_condition::WinCondition;
use crate::game::{attack_power::DefensePower, player::PlayerReference};

use crate::game::visit::{Visit, VisitTag};

use crate::game::Game;
use super::{
    common_role, ControllerParametersMap, Role, RoleListSelection, 
    RoleStateImpl
};


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reeducator{
    convert_charges_remaining: bool,
}
impl Default for Reeducator{
    fn default() -> Self {
        Self {
            convert_charges_remaining: true,
        }
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Reeducator {
    type ClientRoleState = Reeducator;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Roleblock => {
                if !self.convert_charges_remaining || game.day_number() <= 1 {return}

                let mut converting = false;

                actor_ref.set_night_visits(
                    midnight_variables, 
                    actor_ref
                        .all_night_visits_cloned(midnight_variables)
                        .into_iter()
                        .map(|mut v|{
                            if 
                                !InsiderGroupID::in_same_group(game, actor_ref, v.target) &&
                                v.tag == (VisitTag::Role{role: Role::Reeducator, id: 0})
                            {
                                v.attack = true;
                                converting = true;
                            }
                            v
                        })
                        .collect()
                );

                if converting {
                    for fellow_insider in InsiderGroupID::Mafia.players(game).clone().iter(){
                        fellow_insider.roleblock(game, midnight_variables, true);
                    }
                }
            },
            OnMidnightPriority::Convert => {
                if game.day_number() <= 1 {return};
                let visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(visit) = visits.first() else {return};

                let role = 
                if let Some(role) = ControllerID::role(actor_ref, Role::Reeducator, 1)
                    .get_role_list_selection_first(game)
                {
                    *role
                }else{
                    Reeducator::default_role(game)
                };

                let new_state = role.new_state(game);

                if visit.attack {
                    if self.convert_charges_remaining {
                        if visit.target.night_defense(game, midnight_variables).can_block(AttackPower::ProtectionPiercing) {
                            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::YourConvertFailed);
                            return;
                        }

                        actor_ref.try_night_kill_single_attacker(
                            actor_ref,
                            game,
                            midnight_variables,
                            GraveKiller::RoleSet(RoleSet::Mafia),
                            AttackPower::ProtectionPiercing,
                            false,
                        );
    
                        InsiderGroupID::Mafia.add_player_to_revealed_group(game, visit.target);
                        visit.target.set_win_condition(
                            game,
                            WinCondition::new_loyalist(crate::game::game_conclusion::GameConclusion::Mafia)
                        );
                        visit.target.set_night_convert_role_to(midnight_variables, Some(new_state));
    
                        self.convert_charges_remaining = false;
                        actor_ref.set_role_state(game, self);
                    }
                }else{
                    visit.target.set_night_convert_role_to(midnight_variables, Some(new_state));
                };
            },
            _ => {}
        }                
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reeducator, 0))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|player| 
                            player.alive(game) &&
                            (
                                InsiderGroupID::in_same_group(game, actor_ref, *player) || 
                                self.convert_charges_remaining
                            )
                        )
                        .collect(),
                    can_choose_duplicates: false,
                    max_players: Some(1)
                })
                .night_typical(actor_ref)
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reeducator, 1))
                .single_role_selection_typical(game, |role|
                    RoleSet::MafiaSupport.get_roles().contains(role) &&
                    *role != Role::Reeducator
                )
                .default_selection(RoleListSelection(vec![Reeducator::default_role(game)]))
                .allow_players([actor_ref])
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map()
        ])
    }
    // Unlike other conversion roles, its visit isn't tagged as an attack.
    // This is because if the target is syndicate then it is converted without an attack
    // After transportation, it becomes an attack
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game, 
            actor_ref, 
            ControllerID::role(actor_ref, Role::Reeducator, 0),
            false
        )
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}

impl Reeducator {
    pub fn default_role(game: &Game) -> Role {
        RoleSet::MafiaSupport.get_roles().into_iter()
            .filter(|p|game.settings.enabled_roles.contains(p))
            .filter(|p|*p!=Role::Reeducator)
            .choose(&mut rand::rng()).unwrap_or(Role::Reeducator)
    }
}