use std::{collections::HashMap, iter};

use serde::Serialize;
use crate::{game::{components::attack::night_attack::NightAttack, prelude::*}, vec_map, vec_set};

#[derive(Debug, Clone, Serialize, Default)]
pub struct Lich;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Lich {
    type ClientAbilityState = Lich;
    fn on_midnight(self, game: &mut Game, id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {

        if let Some(target) = Visits::into_iter(midnight_variables)
            .with_tag(VisitTag::Ability { ability: *id, id: Self::WARD_ID.cast_unsigned() })
            .with_visitor(actor_ref)
            .map_target()
            .next()
        {
            target.ward_night_action(game, midnight_variables, priority);
        }

        match priority {
            OnMidnightPriority::Possess => {
                Possession::possess_night_action(
                    game,
                    midnight_variables,
                    VisitTag::Ability { ability: *id, id: Self::POSSESS_ID.cast_unsigned() },
                    VisitTag::Ability { ability: *id, id: Self::POSSESS_INTO_ID.cast_unsigned() },
                );
            },
            OnMidnightPriority::Transporter => {
                let mut targets = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: *id, id: Self::TRANSPORT_ID.cast_unsigned() })
                    .with_visitor(actor_ref)
                    .map_target();

                let Some(a) = targets.next() else {return};
                let Some(b) = targets.next() else {return};
                
                Transport::transport(
                    midnight_variables, TransportPriority::Transporter,
                    &vec_map![(a, b), (b, a)], |_| true, true
                );
            },
            OnMidnightPriority::Warper => {
                let mut transporter_visits = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: *id, id: Self::WARP_ID.cast_unsigned() })
                    .with_visitor(actor_ref)
                    .map_target();

                let Some(first_visit) = transporter_visits.next() else {return};
                let Some(second_visit) = transporter_visits.next() else {return};
                
