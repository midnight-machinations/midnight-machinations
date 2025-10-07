use serde::Serialize;
use crate::game::components::detained::Detained;
use crate::game::components::night_visits::Visits;
use crate::game::controllers::{ControllerID, PlayerListSelection};
use crate::game::attack_power::AttackPower;
use crate::game::chat::ChatMessageVariant;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;
use crate::game::role_list::RoleSet;
use crate::game::role_list_generation::criteria::{GenerationCriterion, GenerationCriterionResult};
use crate::game::role_list_generation::PartialOutlineListAssignmentNode;
use crate::game::settings::Settings;
use crate::game::{attack_power::DefensePower, player::PlayerReference};
use crate::game::abilities_component::ability_id::AbilityID;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{
    common_role, ControllerParametersMap, Role,
    RoleStateTrait
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

impl RoleStateTrait for Reeducator {
    type ClientAbilityState = Reeducator;
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        
        let Some(target) = Visits::default_target(game, midnight_variables, actor_ref) else {return};
        let Some(role) = ControllerID::role(actor_ref, Role::Reeducator, 1).get_role_list_selection_first(game) else {return};
        if !matches!(priority, OnMidnightPriority::Convert) && self.convert_charges_remaining {return}
        
        if !AttackPower::Basic.can_pierce(target.night_defense(game, midnight_variables)) {
            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::YourConvertFailed);
            return;
        }

        self.convert_charges_remaining = false;

        target.set_role_and_win_condition_and_revealed_group(game, role.default_state());

        actor_ref.set_role_state(game, self);
    }
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: crate::game::controllers::ControllerInput) {
        if actor_ref != input_player {return;}
        let Some(PlayerListSelection(target_ref)) = ability_input.get_player_list_selection_if_id(
            ControllerID::role(actor_ref, Role::Reeducator, 2)
        )else{return};
        let Some(role) = ControllerID::role(actor_ref, Role::Reeducator, 1).get_role_list_selection_first(game) else {return};
        let Some(target) = target_ref.first() else {return};
        target.set_role(game, role.default_state());
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            //convert
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reeducator, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    game.day_number() <= 1 ||
                    !self.convert_charges_remaining
                )
                .build_map(),

            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reeducator, 1))
                .single_role_selection_typical(game, |role|
                    RoleSet::MafiaSupport.get_roles().contains(role) &&
                    *role != Role::Reeducator
                )
                .allow_players([actor_ref])
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map(),

            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reeducator, 2))
                .player_list_selection_typical(actor_ref, true, true, false, false, false, Some(1))
                .add_grayed_out_condition(
                    actor_ref.ability_deactivated_from_death(game) ||
                    Detained::is_detained(game, actor_ref) ||
                    !matches!(game.current_phase().phase(),PhaseType::Night)
                )
                .dont_save()
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game, 
            actor_ref, 
            ControllerID::role(actor_ref, Role::Reeducator, 0),
            true
        )
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn role_list_generation_criteria() -> Vec<GenerationCriterion> {
        vec![ENSURE_ONE_FEWER_SYNDICATE_PER_REEDUCATOR]
    }
}

pub const ENSURE_ONE_FEWER_SYNDICATE_PER_REEDUCATOR: GenerationCriterion = GenerationCriterion {
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
            .filter(|assignment| assignment.role == Some(Role::Reeducator))
            .count();

        // For each recruiter, we should have one fewer syndicate member.
        if actual_syndicate_members.saturating_add(number_of_recruiters) <= expected_syndicate_members {
            GenerationCriterionResult::Met
        } else {
            let mut new_neighbors = Vec::new();

            #[expect(clippy::indexing_slicing, reason = "Manual bounds checks")]
            // Take a random syndicate member and replace it with a random town role.
            for (syndicate_idx, _) in node.assignments.iter()
                .enumerate()
                .filter(|(_, assignment)| assignment.role.is_some_and(|role| RoleSet::Mafia.get_roles().contains(&role)))
                .filter(|(_, assignment)| assignment.role != Some(Role::Reeducator))
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