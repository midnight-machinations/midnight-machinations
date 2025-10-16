use serde::Serialize;

use crate::game::components::night_visits::{NightVisitsIterator, Visits};
use crate::game::controllers::{AvailablePlayerListSelection, ControllerParametersMap};
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::abilities_component::ability_id::AbilityID;

use crate::game::role_list::RoleSet;
use crate::game::visit::{Visit, VisitTag};

use crate::game::Game;
use super::{ControllerID, PlayerListSelection, Role, RoleState, RoleStateTrait};


#[derive(Debug, Clone, Serialize, Default)]
pub struct Godfather;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Godfather {
    type ClientAbilityState = Godfather;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        Self::night_kill_ability(game, midnight_variables, actor_ref, priority);

        if priority != OnMidnightPriority::Deception {return};

        actor_ref.set_night_appeared_visits(midnight_variables, true);
        Visits::add_visits(midnight_variables, 
            Visits::into_iter(midnight_variables)
                .with_visitor(actor_ref)
                .with_tag(VisitTag::Role { role: Role::Godfather, id: 1 })
                .map(|v|Visit::new_appeared(actor_ref, v.target))
        );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Godfather, 0))
                .single_player_selection_typical(actor_ref, false, false)
                .night_typical(actor_ref)
                .add_grayed_out_condition(game.day_number() <= 1)
                .build_map(),

            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Godfather, 1))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game).collect(),
                    can_choose_duplicates: false,
                    max_players: None
                })
                .night_typical(actor_ref)
                .build_map(),
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Godfather, 0),
            true,
        ).into_iter().chain(
            crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Godfather, 1),
                false,
                VisitTag::Role { role: Role::Godfather, id: 1 }
            ).into_iter().map(|mut v|{v.indirect=true; v.wardblock_immune=true; v.investigate_immune=true; v})
        ).collect()
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
        Self::pass_role_state_down(game, actor_ref, dead_player_ref, self);
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}

impl Godfather{
    pub(super) fn night_kill_ability(game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if game.day_number() == 1 {return}

        match priority {
            //kill the target
            OnMidnightPriority::Kill => {
                let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Godfather) else {return};
                target_ref.clone().try_night_kill_single_attacker(
                    actor_ref, game, midnight_variables, GraveKiller::RoleSet(RoleSet::Mafia),
                    AttackPower::Basic, false
                );
            },
            _ => {}
        }
    }
    pub (super) fn pass_role_state_down(
        game: &mut Game,
        actor_ref: PlayerReference,
        dead_player_ref: PlayerReference,
        new_role_data: impl Into<RoleState>
    ){
        if actor_ref != dead_player_ref {return}
        let Some(PlayerListSelection(backup)) = ControllerID::syndicate_choose_backup().get_player_list_selection(game) else {return};
        let Some(backup) = backup.first().copied() else {return};

        //convert backup to godfather
        backup.set_new_role(game, new_role_data, true);
    }
}