                Transport::transport(
                    midnight_variables, TransportPriority::Warper, 
                    &vec_map![(first_visit, second_visit)], |_| true, true, 
                );
            },
            OnMidnightPriority::Roleblock => {
                if let Some(target_ref) = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: *id, id: Self::ROLEBLOCK_ID.cast_unsigned() })
                    .with_visitor(actor_ref)
                    .map_target()
                    .next()
                {
                    target_ref.roleblock(game, midnight_variables, true);
                }
            }
            OnMidnightPriority::Convert => {
                if priority != OnMidnightPriority::Convert {return;}
                let Some(target) = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: *id, id: Self::TAILOR_ID.cast_unsigned() })
                    .with_visitor(actor_ref)
                    .map_target()
                    .next() else {return};

                let Some(&role) = ControllerID::role(actor_ref, Role::Lich, Self::TAILOR_ID_ROLE.cast_unsigned()).get_role_list_selection_first(game) else {return};
            
                if RoleSet::TownCommon.get_roles().contains(&target.role(game)) {
                    target.set_night_convert_role_to(midnight_variables, Some(role.new_state(game)));
                }
            },
            OnMidnightPriority::Kill => {
                if game.day_number() == 1 {return}

                if let Some(target) = Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::Ability { ability: *id, id: Self::CHOOSE_ATTACK_ID.cast_unsigned() })
                    .with_visitor(actor_ref)
                    .map_target()
                    .next()
                {
                    NightAttack::new()
                        .attackers([actor_ref])
                        .grave_killer(Role::Lich)
                        .power(AttackPower::ArmorPiercing)
                        .leave_death_note()
                        .attack(game, midnight_variables, target);
                }
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Lich, Self::CHOOSE_ATTACK_ID.cast_unsigned()))
                .single_player_selection_typical(actor_ref, false, true)
                .night_typical(actor_ref)
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map(),

            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Lich, Self::CHOOSE_ABILITY_ID.cast_unsigned()))
                .available_selection(AvailableIntegerSelection{
                    min: 2,
                    max: 7,
                })
                .default_selection(IntegerSelection(2))
                .allow_players([actor_ref])
                .build_map()

        ].into_iter().chain(

            PlayerReference::all_players(game)
                .filter(|p| !p.alive(game))
                .map(|player|
                    ControllerParametersMap::builder(game)
                        .id(ControllerID::LichVote { lich: actor_ref, player })
                        .single_player_selection_typical(player, false, true)
                        .reset_on_phase_start(PhaseType::Obituary)
                        .allow_players([player])
                        .add_grayed_out_condition(game.day_number() <= 1)
                        .build_map()
                )
        ).chain(iter::once(
                match ControllerID::role(actor_ref, Role::Lich, Self::CHOOSE_ABILITY_ID.cast_unsigned()).get_integer_selection(game) {
                    Some(IntegerSelection(Self::TRANSPORT_ID)) => {
                        let available_players: vec_set::VecSet<PlayerReference> = PlayerReference::all_players(game)
                            .filter(|p| p.alive(game))
                            .collect();
                        Some(
                            ControllerParametersMap::builder(game)
                                .id(ControllerID::role(actor_ref, Role::Lich, Self::TRANSPORT_ID.cast_unsigned()))
                                .available_selection(AvailableTwoPlayerOptionSelection {
                                    available_first_players: available_players.clone(),
                                    available_second_players: available_players,
                                    can_choose_duplicates: false,
                                    can_choose_none: true
                                })
                                .night_typical(actor_ref)
                                .build_map()
                        )
                    },
                    Some(IntegerSelection(Self::WARP_ID)) => Some(
                        ControllerParametersMap::builder(game)
                            .id(ControllerID::role(actor_ref, Role::Lich, Self::WARP_ID.cast_unsigned()))
                            .available_selection(AvailableTwoPlayerOptionSelection {
                                available_first_players: PlayerReference::all_players(game)
                                    .filter(|p|p.alive(game))
                                    .filter(|p|*p != actor_ref)
                                    .collect(),
                                available_second_players:PlayerReference::all_players(game)
                                    .filter(|p|p.alive(game))
                                    .collect(),
                                can_choose_duplicates: false,
                                can_choose_none: true
                            })
                            .night_typical(actor_ref)
                            .build_map()
                    ),
                    Some(IntegerSelection(Self::WARD_ID)) => Some(
                        ControllerParametersMap::builder(game)
                            .id(ControllerID::role(actor_ref, Role::Lich, Self::WARD_ID.cast_unsigned()))
                            .single_player_selection_typical(actor_ref, false, true)
                            .night_typical(actor_ref)
                            .build_map()
                    ),
                    Some(IntegerSelection(Self::TAILOR_ID)) => Some(
                        ControllerParametersMap::combine([
                            //player to convert
                            ControllerParametersMap::builder(game)
                                .id(ControllerID::role(actor_ref, Role::Lich, Self::TAILOR_ID.cast_unsigned()))
                                .single_player_selection_typical(actor_ref, false, true)
                                .night_typical(actor_ref)
                                .build_map(),
                            //role
                            ControllerParametersMap::builder(game)
                                .id(ControllerID::role(actor_ref, Role::Lich, Self::TAILOR_ID_ROLE.cast_unsigned()))
                                .single_role_selection_typical(game, |r|RoleSet::TownCommon.get_roles().contains(r))
                                .night_typical(actor_ref)
                                .build_map()
                    ])),
                    Some(IntegerSelection(Self::ROLEBLOCK_ID)) => Some(
                        ControllerParametersMap::builder(game)
                            .id(ControllerID::role(actor_ref, Role::Lich, Self::ROLEBLOCK_ID.cast_unsigned()))
                            .single_player_selection_typical(actor_ref, false, true)
                            .night_typical(actor_ref)
                            .build_map()
                    ),
                    Some(IntegerSelection(Self::POSSESS_ID)) => Some(
                        ControllerParametersMap::builder(game)
                            .id(ControllerID::role(actor_ref, Role::Lich, Self::POSSESS_ID.cast_unsigned()))
                            .available_selection(AvailableTwoPlayerOptionSelection {
                                available_first_players: PlayerReference::all_players(game)
                                    .filter(|p|p.alive(game))
                                    .filter(|p|*p != actor_ref)
                                    .collect(),
                                available_second_players: PlayerReference::all_players(game)
                                    .filter(|p|p.alive(game))
                                    .collect(),
                                can_choose_duplicates: true,
                                can_choose_none: true
                            })
                            .night_typical(actor_ref)
                            .build_map()
                    ),
                    _ => None
                }
            ).flatten()
        ))
    }
    fn convert_selection_to_visits(self, game: &Game, id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {

        //// TRANSPORT
        let mut out = Vec::new();
        out.append(
            &mut common_role::convert_controller_selection_to_visits_visit_tag(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Lich, Self::TRANSPORT_ID.cast_unsigned()),
                false,
                VisitTag::Ability { ability: *id, id: Self::TRANSPORT_ID.cast_unsigned() }
            )
        );

        //WARP
        out.append(
            &mut common_role::convert_controller_selection_to_visits_visit_tag(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Lich, Self::WARP_ID.cast_unsigned()),
            false,
            VisitTag::Ability { ability: *id, id: Self::WARP_ID.cast_unsigned() }
            )
        );

        //WARD
        out.append(
            &mut common_role::convert_controller_selection_to_visits_visit_tag(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Lich, Self::WARD_ID.cast_unsigned()),
                false,
                VisitTag::Ability { ability: *id, id: Self::WARD_ID.cast_unsigned() }
            ).into_iter().map(|mut v|{v.wardblock_immune = true; v}).collect()
        );

        //TAILOR
        if ControllerID::role(actor_ref, Role::Lich, Self::TAILOR_ID_ROLE.cast_unsigned()).get_role_list_selection_first(game).is_some() {   
            out.append(
                &mut crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
                    game,
                    actor_ref,
                    ControllerID::role(actor_ref, Role::Lich, Self::TAILOR_ID.cast_unsigned()),
                    false,
                    VisitTag::Ability { ability: *id, id: Self::TAILOR_ID.cast_unsigned() }
                )
            )
        }

        //ROLEBLOCK
        out.append(
            &mut crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Lich, Self::ROLEBLOCK_ID.cast_unsigned()),
                false,
                VisitTag::Ability { ability: *id, id: Self::ROLEBLOCK_ID.cast_unsigned() }
            )
        );

        //POSSESS
        out.append(
            &mut common_role::convert_controller_selection_to_visits_possession(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Lich, Self::POSSESS_ID.cast_unsigned()),
                VisitTag::Ability { ability: AbilityID::Role { role: Role::Lich, player: actor_ref }, id: Self::POSSESS_ID.cast_unsigned() },
                VisitTag::Ability { ability: AbilityID::Role { role: Role::Lich, player: actor_ref }, id: Self::POSSESS_INTO_ID.cast_unsigned() }
            )
        );

        Lich::append_visits_from_votes(id, game, actor_ref, &mut out);

        out
    }

    fn on_visit_wardblocked(self, _game: &mut Game, id: &AbilityID, event: &OnVisitWardblocked, fold: &mut OnMidnightFold, _priority: ()) {
        if
            let VisitTag::Ability { ability: visit_ability, id: visit_id } = event.visit.tag &&
            visit_id == Self::WARD_ID.cast_unsigned() &&
            visit_ability == *id
        {
            return;
        }
        common_role::on_visit_wardblocked(id, fold, event.visit);
    }
    fn on_player_roleblocked(self, _game: &mut Game, id: &AbilityID, event: &OnPlayerRoleblocked, fold: &mut OnMidnightFold, _priority: ()) {
        if id.get_player_from_role_id().is_some_and(|actor|actor == event.player) {
            //all else is roleblock immune lmao, transport ward ward roleblock and possess. Tailor is the only non immune one
            Visits::retain(fold, |v|
                if let VisitTag::Ability { ability, id: visit_tag_id } = v.tag {
                    ability != *id ||
                    visit_tag_id != Self::TAILOR_ID.cast_unsigned()
                }else{
                    true
                }
            );
        }
    }
}

