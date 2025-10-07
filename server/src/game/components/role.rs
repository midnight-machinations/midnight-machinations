use crate::{game::{
    abilities::role_abilities::RoleAbility, abilities_component::{ability::Ability, ability_id::AbilityID}, components::player_component::PlayerComponent, event::{on_ability_edit::OnAbilityEdit, on_role_switch::OnRoleSwitch, Event}, player::PlayerReference, role::{Role, RoleState}, Assignments, Game
}, packet::ToClientPacket};

pub type RoleComponent = PlayerComponent::<Role>;
impl RoleComponent{
    /// # Safety
    /// num_players must be correct
    pub unsafe fn new(num_players: u8, assignments: &Assignments) -> Self {
        unsafe {
            PlayerComponent::<Role>::new_component_box(
                num_players,
                |player| assignments.get(&player).expect("Already checked this was fine").role
            )
        }
    }
    pub fn set_role(player: PlayerReference, game: &mut Game, role: Role){
        *game.role.get_mut(player) = role;
        Self::send_your_role_state(game, player, player.role_state_ability(game));
    }
    pub fn on_ability_edit(game: &mut Game, event: &OnAbilityEdit, _fold: &mut (), _priority: ()){
        let AbilityID::Role{player, ..} = event.id else {return};
        let Some(ref new_ability) = event.new_ability else {return};
        Self::send_your_role_state(game, player, new_ability);
    }

    fn send_your_role_state(game: &Game, player: PlayerReference, new_ability: &Ability){
        let Ability::Role(RoleAbility(new_role_data)) = new_ability else {return};

        if !new_role_data.role().should_inform_player_of_assignment() {return}
        if player.role(game) != new_role_data.role() {return} 

        player.send_packet(game, ToClientPacket::YourRoleState {
            role_state: new_role_data.clone().get_client_ability_state(game, player)
        });
    }
}
impl PlayerReference{
    pub fn role(&self, game: &Game) -> Role {
        *game.role.get(*self)
    }
    pub fn set_new_role_delete_old(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let old = self.role_state(game).clone();

        let new_role_data = new_role_data.into();
        let new_role = new_role_data.role();
        let role_ability_id = AbilityID::Role { role: self.role(game), player: *self };

        if self.role(game) != new_role {
            role_ability_id.delete_ability(game);
        }
        
        role_ability_id.new_role_ability(game, new_role_data.clone());
        
        RoleComponent::set_role(*self, game, new_role);

        OnRoleSwitch::new(*self, old, self.role_state(game).clone()).invoke(game);
    }
}