use std::iter;

use rand::seq::IteratorRandom;
use serde::Serialize;
use crate::{game::prelude::*, vec_map::VecMap, vec_set::VecSet};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Dreamcatcher{
    target_nightmare: Option<(PlayerReference, Option<PlayerReference>)>,
    results: VecMap<PlayerReference, VecSet<Role>>,

    nightmares: VecMap<PlayerReference, u8>
}

impl RoleStateTrait for Dreamcatcher {
    type ClientAbilityState = Dreamcatcher;
    fn new_state(game: &mut Game) -> Self {
        Self {
            target_nightmare: None,
            results: VecMap::new(),
            nightmares: PlayerReference::all_players(game).map(|p|(p,0)).collect()
        }
    }
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if !matches!(priority, OnMidnightPriority::Investigative) {return};

        let Some((target, _nightmare)) = self.target_nightmare else {return};
        self.target_nightmare = None;
        actor_ref.edit_role_ability_helper(game, self.clone());

        let Some(result) = self.results.get(&target).cloned().or(
            ControllerID::role(actor_ref, Role::Dreamcatcher, 1)
                .get_role_list_selection(game).map(|r|{
                    r.0.clone().into_iter().chain(iter::once(target.role(game))).collect::<VecSet<Role>>()
                })
        ) else {return};
            
        self.results.insert(target, result.clone());
        actor_ref.push_night_message(midnight_variables, ChatMessageVariant::DreamcatcherResult { result });
        actor_ref.edit_role_ability_helper(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {

        let nightmare_controller = self.target_nightmare
            .iter()
            .filter_map(|(_target, nightmare)|*nightmare)
            .map(|nightmare|
                ControllerParametersMap::builder(game)
                    .id(ControllerID::role(actor_ref, Role::Dreamcatcher, 1))
                    .available_selection(AvailableRoleListSelection{
                        available_roles: game.settings.enabled_roles.clone(),
                        can_choose_duplicates: false,
                        max_roles: Some(3)
                    })
                    .reset_on_phase_start(PhaseType::Obituary)
                    .allow_players([nightmare])
                    .build_map()
            );

        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Dreamcatcher, 0))
                .single_player_selection_typical(actor_ref, false, true)
                
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .reset_on_phase_start(PhaseType::Night)
                .allow_players([actor_ref])

                .build_map(),
        ].into_iter().chain(nightmare_controller))
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        if phase != PhaseType::Night {return}

        if let Some(target) = ControllerID::role(actor_ref, Role::Dreamcatcher, 0)
            .get_player_list_selection(game)
            .iter()
            .filter_map(|o|o.0.first())
            .copied()
            .next()
        {
            let nightmare = Self::find_nightmare(&self, game, target);
            self.target_nightmare = Some((target, nightmare));
            if let Some(nightmare) = nightmare {
                let count = self.nightmares.get_mut(&nightmare)
                    .expect("self.nightmares is full due to the new_state function creating a full VecMap");
                *count = count.saturating_add(1);
                nightmare.add_private_chat_message(game, ChatMessageVariant::DreamcatcherTarget { target });
            }
            
        }else{
            self.target_nightmare = None;
        }

        actor_ref.edit_role_ability_helper(game, self);
    }
    fn convert_selection_to_visits(self, game: &Game, _id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Dreamcatcher, 0),
            false
        ).into_iter().map(|mut v|{
            v.transport_immune = true;
            v
        }).collect()
    }
}

impl Dreamcatcher {
    fn find_nightmare(&self, game: &mut Game, target: PlayerReference) -> Option<PlayerReference> {
        if self.results.contains(&target) {return None}

        if Self::valid_nightmare(game, target) {
            return Some(target);
        }

        let min_votes = self.nightmares.iter().map(|o|o.1).min().copied()
            .expect("self.nightmares is full due to the new_state function creating a full VecMap");

        self.nightmares
            .iter()
            .filter(|(_,v)|**v == min_votes)
            .map(|p|*p.0)
            .filter(|p|Self::valid_nightmare(game, *p))
            .collect::<Vec<_>>()
            .into_iter()
            .choose(&mut game.rng)
    }
    fn valid_nightmare(game: &Game, target: PlayerReference) -> bool {
        !target.win_condition(game).friends_with_conclusion(GameConclusion::Town) &&
        target.alive(game) &&
        !AbilityID::Role { role: Role::Pawn, player: target }.exists(game)
    }
}