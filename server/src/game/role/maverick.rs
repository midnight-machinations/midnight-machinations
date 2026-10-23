use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::game::prelude::*;
use crate::vec_set::VecSet;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Maverick{
    pub other_roles: VecSet<Role>
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Maverick {
    type ClientAbilityState = Maverick;
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &crate::game::event::on_ability_creation::OnAbilityCreation, _fold: &mut crate::game::event::on_ability_creation::OnAbilityCreationFold, priority: crate::game::event::on_ability_creation::OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Maverick) {return}
        
        let roles = Self::get_available_roles(game);
        roles
            .elements()
            .collect::<Vec<_>>()
            .choose_multiple(&mut game.rng, 2)
            .for_each(|r|{
                let new_state = r.new_state(game);
                AbilityID::Role { role: **r, player: actor_ref }.new_role_ability(game, new_state)
            });

        AbilityID::Role { role: Role::Maverick, player: actor_ref }.edit_role_ability(game, Maverick{
            other_roles: roles
        });
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &crate::game::event::on_ability_deletion::OnAbilityDeletion, _fold: &mut (), priority: crate::game::event::on_ability_deletion::OnAbilityDeletionPriority) {
        if priority != OnAbilityDeletionPriority::BeforeSideEffect || !event.id.is_players_role(actor_ref, Role::Maverick) {return}
        self.other_roles.iter().for_each(|r|
            AbilityID::Role { role: *r, player: actor_ref }.delete_ability(game)
        );
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}
impl Maverick{
    fn get_available_roles(game: &Game) -> VecSet<Role> {
        RoleSet::MafiaSupport.get_roles().into_iter()
            .filter(|role|game.settings.enabled_roles.contains(role))
            .filter(|r|!matches!(r, Role::Reeducator))
            .collect::<VecSet<Role>>()
    }
}