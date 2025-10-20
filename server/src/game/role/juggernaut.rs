use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::{game::prelude::*, vec_set::VecSet};

#[derive(Debug, Clone, Default, Serialize)]
pub struct Juggernaut{
    pub other_roles: VecSet<Role>
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Juggernaut {
    type ClientAbilityState = Juggernaut;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Kill {return}
        if game.day_number() == 1 {return}
        let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Juggernaut) else {return};
        target_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Juggernaut), AttackPower::ArmorPiercing, true);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Juggernaut, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Juggernaut, 0),
            true
        )
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &crate::game::event::on_ability_creation::OnAbilityCreation, fold: &mut crate::game::event::on_ability_creation::OnAbilityCreationFold, priority: crate::game::event::on_ability_creation::OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Juggernaut) || fold.cancelled {return}
        
        let roles = Self::get_available_roles(game);
        roles
            .elements()
            .collect::<Vec<_>>()
            .choose_multiple(&mut game.rng, 2)
            .for_each(|r|{
                let new_state = r.new_state(game);
                AbilityID::Role { role: **r, player: actor_ref }.new_role_ability(game, new_state)
            });

        AbilityID::Role { role: Role::Juggernaut, player: actor_ref }.edit_role_ability(game, Juggernaut{
            other_roles: roles
        });
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &crate::game::event::on_ability_deletion::OnAbilityDeletion, _fold: &mut (), priority: crate::game::event::on_ability_deletion::OnAbilityDeletionPriority) {
        if priority != OnAbilityDeletionPriority::BeforeSideEffect || !event.id.is_players_role(actor_ref, Role::Juggernaut) {return}
        self.other_roles.iter().for_each(|r|
            AbilityID::Role { role: *r, player: actor_ref }.delete_ability(game)
        );
    }
}
impl Juggernaut{
    fn get_available_roles(game: &Game) -> VecSet<Role> {
        RoleSet::MafiaSupport.get_roles().into_iter()
            .filter(|role|game.settings.enabled_roles.contains(role))
            .filter(|r|!matches!(r, Role::Reeducator))
            .collect::<VecSet<Role>>()
    }
}