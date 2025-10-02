
use serde::Serialize;

use crate::game::components::blocked::BlockedComponent;
use crate::game::components::night_visits::{NightVisitsIterator, Visits};
use crate::game::controllers::*;
use crate::game::components::graves::grave::GraveInformation;
use crate::game::components::graves::grave_reference::GraveReference;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};
use crate::game::visit::{Visit, VisitTag};
use crate::game::Game;
use super::{InsiderGroupID, Role, RoleStateTrait};


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Disguiser{
    pub last_role_selection: Role
}
impl Default for Disguiser {
    fn default() -> Self {
        Self { last_role_selection: Role::Disguiser }
    }
}
pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Disguiser {
    type ClientAbilityState = Disguiser;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return}

        let Some(appeared_visit_player) = Visits::default_target(game, midnight_variables, actor_ref) else {return};
        
        self.last_role_selection = Self::disguised_role(&self, game, actor_ref);

        appeared_visit_player.set_night_appeared_visits(midnight_variables, true);
        Visits::add_visits(midnight_variables, 
            Visits::into_iter(midnight_variables)
                .with_visitor(actor_ref)
                .with_tag(VisitTag::Role { role: Role::Disguiser, id: 1 })
                .map(|v|Visit::new_appeared(appeared_visit_player, v.target))
        );

        actor_ref.set_role_state(game, self);
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Disguiser, 0),
            false,
        ).into_iter().chain(
            crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Disguiser, 1),
                false,
                VisitTag::Role { role: Role::Disguiser, id: 1 }
            ).into_iter().map(|mut v|{v.indirect=true; v.wardblock_immune=true; v.investigate_immune=true; v})
        ).collect()
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            //player to give appeared visits
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Disguiser, 0))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|p|p.alive(game))
                        .filter(|player|*player == actor_ref || InsiderGroupID::in_same_group(game, actor_ref, *player))
                        .collect(),
                    can_choose_duplicates: false,
                    max_players: Some(1),
                })
                .night_typical(actor_ref)
                .build_map(),
            
            //appeared visits
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Disguiser, 1))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game).collect(),
                    can_choose_duplicates: false,
                    max_players: None
                })
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    // Framed player is not selected
                    ControllerID::role(actor_ref, Role::Disguiser, 0)
                        .get_player_list_selection(game)
                        .is_none_or(|selection| selection.0.is_empty())
                )
                .allow_players(
                    ControllerID::role(actor_ref, Role::Disguiser, 0)
                        .get_player_list_selection(game)
                        .and_then(|p|p.0.first().copied())
                )
                .build_map(),

            //disguise
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Disguiser, 2))
                .single_role_selection_typical(game, |_|true)
                .default_selection(RoleListSelection(vec!(self.last_role_selection)))
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn on_grave_added(self, game: &mut Game, actor_ref: PlayerReference, grave: GraveReference) {
        if BlockedComponent::blocked(game, actor_ref) {return;}
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