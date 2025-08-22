use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};

use crate::game::components::win_condition::WinCondition;
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{Role, RoleState, RoleStateImpl};
use crate::game::controllers::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Yer{
    pub star_passes_remaining: u8,
    pub old_role: Role,
}

impl Default for Yer {
    fn default() -> Yer {
        Yer {
            star_passes_remaining: 3,
            old_role: Role::Yer
        }
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateImpl for Yer {
    type ClientRoleState = Yer;
    fn new_state(game: &Game) -> Self {
        Self{
            star_passes_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if game.day_number() == 1 {return}

        let chose_to_convert = ControllerID::role(actor_ref, Role::Yer, 0)
            .get_boolean_selection(game)
            .map(|selection| selection.0)
            .unwrap_or(false);

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first(){
            let target_ref = visit.target;

            if !chose_to_convert {
                if priority != OnMidnightPriority::Kill {return}

                target_ref.try_night_kill_single_attacker(
                    actor_ref,
                    game,
                    midnight_variables,
                    GraveKiller::Role(Role::Yer),
                    AttackPower::ArmorPiercing,
                    true
                );
            } else {
                if priority != OnMidnightPriority::Convert {return}
                if self.star_passes_remaining == 0 {return}

                if target_ref.night_defense(game, midnight_variables).can_block(AttackPower::ArmorPiercing) {
                    actor_ref.push_night_message(midnight_variables, ChatMessageVariant::YourConvertFailed);
                    return
                }

                self.star_passes_remaining = self.star_passes_remaining.saturating_sub(1);

                //role switching stuff
                let fake_role = self.current_fake_role(game, actor_ref);

                actor_ref.set_night_grave_role(midnight_variables, Some(fake_role));
                
                //convert & kill stuff
                target_ref.set_win_condition(
                    game,
                    WinCondition::new_loyalist(crate::game::game_conclusion::GameConclusion::Fiends)
                );
                target_ref.set_night_convert_role_to(midnight_variables, Some(RoleState::Yer(Yer { 
                    star_passes_remaining: self.star_passes_remaining, 
                    old_role: target_ref.role(game),
                })));

                actor_ref.try_night_kill_single_attacker(
                    actor_ref,
                    game,
                    midnight_variables,
                    GraveKiller::Role(Role::Yer),
                    AttackPower::ProtectionPiercing,
                    true
                );

                actor_ref.set_role_state(game, self);
            }
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Yer, 0))
                .available_selection(AvailableBooleanSelection)
                .add_grayed_out_condition(self.star_passes_remaining == 0 || game.day_number() <= 1)
                .night_typical(actor_ref)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Yer, 1))
                .single_player_selection_typical(actor_ref, false, true)
                .night_typical(actor_ref)
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Yer, 2))
                .single_role_selection_typical(game, |_|true)
                .default_selection(RoleListSelection(vec!(self.old_role)))
                .add_grayed_out_condition(
                    self.star_passes_remaining == 0 ||
                    actor_ref.ability_deactivated_from_death(game) ||
                    game.day_number() <= 1
                )
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Yer, 1),
            true
        )
    }
}


impl Yer{
    pub fn current_fake_role(&self, game: &Game, actor_ref: PlayerReference) -> Role {
        *ControllerID::role(actor_ref, Role::Yer, 2)
            .get_role_list_selection_first(game)
            .unwrap_or(&self.old_role)
    }
}