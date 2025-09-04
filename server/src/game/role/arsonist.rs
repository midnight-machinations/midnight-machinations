use serde::Serialize;

use crate::game::controllers::AvailableBooleanSelection;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::AttackPower;
use crate::game::components::tags::{TagSetID, Tags};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};


#[derive(Clone, Debug, Serialize, Default)]
pub struct Arsonist;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateImpl for Arsonist {
    type ClientRoleState = Arsonist;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Deception => {
                //douse target
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                if let Some(visit) = actor_visits.first(){
                    let target_ref = visit.target;
                    Self::douse(game, target_ref);
                }
                
                //douse all visitors
                for other_player_ref in actor_ref.all_direct_night_visitors_cloned(midnight_variables)
                    .filter(|other_player_ref| *other_player_ref != actor_ref)
                    .collect::<Vec<PlayerReference>>()
                {
                    Self::douse(game, other_player_ref);
                }
            },
            OnMidnightPriority::Kill => {
                if 
                    game.day_number() <= 1 ||
                    actor_ref.night_blocked(midnight_variables) ||
                    !Self::chose_to_ignite(game, actor_ref)
                {return};

                Self::ignite(game, actor_ref, midnight_variables);
            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Arsonist, 0))
            .available_selection(AvailableBooleanSelection)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
            .combine_overwrite_owned(
                if !Self::chose_to_ignite(game, actor_ref) {
                    ControllerParametersMap::builder(game)
                        .id(ControllerID::role(actor_ref, Role::Arsonist, 1))
                        .single_player_selection_typical(
                            actor_ref,
                            false,
                            true
                        )
                        .night_typical(actor_ref)
                        .build_map()
                }else{
                    ControllerParametersMap::default()
                }
            )
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Arsonist, 1),
            false
        )
    }
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
        Tags::remove_tag(game, TagSetID::ArsonistDoused, actor_ref);
        Tags::add_viewer(game, TagSetID::ArsonistDoused, actor_ref);
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _new: super::RoleState, _old: super::RoleState) {
        if actor_ref == player {
            Tags::remove_viewer(game, TagSetID::ArsonistDoused, actor_ref); 
        }
    }
}
impl Arsonist{
    fn douse(game: &mut Game, player: PlayerReference){
        if player.role(game) == Role::Arsonist {
            return
        }

        Tags::add_tag(game, TagSetID::ArsonistDoused, player);
    }
    pub fn ignite(game: &mut Game, igniter: PlayerReference, midnight_variables: &mut MidnightVariables) {
        for player in Tags::tagged(game, TagSetID::ArsonistDoused) {
            if player.role(game) == Role::Arsonist {continue;}
            if !player.alive(game) {continue;}
            player.try_night_kill_single_attacker(
                igniter,
                game,
                midnight_variables,
                GraveKiller::Role(Role::Arsonist),
                AttackPower::ProtectionPiercing,
                true
            );
        }
    }
    pub fn has_suspicious_aura_douse(game: &Game, player: PlayerReference) -> bool {
        Tags::has_tag(game, TagSetID::ArsonistDoused, player) &&
        PlayerReference::all_players(game).any(|player_ref|
            !player_ref.ability_deactivated_from_death(game) &&
            player_ref.role(game) == Role::Arsonist
        )
    }
    fn chose_to_ignite(game: &Game, actor: PlayerReference)->bool{
        ControllerID::role(actor, Role::Arsonist, 0).get_boolean_selection(game).is_some_and(|b|b.0)
    }
}