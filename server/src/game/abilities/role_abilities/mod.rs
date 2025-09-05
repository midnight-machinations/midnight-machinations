use crate::{
    game::{
        abilities_component::{
            ability::Ability,
            ability_id::AbilityID,
            ability_trait::AbilityTrait
        },
        components::role::RoleComponent, player::PlayerReference, role::RoleState, Game
    },
    packet::ToClientPacket
};

#[derive(Clone)]
pub struct RoleAbility(pub PlayerReference, pub RoleState);
impl AbilityTrait for RoleAbility {}

impl From<RoleAbility> for Ability where RoleAbility: AbilityTrait {
    fn from(role_struct: RoleAbility) -> Self {
        Ability::RoleAbility(role_struct)
    }
}

impl PlayerReference{
    pub fn role_state<'a>(&self, game: &'a Game) -> &'a RoleState {
        let Ability::RoleAbility(RoleAbility(_, role_state)) = AbilityID::Role { player: *self }
            .get(game)
            .expect("every player must have a role ability");
        
        role_state
    }
    pub fn set_role_state(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let new_role_data = new_role_data.into();
        let new_role = new_role_data.role();

        RoleComponent::set_role(*self, game, new_role);

        AbilityID::Role { player: *self }
            .set(game, Some(Ability::RoleAbility(RoleAbility(*self, new_role_data.clone()))));

        self.send_packet(game, ToClientPacket::YourRoleState {
            role_state: new_role_data.get_client_ability_state(game, *self)
        });
    }
}