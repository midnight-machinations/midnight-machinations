use serde::Serialize;

use crate::game::controllers::{AvailableIntegerSelection, AvailablePlayerListSelection};
use crate::game::attack_power::AttackPower;
use crate::game::components::detained::Detained;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{
    attack_power::DefensePower,
    components::puppeteer_marionette::PuppeteerMarionette
};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, IntegerSelection, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Puppeteer{
    pub marionettes_remaining: u8,
}

impl Default for Puppeteer{
    fn default() -> Self {
        Self {marionettes_remaining: 3,}
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateImpl for Puppeteer {
    type ClientRoleState = Puppeteer;
    fn new_state(game: &Game) -> Self {
        Self{
            marionettes_remaining: crate::game::role::common_role::standard_charges(game),
        }
    }
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {

        if priority != OnMidnightPriority::Kill {return;}
        if game.day_number() <= 1 {return;}

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first(){
            let target = visit.target;
            
            if 
                ControllerID::role(actor_ref, Role::Puppeteer, 1).get_integer_selection(game)
                    .unwrap_or(&IntegerSelection(0)).0 == 1
            {
                if !AttackPower::ArmorPiercing.can_pierce(target.night_defense(game, midnight_variables)) {
                    actor_ref.push_night_message(midnight_variables, crate::game::chat::ChatMessageVariant::YourConvertFailed);
                }else{
                    if PuppeteerMarionette::string(game, midnight_variables, target){
                        self.marionettes_remaining = self.marionettes_remaining.saturating_sub(1);
                    }
                    actor_ref.set_role_state(game, self);
                }
            }else{
                target.try_night_kill_single_attacker(
                    actor_ref,
                    game,
                    midnight_variables,
                    GraveKiller::Role(Role::Puppeteer),
                    AttackPower::ArmorPiercing,
                    true
                );
            }
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Puppeteer, 0))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|&p|
                            actor_ref != p &&
                            p.alive(game) &&
                            !PuppeteerMarionette::marionettes_and_puppeteer(game).contains(&p)
                        )
                        .collect(),
                    can_choose_duplicates: false,
                    max_players: Some(1)
                })
                .add_grayed_out_condition(
                    Detained::is_detained(game, actor_ref) ||
                    actor_ref.ability_deactivated_from_death(game) ||
                    game.day_number() <= 1
                )
                .allow_players([actor_ref])
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Puppeteer, 1))
                .available_selection(AvailableIntegerSelection {
                    min: 0,
                    max: if self.marionettes_remaining > 0 {1} else {0}
                })
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Puppeteer, 0),
            true,
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Puppeteer
        ].into_iter().collect()
    }
}