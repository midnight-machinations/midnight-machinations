use crate::{
    game::{
        abilities_component::{
            ability::Ability,
            ability_id::AbilityID,
            ability_trait::AbilityTrait
        },
        components::role::RoleComponent, player::PlayerReference, role::{Role, RoleState}, Game
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
impl Role{
    pub fn should_inform_player_of_assignment(&self)->bool{
        !matches!(self, Role::Pawn|Role::Drunk)
    }
}

impl PlayerReference{
    pub fn role_state<'a>(&self, game: &'a Game) -> &'a RoleState {
        let Ability::RoleAbility(RoleAbility(_, role_state)) = AbilityID::Role { role: self.role(game), player: *self }
            .get_ability(game)
            .expect("every player must have a role ability")
            else { unreachable!() };
        
        role_state
    }
    pub fn set_role_state(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let new_role_data = new_role_data.into();
        let new_role = new_role_data.role();

        if self.role(game) != new_role {
            AbilityID::Role { role: self.role(game), player: *self }
                .delete_ability(game);
        }

        self.set_role_state_without_deleting_previous(game, new_role_data);
    }
    pub fn set_role_state_without_deleting_previous(&self, game: &mut Game, new_role_data: impl Into<RoleState>){
        let new_role_data = new_role_data.into();
        let new_role = new_role_data.role();
        RoleComponent::set_role(*self, game, new_role);

        AbilityID::Role { role: new_role, player: *self }
            .set_ability(game, Some(Ability::RoleAbility(RoleAbility(*self, new_role_data.clone()))));

        if self.role(game).should_inform_player_of_assignment() {
            self.send_packet(game, ToClientPacket::YourRoleState {
                role_state: new_role_data.get_client_ability_state(game, *self)
            });
        }
    }
}