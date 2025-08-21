#![allow(clippy::indexing_slicing, reason = "We ensure the index is valid before accessing it")]

use std::collections::HashMap;

use crate::{game::{components::{insider_group::InsiderGroupID, win_condition::WinCondition}, game_conclusion::{GameConclusion, GameOverCheckPlayer}, player::PlayerReference, role_list::{RoleOutlineOptionInsiderGroups, RoleOutlineOptionWinCondition}, role_list_generation::{PartialOutlineAssignment, PartialOutlineListAssignmentNode}, settings::Settings}, vec_set::VecSet};


#[derive(Clone, Copy)]
pub struct GenerationCriterion {
    pub evaluate: fn(node: &PartialOutlineListAssignmentNode, settings: &Settings) -> GenerationCriterionResult
}

pub enum GenerationCriterionResult {
    /// This criterion is unmet, but these neighbors satisfy it!
    Unmet(Vec<PartialOutlineListAssignmentNode>),
    /// This criterion is met, carry on.
    Met
}

pub const FILL_ALL_ROLES: GenerationCriterion = GenerationCriterion {
    evaluate: |node, settings| {
        if let Some((i, _)) = node.assignments
            .iter()
            .enumerate()
            .find(|(_, assignment)| assignment.role.is_none())
        {
            GenerationCriterionResult::Unmet(
                settings.role_list.0[i].get_all_roles()
                    .iter()
                    .filter(|role| {
                        settings.enabled_roles.contains(role)
                    })
                    .map(|role| {
                        let mut new_node = node.clone();
                        new_node.assignments[i].role = Some(*role);
                        new_node
                    })
                    .collect()
            )
        } else {
            GenerationCriterionResult::Met
        }
    }
};

pub const REJECT_EXCEEDED_ROLE_LIMITS: GenerationCriterion = GenerationCriterion {
    evaluate: |node, _| {
        let mut role_appearances = HashMap::new();

        for assignment in node.assignments.iter() {
            if let Some(role) = assignment.role {
                *role_appearances.entry(role).or_insert(0) += 1;
            }
        }

        let mut exceeded_roles = vec![];

        for role in role_appearances.keys() {
            if let Some(max) = role.maximum_count() && role_appearances[role] > max && !exceeded_roles.contains(role) {
                exceeded_roles.push(*role);
            }
        }

        if !exceeded_roles.is_empty() {
            let mut neighbors_to_add = Vec::new();

            for role in exceeded_roles {
                for assignment in node.assignments.iter() {
                    if assignment.role == Some(role) {
                        let mut new_node = node.clone();
                        new_node.assignments.iter_mut()
                            .filter(|a| a.role == Some(role))
                            .for_each(|a| a.role = None);
                        neighbors_to_add.push(new_node);
                    }
                }
            }

            GenerationCriterionResult::Unmet(neighbors_to_add)
        } else {
            GenerationCriterionResult::Met
        }
    }
};

pub const FILL_ALL_OUTLINE_OPTIONS: GenerationCriterion = GenerationCriterion {
    evaluate: |node, settings| {
        if let Some((i, assignment)) = node.assignments
            .iter()
            .enumerate()
            .find(|(_, assignment)| assignment.outline_option.is_none())
        {
            GenerationCriterionResult::Unmet(
                settings.role_list.0[i].options.iter()
                    .filter(|&o| assignment.role.is_some_and(|r| o.roles.get_roles().contains(&r)))
                    .cloned()
                    .map(|outline_option| {
                        let mut new_node = node.clone();
                        new_node.assignments[i].outline_option = Some(outline_option);
                        new_node
                    })
                    .collect()
            )
        } else {
            GenerationCriterionResult::Met
        }
    }
};

