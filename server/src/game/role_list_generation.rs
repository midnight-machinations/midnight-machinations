use std::collections::VecDeque;

use rand::seq::SliceRandom;

use crate::{game::{components::{insider_group::InsiderGroupID, win_condition::WinCondition}, player::PlayerReference, role::Role, role_list::{RoleList, RoleOutlineOption, RoleOutlineOptionInsiderGroups, RoleOutlineOptionWinCondition}}, vec_set::VecSet};


pub struct RoleListGenerator<'a> {
    role_list: &'a RoleList,
    nodes: Vec<PartialOutlineListAssignmentNode>
}

impl<'a> RoleListGenerator<'a> {
    #![allow(clippy::indexing_slicing, reason = "We ensure the index is valid before accessing it")]

    pub fn new(role_list: &'a RoleList) -> RoleListGenerator<'a> {
        RoleListGenerator {
            role_list,
            nodes: Vec::new()
        }
    }

    pub fn generate_role_list(&mut self) -> Option<OutlineListAssignment> {
        let mut nodes_to_visit = VecDeque::new();
        nodes_to_visit.push_back(0);
        self.nodes.push(PartialOutlineListAssignmentNode {
            assignments: vec![PartialOutlineAssignment {
                outline_option: None,
                role: None,
                insider_groups: None,
                win_condition: None,
                player: None
            }; self.role_list.0.len()]
        });
        let mut seen = VecSet::new();

        while let Some((current_idx, current_node)) = nodes_to_visit.pop_front().and_then(|idx| self.nodes.get(idx).map(|node| (idx, node))) {
            if seen.contains(&current_idx) {
                continue;
            }
            seen.insert(current_idx);
            let current_node = current_node.clone();

            if let Some(role_assignment) = self.try_build_final_list(&current_node) {
                return Some(role_assignment);
            }

            nodes_to_visit.extend(self.neighbors_of(&current_node));
        }

        None
    }

    /// The neighbors of a node (more like children of the node) go like this:
    /// If the node looks like
    /// ```txt
    /// [godfather]
    /// [consort]
    /// [detective]
    /// [doctor]
    /// [ <unfilled> ]
    /// [ <unfilled> ]
    /// ```
    /// then all of its children will be like
    /// ```txt
    /// [godfather]
    /// [consort]
    /// [detective]
    /// [doctor]
    /// [ <some random role> ]
    /// [ <unfilled> ]
    /// ```
    /// That's true when it comes to filling in roles, and that pattern continues for the following order of precedence:
    /// outline options, players, win conditions, and insider groups
    fn neighbors_of(&mut self, node: &PartialOutlineListAssignmentNode) -> Vec<usize> {
        let mut out = Vec::new();

        // Populate roles before anything else
        for (i, assignment) in node.assignments.iter().enumerate() {
            if assignment.role.is_none() {
                let mut roles = self.role_list.0[i].get_all_roles();
                roles.shuffle(&mut rand::rng());
                for role in roles {
                    out.push(self.nodes.len());
                    let mut new_node = node.clone();
                    new_node.assignments[i].role = Some(role);
                    self.nodes.push(new_node);
                }
                return out;
            }
        }

        // Populate outline option choices
        // This is done after roles, despite being slower, to ensure TP | Veteran doesn't
        // have a 50% chance of generating Veteran.
        for (i, assignment) in node.assignments.iter().enumerate() {
            if assignment.outline_option.is_none() {
                let Some(role) = assignment.role else {
                    continue; // If role is not set, skip this assignment
                };

                let mut options = self.role_list.0[i].options.iter()
                    .filter(|&o| o.roles.get_roles().contains(&role))
                    .cloned()
                    .collect::<Vec<_>>();

                options.shuffle(&mut rand::rng());
                for outline_option in options {
                    out.push(self.nodes.len());
                    let mut new_node = node.clone();
                    new_node.assignments[i].outline_option = Some(outline_option);
                    self.nodes.push(new_node);
                }
                return out;
            }
        }

        // Populate players assignments
        for (i, assignment) in node.assignments.iter().enumerate() {
            if assignment.player.is_none() {
                #[expect(clippy::cast_possible_truncation, reason = "The role list is guaranteed to have at most 256 items")]
                let mut players = (0..self.role_list.0.len())
                    .map(|idx| unsafe { PlayerReference::new_unchecked(idx as u8) })
                    .filter(|p| {
                        // Ensure the player is not already assigned a role in this assignment
                        !node.assignments.iter().any(|a| a.player == Some(*p))
                    })
                    .filter(|p| {
                        // Ensure the player is able to be in one of the role outline options
                        assignment.outline_option.as_ref().is_some_and(|o| {
                            o.player_pool.is_empty() || o.player_pool.contains(&p.index())
                        })
                    })
                    .collect::<Vec<_>>();
                players.shuffle(&mut rand::rng());
                for player_possibility in players {
                    out.push(self.nodes.len());
                    let mut new_node = node.clone();
                    new_node.assignments[i].player = Some(player_possibility);
                    self.nodes.push(new_node);
                }
                return out;
            }
        }

        // Populate win conditions
        for (i, assignment) in node.assignments.iter().enumerate() {
            if assignment.win_condition.is_none() {
                let mut win_conditions = assignment.get_possible_win_conditions();
                win_conditions.shuffle(&mut rand::rng());
                for win_condition in win_conditions {
                    out.push(self.nodes.len());
                    let mut new_node = node.clone();
                    new_node.assignments[i].win_condition = Some(win_condition);
                    self.nodes.push(new_node);
                }
                return out;
            }
        }

        // Populate insider groups
        for (i, assignment) in node.assignments.iter().enumerate() {
            if assignment.insider_groups.is_none() {
                let mut insider_groups = assignment.get_possible_insider_group_combinations();
                insider_groups.shuffle(&mut rand::rng());
                for insider_group_possibility in insider_groups {
                    out.push(self.nodes.len());
                    let mut new_node = node.clone();
                    new_node.assignments[i].insider_groups = Some(insider_group_possibility);
                    self.nodes.push(new_node);
                }
                return out;
            }
        }

        out
    }

    fn try_build_final_list(&self, node: &PartialOutlineListAssignmentNode) -> Option<OutlineListAssignment> {
        if node.assignments.iter().any(|a| a.role.is_none() || a.win_condition.is_none() || a.insider_groups.is_none() || a.player.is_none()) {
            return None;
        }

        let mut assignments = Vec::new();
        #[expect(clippy::unwrap_used, reason = "We ensure all fields are set before this point")]
        for assignment in node.assignments.iter() {
            assignments.push(OutlineAssignment {
                role: assignment.role.unwrap(),
                insider_groups: assignment.insider_groups.clone().unwrap(),
                win_condition: assignment.win_condition.clone().unwrap(),
                player: assignment.player.unwrap(),
            });
        }
        Some(OutlineListAssignment { assignments })
    }
}

#[derive(Clone)]
struct PartialOutlineListAssignmentNode {
    pub assignments: Vec<PartialOutlineAssignment>
}

#[derive(Clone)]
struct PartialOutlineAssignment {
    pub outline_option: Option<RoleOutlineOption>,
    pub role: Option<Role>,
    pub insider_groups: Option<VecSet<InsiderGroupID>>,
    pub win_condition: Option<WinCondition>,
    pub player: Option<PlayerReference>
}

impl PartialOutlineAssignment {
    pub fn get_possible_win_conditions(&self) -> Vec<WinCondition> {
        let Some(role) = self.role else {
            return vec![];
        };
        let Some(outline_option) = self.outline_option.clone() else {
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

    pub fn get_possible_insider_group_combinations(&self) -> Vec<VecSet<InsiderGroupID>> {
        let Some(role) = self.role else {
            return vec![];
        };
        let Some(outline_option) = self.outline_option.clone() else {
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
}

pub struct OutlineListAssignment {
    pub assignments: Vec<OutlineAssignment>
}

#[derive(Clone)]
pub struct OutlineAssignment {
    pub role: Role,
    pub insider_groups: VecSet<InsiderGroupID>,
    pub win_condition: WinCondition,
    pub player: PlayerReference
}