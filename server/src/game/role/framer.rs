use serde::Serialize;
use crate::game::components::night_visits::{NightVisitsIterator, Visits};
use crate::game::components::tags::{TagSetID, Tags};
use crate::game::controllers::AvailablePlayerListSelection;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};
use crate::game::visit::{Visit, VisitTag};
use crate::game::Game;
use crate::vec_set::vec_set;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};


#[derive(Clone, Debug, Default, Serialize)]
pub struct Framer;

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Framer {
    type ClientAbilityState = Framer;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return}

        let Some(framed) = Visits::default_target(game, midnight_variables, actor_ref) else {return};
        
        framed.set_night_framed(midnight_variables, true);
        for framed_tagged in Tags::tagged(game, TagSetID::Framer(actor_ref)){
            framed_tagged.set_night_framed(midnight_variables, true);
        }
        
        Tags::set_tagged(game, TagSetID::Framer(actor_ref), &vec_set![framed]);

        framed.set_night_appeared_visits(midnight_variables, true);
        Visits::add_visits(midnight_variables, 
            Visits::into_iter(midnight_variables)
                .with_visitor(actor_ref)
                .with_tag(VisitTag::Role { role: Role::Framer, id: 1 })
                .map(|v|Visit::new_appeared(framed, v.target))
        );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Framer, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Framer, 1))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game).collect(),
                    can_choose_duplicates: false,
                    max_players: None
                })
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    // Framed player is not selected
                    ControllerID::role(actor_ref, Role::Framer, 0)
                        .get_player_list_selection(game)
                        .is_none_or(|selection| selection.0.is_empty())
                )
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Framer, 0),
            false,
        ).into_iter().chain(
            crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Framer, 1),
                false,
                VisitTag::Role { role: Role::Framer, id: 1 }
            ).into_iter().map(|mut v|{v.indirect=true; v.wardblock_immune=true; v.investigate_immune=true; v})
        ).collect()
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }

    
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {
        Tags::add_viewer(game, TagSetID::Framer(actor_ref), actor_ref);
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _new: super::RoleState, _old: super::RoleState) {
        if actor_ref != player {return}
        Tags::remove_viewer(game, TagSetID::Framer(actor_ref), actor_ref);
    }
}