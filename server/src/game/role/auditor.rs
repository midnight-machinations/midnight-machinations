use std::iter::once;
use serde::{Deserialize, Serialize};
use crate::game::components::confused::Confused;
use crate::game::role_outline_reference::RoleOutlineReference;
use crate::game::controllers::*;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::vec_map::VecMap;
use crate::vec_set::VecSet;
use rand::prelude::SliceRandom;
use super::{common_role, Role, RoleStateTrait};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};


#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Auditor{
    pub previously_given_results: VecMap<RoleOutlineReference, AuditorResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct AuditorResult(pub VecSet<Role>);

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Auditor {
    type ClientAbilityState = Auditor;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {

        if priority != OnMidnightPriority::Investigative {return;}
        if actor_ref.night_blocked(midnight_variables) {return;}
        
        let Some(TwoRoleOutlineOptionSelection(first, second)) = ControllerID::role(actor_ref, Role::Auditor, 0).get_two_role_outline_option_selection(game).cloned()else{return};

        if let Some(chosen_outline) = first{
            let result = Self::get_result(game, chosen_outline, Confused::is_confused(game, actor_ref));
            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::AuditorResult {
                outline_index: chosen_outline.index(),
                result: result.clone(),
            });

            self.previously_given_results.insert(chosen_outline, result);
        }

        if let Some(chosen_outline) = second{
            let result = Self::get_result(game, chosen_outline, Confused::is_confused(game, actor_ref));
            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::AuditorResult {
                outline_index: chosen_outline.index(),
                result: result.clone()
            });

            self.previously_given_results.insert(chosen_outline, result);
        }

        actor_ref.set_role_state(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Auditor, 0))
            .available_selection(AvailableTwoRoleOutlineOptionSelection(
                RoleOutlineReference::all_outlines(game)
                    .filter(|o|!self.previously_given_results.contains(o))
                    .filter(|o|o.deref(game).get_all_roles().len() > 1)
                    .map(Some)
                    .chain(once(None))
                    .collect()
            ))
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        let mut out = common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Auditor, 0),
            false
        );
        out.iter_mut().for_each(|v|{
            v.transport_immune = true;
            v.indirect = true;
        });
        out
    }
}

impl Auditor{
    const MAX_RESULT_COUNT: usize = 4;
    pub fn get_result(game: &Game, chosen_outline: RoleOutlineReference, confused: bool) -> AuditorResult {
        let outline = chosen_outline.deref(game);

        let mut all_possible_fake_roles = outline
            .get_all_roles()
            .into_iter()
            .filter(|x|game.settings.enabled_roles.contains(x))
            .collect::<Vec<Role>>();
        all_possible_fake_roles.shuffle(&mut rand::rng());

        let role = chosen_outline.deref_as_role_and_player_originally_generated(game).0;
        let mut out = VecSet::new();

        // this check says dont put the real role in if either your confused OR if a recruiter messed with the role
        // We dont want an auditor seeing that a recruiter is in the game
        if !confused && all_possible_fake_roles.contains(&role) {
            out.insert(role);
        }

        for role in all_possible_fake_roles.iter(){
            if out.count() >= Auditor::MAX_RESULT_COUNT || out.count() >= all_possible_fake_roles.len().saturating_sub(1) {break}
            out.insert(*role);
        }
        out.shuffle(&mut rand::rng());

        AuditorResult(out)
    }
}