impl Lich{
    pub const CHOOSE_ATTACK_ID: i8 = 0;
    pub const CHOOSE_ABILITY_ID: i8 = 1;
    pub const TRANSPORT_ID: i8 = 2;
    pub const WARP_ID: i8 = 3;
    pub const TAILOR_ID: i8 = 4;
    pub const WARD_ID: i8 = 5;
    pub const ROLEBLOCK_ID: i8 = 6;
    pub const POSSESS_ID: i8 = 7;

    pub const TAILOR_ID_ROLE: i8 = 14;
    pub const POSSESS_INTO_ID: i8 = 15;

    fn append_visits_from_votes(id: &AbilityID, game: &Game, actor_ref: PlayerReference, out: &mut Vec<Visit>) {
        if game.day_number() == 1 {return}

        let mut backup_choice_visits = common_role::convert_controller_selection_to_visits_visit_tag(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Lich, Self::CHOOSE_ATTACK_ID.cast_unsigned()),
            true,
            VisitTag::Ability { ability: *id, id: Self::CHOOSE_ATTACK_ID.cast_unsigned() }
        );
        if backup_choice_visits.is_empty() {return}

        //// ATTACK
        // Count votes into map
        let mut target_vote_map = HashMap::<PlayerReference, u8>::default();
        PlayerReference::all_players(game)
            .filter_map(|player|
                ControllerID::LichVote { lich: actor_ref, player }
                    .get_player_list_selection(game)
                    .and_then(|o|o.0.first())
                    .copied()
            )
            .for_each(|target|{
                let votes = target_vote_map.entry(target).or_default();
                *votes = (*votes).saturating_add(1);
            });

        // Find max votes
        let max_votes = 
        if
            let Some((_, &max_votes)) = target_vote_map.iter()
                .max_by(|a,b|{
                    a.1.cmp(b.1)
                })
            && max_votes >= 1
        {
            max_votes
        }else {
            out.append(&mut backup_choice_visits);
            return;
        };

        // Create visits
        let players_to_attack: Vec<PlayerReference> = target_vote_map
            .into_iter()
            .filter(|(_,v)|*v == max_votes)
            .map(|p|p.0)
            .filter(|p|*p!=actor_ref)
            .collect();
            
        let mut visits = 
        if players_to_attack.is_empty() {
            backup_choice_visits
        }else{
            players_to_attack
                .into_iter()
                .map(|target_ref|
                    Visit{
                        visitor: actor_ref,
                        target: target_ref,
                        tag: VisitTag::Ability { ability: *id, id: Self::CHOOSE_ATTACK_ID.cast_unsigned() },
                        attack: true,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    }
                )
                .collect()
        };

        out.append(&mut visits);
        
    }
}