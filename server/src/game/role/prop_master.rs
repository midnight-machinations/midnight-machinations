use std::matches;

use rand::seq::SliceRandom as _;
use serde::Serialize;
use crate::game::abilities::role_abilities::RoleAbility;
use crate::game::prelude::*;


#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropMaster{
    prop: Prop
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum Prop{
    Set{target: PlayerReference},
    #[default]
    Holding,
}

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for PropMaster {
    type ClientAbilityState = PropMaster;
    fn on_midnight(self, game: &mut Game, id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if 
            actor_ref.ability_deactivated_from_death(game) ||
            actor_ref.night_blocked(midnight_variables)
        {
            return
        }

        match priority {
            OnMidnightPriority::Deception => {
                if
                    Prop::Holding == self.prop &&
                    let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::PropMaster)
                {
                    actor_ref.edit_role_ability_helper(game, Self {prop: Prop::Set{target}});
                }

                let Some(Ability::Role(RoleAbility(RoleState::PropMaster(PropMaster{prop: Prop::Set{target}})))) = id.get_ability(game) else {return};

                target.set_night_framed(midnight_variables, true);
                target.set_night_appeared_visits(midnight_variables, true);
                Visits::add_visits(midnight_variables, 
                    Visits::into_iter(midnight_variables)
                        .with_visitor(actor_ref)
                        .with_tag(VisitTag::Role { role: Role::PropMaster, id: 1 })
                        .map(|v|Visit::new_appeared(*target, v.target))
                );
            },
            OnMidnightPriority::Investigative => {

                let Prop::Set{target} = self.prop else {return};
                let mut visit_tags: Vec<VisitTag> = Visits::into_iter(midnight_variables)
                    .with_investigatable()
                    .with_target(target)
                    .filter(|v|v.tag != VisitTag::Role { role: Role::PropMaster, id: 0 } || v.visitor != actor_ref)
                    .map_tag()
                    .collect();
                visit_tags.shuffle(&mut game.rng);

                actor_ref.push_night_message(midnight_variables, ChatMessageVariant::SpyBug { visit_tags } );
            },
            _ => {}
        }
    }





    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {       
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::PropMaster, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    Prop::Holding != self.prop
                )
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::PropMaster, 1))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game).collect(),
                    can_choose_duplicates: false,
                    max_players: None
                })
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    // Framed player is not selected
                    !matches!(self.prop, Prop::Set{..}) &&
                    ControllerID::role(actor_ref, Role::PropMaster, 0)
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
            ControllerID::role(actor_ref, Role::PropMaster, 0),
            false,
        ).into_iter().chain(
            crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::PropMaster, 1),
                false,
                VisitTag::Role { role: Role::PropMaster, id: 1 }
            ).into_iter().map(|mut v|{v.indirect=true; v.wardblock_immune=true; v.investigate_immune=true; v})
        ).collect()
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }

    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, _dead_player_ref: PlayerReference) {
        self.take_prop(game, actor_ref);
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType) {
        self.take_prop(game, actor_ref);
    }



    fn on_player_roleblocked(self, game: &mut Game, midnight_variables: &mut OnMidnightFold, actor_ref: PlayerReference, player: PlayerReference, invisible: bool) {
        common_role::on_player_roleblocked(Role::PropMaster, midnight_variables, actor_ref, player);
        if player != actor_ref {return}
        for seanced in PropMaster::get_seanced_targets(game, actor_ref) {
            seanced.roleblock(game, midnight_variables, invisible);
        }
    }
}


impl PropMaster {
    pub fn take_prop(self, game: &mut Game, actor_ref: PlayerReference) {
        if
            let Prop::Set { target } = self.prop &&
            (
                InsiderGroupID::in_same_group(game, actor_ref, target) ||
                !target.alive(game) ||
                actor_ref == target
            )
        {
            actor_ref.edit_role_ability_helper(game, PropMaster{prop: Prop::Holding});
        }
    }
    pub fn get_seanced_targets(game: &Game, actor_ref: PlayerReference) -> Vec<PlayerReference> {
        if !actor_ref.alive(game) {return vec![]}
        if !(AbilityID::Role { role: Role::PropMaster, player: actor_ref }.exists(game)) {return vec![];}
        PlayerReference::all_players(game)
            .filter(|player|
                !player.alive(game) &&
                InsiderGroupID::in_same_group(game, actor_ref, *player) &&
                !RoleSet::MafiaKilling.get_roles().contains(&player.role(game)) &&
                !RoleSet::Fiends.get_roles().contains(&player.role(game)) &&
                !RoleSet::Cult.get_roles().contains(&player.role(game))
            )
            .collect()
    }
}