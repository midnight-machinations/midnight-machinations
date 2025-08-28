
use serde::Serialize;

use crate::game::controllers::*;
use crate::game::components::graves::grave::GraveInformation;
use crate::game::components::graves::grave_reference::GraveReference;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};
use crate::game::visit::Visit;
use crate::game::Game;
use super::{InsiderGroupID, Role, RoleStateImpl};


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Disguiser{
    pub last_role_selection: Role,
}
impl Default for Disguiser {
    fn default() -> Self {
        Self { last_role_selection: Role::Disguiser }
    }
}
pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Disguiser {
    type ClientRoleState = Disguiser;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        
        if priority != OnMidnightPriority::Deception {return}

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let Some(first_visit) = actor_visits.first() else {return};
        
        if !InsiderGroupID::in_same_group(game, actor_ref, first_visit.target) {return}

        self.last_role_selection = Self::disguised_role(&self, game, actor_ref);

        actor_ref.set_role_state(game, self);
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Disguiser, 0),
            false
        )
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Disguiser, 0))
                .single_player_selection_typical(actor_ref, false, true)
                .night_typical(actor_ref)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Disguiser, 1))
                .single_role_selection_typical(game, |_|true)
                .default_selection(RoleListSelection(vec!(self.last_role_selection)))
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn on_grave_added(self, game: &mut Game, actor_ref: PlayerReference, grave: GraveReference) {
        let grave_ref = grave;
        
        if actor_ref == grave.deref(game).player && actor_ref.alive(game) {
            let mut grave = grave_ref.deref(game).clone();
            *grave_ref.deref_mut(game) = match grave.information {
                GraveInformation::Normal{role: _, will, death_cause, death_notes} => {
                    grave.information = GraveInformation::Normal{
                        role: Self::disguised_role(&self, game, actor_ref),
                        will,
                        death_cause,
                        death_notes
                    };
                    grave
                },
                _ => grave
            };
        }
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}

impl Disguiser{
    fn disguised_role(&self, game: &Game, actor_ref: PlayerReference)->Role{
        if let Some(role) = ControllerID::role(actor_ref, Role::Disguiser, 1)
            .get_role_list_selection(game)
            .and_then(|selection| selection.0.first().copied())
        {
            role
        }else{
            Role::Disguiser
        }
    }
}