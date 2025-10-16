use serde::Serialize;

use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::components::night_visits::Visits;
use crate::game::controllers::ControllerParametersMap;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::components::cult::Cult;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use super::{common_role, ControllerID, Role, RoleStateTrait};


#[derive(Clone, Debug, Default, Serialize)]
pub struct Apostle;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Apostle {
    type ClientAbilityState = Apostle;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if !matches!(priority, OnMidnightPriority::Convert) {return}
        let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::Apostle) else {return};
        if !Cult::enough_sacrifices(game) {return}
        
        if !AttackPower::Basic.can_pierce(target.night_defense(game, midnight_variables)) {
            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::YourConvertFailed);
            return;
        }

        Cult::use_sacrifices(game);
        
        for p in PlayerReference::all_players(game){
            if !InsiderGroupID::Cult.contains_player(game, p) {continue}
            if !matches!(p.role(game), Role::Zealot) {continue}
            p.set_night_convert_role_to(midnight_variables, Some(Role::Disciple.default_state()))
        }

        target.set_role_win_con_insider_group_midnight(game, midnight_variables, Role::Zealot.default_state());
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref, 
            ControllerID::role(actor_ref, Role::Apostle, 0),
            true
        )
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Apostle, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(
        game.day_number() <= 1 || !Cult::enough_sacrifices(game)
            )
            .build_map()
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Cult
        ].into_iter().collect()
    }
}
