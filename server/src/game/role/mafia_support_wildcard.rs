use serde::{Serialize, Deserialize};

use crate::game::{attack_power::DefensePower, role_list::TemplateSet};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::Game;

use super::{wild_card::Wildcard, ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MafiaSupportWildcard;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for MafiaSupportWildcard {
    type ClientAbilityState = MafiaSupportWildcard;
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        match phase {
            PhaseType::Night => {
                if actor_ref.ability_deactivated_from_death(game) {return;}
                Wildcard::become_role(game, actor_ref, Role::MafiaSupportWildcard);
            },
            _ => {}
        }
    }
     fn default_insider_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::MafiaSupportWildcard, 0))
            .single_role_selection_typical(game, |role|TemplateSet::MafiaSupport.contains(*role) && *role != Role::MafiaSupportWildcard)
            .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
            .allow_players([actor_ref])
            .build_map()
    }
}