pub const FILL_ALL_PLAYERS: GenerationCriterion = GenerationCriterion {
    evaluate: |node, _| {
        #[expect(clippy::cast_possible_truncation, reason = "node.assignments.len() cannot exceed the number of players")]
        if let Some((i, assignment)) = node.assignments
            .iter()
            .enumerate()
            .find(|(_, assignment)| assignment.player.is_none())
        {
            GenerationCriterionResult::Unmet(
                (0..node.assignments.len())
                    .map(|idx| unsafe { PlayerReference::new_unchecked(idx as u8) })
                    .filter(|p| {
                        // Ensure the player is not already assigned a role in this assignment
                        !node.assignments.iter().any(|a| a.player == Some(*p))
                    })
                    .filter(|p| {
                        // Ensure the player is able to be in this role outline option
                        assignment.outline_option.as_ref().is_some_and(|o| {
                            o.player_pool.is_empty() || o.player_pool.contains(&p.index())
                        })
                    })
                    .map(|player_possibility| {
                        let mut new_node = node.clone();
                        new_node.assignments[i].player = Some(player_possibility);
                        new_node
                    })
                    .collect()
            )
        } else {
            GenerationCriterionResult::Met
        }
    }
};

fn possible_win_conditions_for_assignment(assignment: &PartialOutlineAssignment) -> Vec<WinCondition> {
    let Some(role) = assignment.role else {
        return vec![];
    };
    let Some(outline_option) = assignment.outline_option.clone() else {
        return vec![];
    };
    match outline_option.win_condition {
        RoleOutlineOptionWinCondition::GameConclusionReached { win_if_any } => {
            vec![WinCondition::GameConclusionReached { win_if_any }]
        },
        RoleOutlineOptionWinCondition::RoleDefault => {
            vec![role.default_state().default_win_condition()]
        }
    }
}

pub const FILL_ALL_WIN_CONDITIONS: GenerationCriterion = GenerationCriterion {
    evaluate: |node, _| {
        if let Some((i, assignment)) = node.assignments
            .iter()
            .enumerate()
            .find(|(_, assignment)| assignment.win_condition.is_none())
        {
            GenerationCriterionResult::Unmet(
                possible_win_conditions_for_assignment(assignment)
                    .iter()
                    .map(|win_condition| {
                        let mut new_node = node.clone();
                        new_node.assignments[i].win_condition = Some(win_condition.clone());
                        new_node
                    })
                    .collect()
            )
        } else {
            GenerationCriterionResult::Met
        }
    }
};

pub fn possible_insider_group_combinations_for_assignment(assignment: &PartialOutlineAssignment) -> Vec<VecSet<InsiderGroupID>> {
    let Some(role) = assignment.role else {
        return vec![];
    };
    let Some(outline_option) = assignment.outline_option.clone() else {
        return vec![];
    };
    match outline_option.insider_groups {
        RoleOutlineOptionInsiderGroups::Custom { insider_groups } => {
            vec![insider_groups.clone()]
        },
        RoleOutlineOptionInsiderGroups::RoleDefault => {
            vec![role.default_state().default_revealed_groups()]
        }
    }
}

pub const FILL_ALL_INSIDER_GROUPS: GenerationCriterion = GenerationCriterion {
    evaluate: |node, _| {
        if let Some((i, assignment)) = node.assignments
            .iter()
            .enumerate()
            .find(|(_, assignment)| assignment.insider_groups.is_none())
        {
            GenerationCriterionResult::Unmet(
                possible_insider_group_combinations_for_assignment(assignment)
                    .into_iter()
                    .map(|insider_group_possibility| {
                        let mut new_node = node.clone();
                        new_node.assignments[i].insider_groups = Some(insider_group_possibility);
                        new_node
                    })
                    .collect()
            )
        } else {
            GenerationCriterionResult::Met
        }
    }
};

pub const GAME_DOESNT_END_INSTANTLY: GenerationCriterion = GenerationCriterion {evaluate: |node, _| {
    let players: Vec<GameOverCheckPlayer> = node.assignments
        .iter()
        .filter_map(|a|
            if
                let Some(win_condition) = &a.win_condition &&
                let Some(insider_groups) = &a.insider_groups &&
                let Some(role) = a.role
            {
                Some(GameOverCheckPlayer{
                    role, win_condition: win_condition.clone(), insider_groups: insider_groups.clone()
                })
            }else{
                None
            }
        )
        .collect();
    
    if GameConclusion::game_is_over(players).is_some(){
        GenerationCriterionResult::Unmet(Vec::new())
    }else{
        GenerationCriterionResult::Met
    }
}};
