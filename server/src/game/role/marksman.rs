use serde::Serialize;
use crate::game::components::attack::night_attack::NightAttack;
use crate::game::prelude::*;
use crate::vec_set::VecSet;

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Marksman {
    state: MarksmanState
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum MarksmanState{
    #[default]
    NotLoaded,
    Loaded,
    ShotTownie
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Marksman {
    type ClientAbilityState = Marksman;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {

        if !matches!(self.state, MarksmanState::ShotTownie | MarksmanState::NotLoaded) {
            self.clone().midnight(actor_ref, game, midnight_variables, priority, 0, 1);
            self.clone().midnight(actor_ref, game, midnight_variables, priority, 2, 3);
        }

        if matches!(self.state, MarksmanState::NotLoaded) && matches!(priority, OnMidnightPriority::Kill) {
            actor_ref.edit_role_ability_helper(game, Marksman{state: MarksmanState::Loaded});
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        if matches!(self.state, MarksmanState::ShotTownie | MarksmanState::NotLoaded) {return ControllerParametersMap::default()};

        let available_players: VecSet<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p| p.alive(game))
            .filter(|p| *p != actor_ref)
            .collect();

        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Marksman, 0))
                .available_selection(AvailableTwoPlayerOptionSelection {
                    available_first_players: available_players.clone(),
                    available_second_players: available_players.clone(),
                    can_choose_duplicates: true,
                    can_choose_none: true
                })
                .night_typical(actor_ref)
                .build_map(),

            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Marksman, 1))
                .available_selection(AvailableTwoPlayerOptionSelection {
                    available_first_players: available_players.clone(),
                    available_second_players: available_players,
                    can_choose_duplicates: true,
                    can_choose_none: true
                })
                .night_typical(actor_ref)
                .build_map()
        ])
    }
    fn create_visits_initialize_night(self, game: &Game, _id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {
        let visits_1 = Self::one_controller_to_visits(game, actor_ref, ControllerID::role(actor_ref, Role::Marksman, 0), 0, 1);
        let visits_2 = Self::one_controller_to_visits(game, actor_ref, ControllerID::role(actor_ref, Role::Marksman, 1), 2, 3);
        [visits_1, visits_2].concat()
    }
}

impl Marksman {
    fn midnight(mut self, actor_ref: PlayerReference, game: &mut Game, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority, mark_id: u8, camp_id: u8) {
        match priority {
            OnMidnightPriority::Deception => {
                let Some(mark) = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: mark_id })
                    .map_target()
                    .next() else {return};

                let Some(camp) = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: camp_id })
                    .map_target()
                    .next() else {return};

                
                //can only attack 1 player per night, so if you already are attacking someone, then dont
                //input 0 for the mark id here because it is the other other potential attacking visit with this ability
                if
                    mark_id == 2 &&
                    Visits::into_iter(midnight_variables)
                        .with_tag(VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: 0 })
                        .find(|v|v.attack)
                        .is_some()
                {
                    return;
                }


                if 
                    Visits::into_iter(midnight_variables)
                        .with_visitor(mark)
                        .with_target(camp)
                        .with_direct()
                        .next()
                        .is_some()
                {
                    Visits::retain(midnight_variables, move |v| {
                        v.tag != VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: mark_id }
                    });
                    Visits::add_visit(midnight_variables, Visit {
                        visitor: actor_ref,
                        target: mark,
                        tag: VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: mark_id },
                        attack: true,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    });
                }
            },
            OnMidnightPriority::Kill => {
                let Some(mark) = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: mark_id })
                    .filter(|v|v.attack)
                    .map_target()
                    .next() else {return};

                let killed = NightAttack::new()
                    .attackers([actor_ref])
                    .grave_killer(Role::Marksman)
                    .attack(game, midnight_variables, mark);

                if killed && mark.win_condition(game).is_loyalist_for(GameConclusion::Town) {
                    self.state = MarksmanState::ShotTownie;
                    actor_ref.edit_role_ability_helper(game, self);
                }
            },
            _ => {}
        }
    }
    fn one_controller_to_visits(game: &Game, actor_ref: PlayerReference, id: ControllerID, mark_id: u8, camp_id: u8) -> Vec<Visit> {
        let Some(TwoPlayerOptionSelection(Some((target_1, target_2)))) = id.get_two_player_option_selection(game) else {
            return vec![];
        };

        vec![
            Visit{
                visitor: actor_ref,
                target: *target_1,
                tag: VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: mark_id },
                attack: false,
                wardblock_immune: false,
                transport_immune: false,
                investigate_immune: false,
                indirect: false
            },
            Visit{
                visitor: actor_ref,
                target: *target_2,
                tag: VisitTag::Ability { ability: AbilityID::Role { role: Role::Marksman, player: actor_ref }, id: camp_id },
                attack: false,
                wardblock_immune: false,
                transport_immune: false,
                investigate_immune: false,
                indirect: false
            }
        ]
    }
}