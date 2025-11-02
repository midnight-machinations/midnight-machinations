use std::collections::VecDeque;

use rand::{rngs::SmallRng, seq::SliceRandom};

use crate::{game::{prelude::*, role_list::RoleOutlineOption, role_outline_reference::RoleOutlineReference}, vec_set::VecSet};
pub mod criteria;
pub use criteria::{
    GenerationCriterion,
    GenerationCriterionResult
};

pub struct RoleListGenerator {
    rng: SmallRng,
    settings: Settings,
    new_to_original_role_outline_indices_map: Vec<usize>,
    nodes: Vec<PartialOutlineListAssignmentNode>,
    criteria: Vec<GenerationCriterion>,
}

impl RoleListGenerator {

    #[expect(clippy::indexing_slicing, reason = "Indices are guaranteed to be in-bounds")]
    pub fn new(mut settings: Settings, rng: &mut SmallRng) -> RoleListGenerator {
        let mut new_to_original_role_outline_indices_map: Vec<usize> = (0..settings.role_list.0.len()).collect();
        new_to_original_role_outline_indices_map.shuffle(rng);
        // Reorder according to the shuffled indices.
        settings.role_list.0 = new_to_original_role_outline_indices_map.iter().map(|&i| settings.role_list.0[i].clone()).collect();

        RoleListGenerator {
            settings,
            rng: rng.clone(),
            new_to_original_role_outline_indices_map,
            nodes: Vec::new(),
            criteria: vec![
                // Though it's less efficient, we want to fill roles first
                // so there's no weird probability stuff going on.
                // e.g. if we started with outline options,
                // then Town Protective | Veteran would have a 50% chance of generating Veteran.
                // So this order is important.
                criteria::FILL_ALL_ROLES,
                criteria::REJECT_EXCEEDED_ROLE_LIMITS,
                criteria::FILL_ALL_OUTLINE_OPTIONS,
                criteria::FILL_ALL_PLAYERS,
                criteria::FILL_ALL_WIN_CONDITIONS,
                criteria::FILL_ALL_INSIDER_GROUPS,
                criteria::GAME_DOESNT_END_INSTANTLY,
                criteria::NO_PLAYERS_INSTANTLY_ASCEND,
            ]
        }
    }

    const MAX_TRAVERSAL_DEPTH: usize = 250;

    // Basic DFS with a stack and a set of seen nodes to avoid cycles.
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
            }; self.settings.role_list.0.len()]
        });
        let mut seen = VecSet::new();
        let mut depth = 0;

        while let Some((current_idx, current_node)) = 
            nodes_to_visit.pop_back().and_then(|idx| self.nodes.get(idx).map(|node| (idx, node)))
            && depth < Self::MAX_TRAVERSAL_DEPTH
        {
            if seen.contains(&current_idx) {
                continue;
            }
            depth = depth.saturating_add(1);
            seen.insert(current_idx);

            let current_node = current_node.clone();

            match self.neighbors_of(&current_node) {
                NeighborsResult::AllCriteriaMet => {
                    return Some(unsafe { self.finalize(&current_node) });
                },
                NeighborsResult::NeighborIndices(neighbor_indices) => {
                    nodes_to_visit.extend(neighbor_indices);
                }
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
    fn neighbors_of(&mut self, node: &PartialOutlineListAssignmentNode) -> NeighborsResult {
        if let Some(mut neighbors_to_add) = self.criteria
            .iter()
            .copied()
            .chain(
                // Add criteria from the roles in the node.
                node.assignments.iter()
                    .filter_map(|assignment| assignment.role)
                    .flat_map(|role| role.role_list_generation_criteria())
            )
            .find_map(|criterion| {
                let result = (criterion.evaluate)(node, &self.settings);
                if let GenerationCriterionResult::Unmet(neighbors) = result {
                    Some(neighbors)
                } else {
                    None
                }
            })
        {
            let mut out = Vec::new();

            neighbors_to_add.shuffle(&mut self.rng);
            for neighbor in neighbors_to_add {
                out.push(self.nodes.len());
                self.nodes.push(neighbor);
            }

            return NeighborsResult::NeighborIndices(out);
        }

        NeighborsResult::AllCriteriaMet
    }

    /// # Safety
    /// Before calling this function, you must ensure that all fields in the node are set.
    /// This is usually guaranteed if every criterion has been met.
    #[expect(clippy::unwrap_used, reason = "We ensure all fields are set before this point")]
    unsafe fn finalize(&self, node: &PartialOutlineListAssignmentNode) -> OutlineListAssignment {
        let mut assignments = Vec::new();
        for (index, assignment) in node.assignments.iter().enumerate() {
            #[expect(clippy::indexing_slicing, clippy::cast_possible_truncation, reason = "Indices are guaranteed to be in-bounds")]
            assignments.push(OutlineAssignment {
                role_outline_reference: unsafe { RoleOutlineReference::new_unchecked(self.new_to_original_role_outline_indices_map[index] as u8) },
                role: assignment.role.unwrap(),
                insider_groups: assignment.insider_groups.clone().unwrap(),
                win_condition: assignment.win_condition.clone().unwrap(),
                player: assignment.player.unwrap(),
            });
        }
        OutlineListAssignment { assignments }
    }
}

#[derive(Clone, Debug)]
pub struct PartialOutlineListAssignmentNode {
    pub assignments: Vec<PartialOutlineAssignment>
}

#[derive(Clone, Debug)]
pub struct PartialOutlineAssignment {
    pub outline_option: Option<RoleOutlineOption>,
    pub role: Option<Role>,
    pub insider_groups: Option<VecSet<InsiderGroupID>>,
    pub win_condition: Option<WinCondition>,
    pub player: Option<PlayerReference>
}

#[derive(Debug)]
pub struct OutlineListAssignment {
    pub assignments: Vec<OutlineAssignment>
}

#[derive(Clone, Debug)]
pub struct OutlineAssignment {
    pub role_outline_reference: RoleOutlineReference,
    pub role: Role,
    pub insider_groups: VecSet<InsiderGroupID>,
    pub win_condition: WinCondition,
    pub player: PlayerReference
}

pub enum NeighborsResult {
    NeighborIndices(Vec<usize>),
    AllCriteriaMet
}