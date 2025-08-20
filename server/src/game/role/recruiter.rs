
use serde::Serialize;

use crate::game::controllers::AvailableIntegerSelection;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::components::mafia_recruits::MafiaRecruits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::role_list::RoleSet;
use crate::game::role_list_generation::criteria::{GenerationCriterion, GenerationCriterionResult};
use crate::game::role_list_generation::PartialOutlineListAssignmentNode;
use crate::game::settings::Settings;
use crate::game::visit::Visit;

use crate::game::Game;
use super::godfather::Godfather;
use super::{
    ControllerID,
    ControllerParametersMap, IntegerSelection, Role, RoleStateImpl
};

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
                if let Some(visit) = actor_visits.first() && Recruiter::night_ability(self.clone(), game, midnight_variables, actor_ref, visit.target) {
                    if choose_attack {
                        actor_ref.set_role_state(game, Recruiter{recruits_remaining: self.recruits_remaining.saturating_add(1)})
                    }else{
                        actor_ref.set_role_state(game, Recruiter{recruits_remaining: self.recruits_remaining.saturating_sub(1)});
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
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn role_list_generation_criteria() -> Vec<GenerationCriterion> {
        vec![ENSURE_ONE_FEWER_SYNDICATE_PER_RECRUITER]
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

pub const ENSURE_ONE_FEWER_SYNDICATE_PER_RECRUITER: GenerationCriterion = GenerationCriterion {
    evaluate: |node: &PartialOutlineListAssignmentNode, settings: &Settings| {
        let enabled_roles = &settings.enabled_roles;
        let syndicate_roles = RoleSet::Mafia.get_roles().intersection(enabled_roles);
        let town_common_roles = RoleSet::TownCommon.get_roles().intersection(enabled_roles);

        // There are currently no role sets which have mafia roles and town roles at the same time,
        // but if there were, this check says "uhh sure let's just say this is fine".
        if node.assignments
            .iter()
            .any(|assignment| 
                assignment.outline_option
                    .as_ref()
                    .is_some_and(|o| {
                        let outline_roles = o.roles.get_roles().intersection(enabled_roles);

                        !outline_roles.intersection(&syndicate_roles).is_empty() &&
                        !outline_roles.sub(&syndicate_roles).is_empty()
                    })
            )
        {
            return GenerationCriterionResult::Met;
        }

        // Which assignments are supposed to generate syndicate?
        let expected_syndicate_members = node.assignments.iter()
            .filter(|assignment| assignment.outline_option.as_ref().is_some_and(|o| {
                let outline_roles = o.roles.get_roles().intersection(enabled_roles);

                !outline_roles.is_empty() && outline_roles.is_subset(&syndicate_roles)
            }))
            .count();

        // Which assignments are actually populated with syndicate roles?
        let actual_syndicate_members = node.assignments.iter()
            .filter(|assignment| assignment.role.is_some_and(|role| syndicate_roles.contains(&role)))
            .count();

        let number_of_recruiters = node.assignments.iter()
            .filter(|assignment| assignment.role == Some(Role::Recruiter))
            .count();

        // For each recruiter, we should have one fewer syndicate member.
        if actual_syndicate_members + number_of_recruiters <= expected_syndicate_members {
            GenerationCriterionResult::Met
        } else {
            let mut new_neighbors = Vec::new();

            #[expect(clippy::indexing_slicing, reason = "Manual bounds checks")]
            // Take a random syndicate member and replace it with a random town role.
            for (syndicate_idx, _) in node.assignments.iter()
                .enumerate()
                .filter(|(_, assignment)| assignment.role.is_some_and(|role| RoleSet::Mafia.get_roles().contains(&role)))
                .filter(|(_, assignment)| assignment.role != Some(Role::Recruiter))
            {
                for role in town_common_roles.iter() {
                    let mut new_node = node.clone();
                    new_node.assignments[syndicate_idx].role = Some(*role);
                    new_node.assignments[syndicate_idx].win_condition = Some(role.default_state().default_win_condition());
                    new_node.assignments[syndicate_idx].insider_groups = Some(role.default_state().default_revealed_groups());
                    new_neighbors.push(new_node);
                }
            }

            if new_neighbors.is_empty() {
                return GenerationCriterionResult::Met;
            }

            GenerationCriterionResult::Unmet(new_neighbors)
        }
    }
};