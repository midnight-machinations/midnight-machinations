use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::game::role_list::role_enabled_and_not_taken;
use crate::game::prelude::*;

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Pawn;

impl RoleStateTrait for Pawn {
    type ClientAbilityState = Pawn;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return}

        Visits::into_iter(midnight_variables)
            .with_direct()
            .with_visitor(actor_ref)
            .map_target()
            .filter(|player|!player.win_condition(game).friends_with_conclusion(GameConclusion::Town))
            .filter(|player|*player != actor_ref)
            .for_each(|player|player.push_night_message(midnight_variables, ChatMessageVariant::PawnVisitedYou))
    }
    fn on_role_switch(self, game: &mut Game, actor_ref: PlayerReference, event: &OnRoleSwitch, _fold: &mut (), _priority: ()) {
        if event.old.role() == Role::Pawn {return}
        if actor_ref != event.player {return}
        AbilityID::Role { role: Role::Pawn, player: actor_ref }.delete_ability(game);
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Pawn) {return}
        
        let Some(new_state) = RoleSet::TownInvestigative
            .get_roles()
            .into_iter()
            .filter(|role|role_enabled_and_not_taken(
                *role,
                &game.settings,
                PlayerReference::all_players(game).map(|p|p.role(game)).collect::<Vec<_>>().as_slice()
            ))
            .collect::<Vec<_>>()
            .choose(&mut game.rng)
            .map(|role|role.new_state(game))
        else {return};

        actor_ref.set_new_role(game, new_state, false);
    }
}