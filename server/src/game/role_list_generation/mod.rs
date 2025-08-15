use std::collections::VecDeque;

use rand::seq::SliceRandom;

use crate::{game::{components::{insider_group::InsiderGroupID, win_condition::WinCondition}, player::PlayerReference, role::{recruiter, Role}, role_list::{RoleList, RoleOutlineOption}, role_list_generation::criteria::{GenerationCriterion, GenerationCriterionResult}}, vec_set::VecSet};

pub mod criteria;

pub struct RoleListGenerator<'a> {
    role_list: &'a RoleList,
    nodes: Vec<PartialOutlineListAssignmentNode>,
    criteria: Vec<GenerationCriterion>
}

impl<'a> RoleListGenerator<'a> {

    pub fn new(role_list: &'a RoleList) -> RoleListGenerator<'a> {
        RoleListGenerator {
            role_list,
            nodes: Vec::new(),
            criteria: vec![
                // Though it's less efficient, we want to fill roles first
                // so there's no weird probability stuff going on.
                // e.g. if we started with outline options,
                // then Town Protective | Veteran would have a 50% chance of generating Veteran.
                // So this order is important.
                criteria::FILL_ALL_ROLES,
                criteria::FILL_ALL_OUTLINE_OPTIONS,
                criteria::FILL_ALL_PLAYERS,
                criteria::FILL_ALL_WIN_CONDITIONS,
                criteria::FILL_ALL_INSIDER_GROUPS,
                // recruiter::ENSURE_ONE_FEWER_SYNDICATE_PER_RECRUITER
            ]
        }
    }

    const MAX_TRAVERSAL_DEPTH: usize = 2500;

    // Basic DFS with a queue and a set of seen nodes to avoid cycles.
    pub fn generate_role_list(&mut self) -> Option<OutlineListAssignment> {
        // This is a list of indices of the nodes in self.nodes that we still need to visit.
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
        let mut depth = 0;

        while let Some((current_idx, current_node)) = 
            nodes_to_visit.pop_front().and_then(|idx| self.nodes.get(idx).map(|node| (idx, node)))
            && depth < Self::MAX_TRAVERSAL_DEPTH
        {
            if seen.contains(&current_idx) {
                continue;
            }
            depth = depth.saturating_add(1);
            seen.insert(current_idx);

            let current_node = current_node.clone();
            let neighbors = self.neighbors_of(&current_node);

            if neighbors.is_empty() {
                return Some(unsafe { self.finalize(&current_node) });
            } else {
                nodes_to_visit.extend(neighbors);
            }
        }

        None
    }

    /// This function returns indices of the neighbors of the current node in the self.nodes vector.
    /// 
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
    /// 
    /// That's true when it comes to filling in roles, and that pattern continues for the following order of precedence:
    /// outline options, players, win conditions, insider groups, then all other criteria.
    fn neighbors_of(&mut self, node: &PartialOutlineListAssignmentNode) -> Vec<usize> {
        if let Some(neighbors_to_add) = self.criteria.iter()
            .find_map(|&criterion| {
                let result = (criterion.evaluate)(node, self.role_list);
                if let GenerationCriterionResult::Unmet(neighbors) = result {
                    Some(neighbors)
                } else {
                    None
                }
            })
        {
            let mut out = Vec::new();

            let mut extra_neighbors = neighbors_to_add.clone();
            extra_neighbors.shuffle(&mut rand::rng());
            for neighbor in extra_neighbors {
                out.push(self.nodes.len());
                self.nodes.push(neighbor);
            }

            return out;
        }

        vec![]
    }

    /// # Safety
    /// Before calling this function, you must ensure that all fields in the node are set.
    /// This is usually guaranteed if a node has no neighbors.
    #[expect(clippy::unwrap_used, reason = "We ensure all fields are set before this point")]
    unsafe fn finalize(&self, node: &PartialOutlineListAssignmentNode) -> OutlineListAssignment {
        let mut assignments = Vec::new();
        for assignment in node.assignments.iter() {
            assignments.push(OutlineAssignment {
                role: assignment.role.unwrap(),
                insider_groups: assignment.insider_groups.clone().unwrap(),
                win_condition: assignment.win_condition.clone().unwrap(),
                player: assignment.player.unwrap(),
            });
        }
        OutlineListAssignment { assignments }
    }
}

#[derive(Clone)]
pub struct PartialOutlineListAssignmentNode {
    pub assignments: Vec<PartialOutlineAssignment>
}

#[derive(Clone)]
pub struct PartialOutlineAssignment {
    pub outline_option: Option<RoleOutlineOption>,
    pub role: Option<Role>,
    pub insider_groups: Option<VecSet<InsiderGroupID>>,
    pub win_condition: Option<WinCondition>,
    pub player: Option<PlayerReference>
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