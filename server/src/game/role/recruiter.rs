
use rand::seq::IteratorRandom;
use serde::Serialize;

use crate::game::ability_input::AvailableIntegerSelection;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::components::mafia_recruits::MafiaRecruits;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::components::win_condition::WinCondition;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::game_conclusion::GameConclusion;
use crate::game::role_list::{RoleOutline, RoleOutlineOption, RoleOutlineOptionRoles, RoleSet};
use crate::game::visit::Visit;

use crate::game::Game;
use crate::vec_set::VecSet;
use super::godfather::Godfather;
use super::{
    ControllerID,
    ControllerParametersMap, IntegerSelection, Role, RoleStateImpl
};

use vec1::vec1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recruiter{
    pub recruits_remaining: u8,
}

impl Default for Recruiter {
    fn default() -> Self {
        Self {
            recruits_remaining: 3,
        }
    }
}



pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Recruiter {
    type ClientRoleState = Recruiter;
    fn new_state(game: &Game) -> Self {
        Self{
            recruits_remaining: crate::game::role::common_role::standard_charges(game),
        }
    }
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {

        let choose_attack = Self::choose_attack(game, actor_ref);

        if choose_attack{
            if game.day_number() <= 1 {return}
        } else if self.recruits_remaining == 0 {return}

        match priority {
            OnMidnightPriority::Kill => {
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                if let Some(visit) = actor_visits.first(){
                    if Recruiter::night_ability(self.clone(), game, midnight_variables, actor_ref, visit.target) {
                        if choose_attack {
                            actor_ref.set_role_state(game, Recruiter{recruits_remaining: self.recruits_remaining.saturating_add(1)})
                        }else{
                            actor_ref.set_role_state(game, Recruiter{recruits_remaining: self.recruits_remaining.saturating_sub(1)});
                        }
                    }
                }
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        let choose_attack = Self::choose_attack(game, actor_ref);

        ControllerParametersMap::combine([
            // Player
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Recruiter, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    (!choose_attack && self.recruits_remaining == 0) 
                    || (choose_attack && game.day_number() == 1)
                )
                .build_map(),
            // Attack or Recruit
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Recruiter, 1))
                .available_selection(AvailableIntegerSelection {
                    min: 0,
                    max: if self.recruits_remaining > 0 {1} else {0}
                })
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Recruiter, 0),
            false
        )
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
        Godfather::pass_role_state_down(game, actor_ref, dead_player_ref, self);
    }
    fn before_initial_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {

        if game.settings.role_list.0.contains(&RoleOutline::new_exact(Role::Recruiter)) {
            return;
        }

        //get random mafia player and turn them info a random town role

        let random_mafia_player = PlayerReference::all_players(game)
            .filter(|p|RoleSet::Mafia.get_roles().contains(&p.role(game)))
            .filter(|p|*p!=actor_ref)
            .choose(&mut rand::rng());

        if let Some(random_mafia_player) = random_mafia_player {

            let random_town_role = RoleOutline {options: vec1![RoleOutlineOption {
                win_condition: Default::default(), 
                insider_groups: Default::default(), 
                roles: RoleOutlineOptionRoles::RoleSet{ role_set: RoleSet::TownCommon } ,
                player_pool: VecSet::new(),
            }]}.get_random_role_assignments(
                &game.settings.enabled_roles,
                PlayerReference::all_players(game).map(|p|p.role(game)).collect::<Vec<_>>().as_slice(),
                &[]
            ).map(|assignment| assignment.role());

            if let Some(random_town_role) = random_town_role {
                //special case here. I don't want to use set_role because it alerts the player their role changed
                //NOTE: It will still send a packet to the player that their role state updated,
                //so it might be deducable that there is a recruiter
                InsiderGroupID::Mafia.remove_player_from_insider_group(game, random_mafia_player);
                random_mafia_player.set_win_condition(game, WinCondition::GameConclusionReached{
                    win_if_any: vec![GameConclusion::Town].into_iter().collect()
                });
                random_mafia_player.set_role_state(game, random_town_role.new_state(game));
                
            }
        }
        
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}

impl Recruiter {
    /// returns true if target_ref is killed when trying to kill
    /// returns true if target_ref is recruited when trying to recruit
    pub fn night_ability(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, target_ref: PlayerReference) -> bool {
        let choose_attack = Self::choose_attack(game, actor_ref);

        if choose_attack {
            target_ref.try_night_kill_single_attacker(
                actor_ref,
                game,
                midnight_variables,
                GraveKiller::RoleSet(RoleSet::Mafia),
                AttackPower::Basic,
                false
            )
        }else if AttackPower::Basic.can_pierce(target_ref.night_defense(game, midnight_variables)) {
            MafiaRecruits::recruit(game, midnight_variables, target_ref)
        }else{
            false
        }
    }

    fn choose_attack(game: &Game, actor_ref: PlayerReference)->bool{
        if let Some(IntegerSelection(x)) = ControllerID::role(actor_ref, Role::Recruiter, 1).get_integer_selection(game)
        {*x==0}else{true}
    }
}
