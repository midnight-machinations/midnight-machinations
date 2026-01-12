use crate::{game::{
    Assignments, Game, abilities::role_abilities::RoleAbility, abilities_component::{ability::Ability, ability_id::AbilityID}, chat::ChatMessageVariant, components::player_component::PlayerComponent, event::{AsInvokable as _, Invokable as _, on_ability_edit::OnAbilityEdit, on_role_switch::OnRoleSwitch}, player::PlayerReference, prelude::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority}, role::{Role, RoleState}
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
    pub fn set_role_without_ability(player: PlayerReference, game: &mut Game, role: Role){
        *game.role.get_mut(player) = role;
        Self::send_your_role_state(game, player, player.role_state_ability(game));
        if role.should_inform_player_of_assignment() {
            player.add_private_chat_message(game, ChatMessageVariant::RoleAssignment{role});
        }
    }
    pub fn on_ability_edit(game: &mut Game, event: &OnAbilityEdit, _fold: &mut (), _priority: ()){
        let AbilityID::Role{player, ..} = event.id else {return};
        let Some(ref new_ability) = event.new_ability else {return};
        Self::send_your_role_state(game, player, new_ability);
    }

    fn send_your_role_state(game: &Game, player: PlayerReference, new_ability: &Ability){
        let Ability::Role(RoleAbility(new_role_data)) = new_ability else {return};

        if !new_role_data.role().should_inform_player_of_assignment() {return}
        player.send_packet(game, ToClientPacket::YourRoleState {
            role_state: new_role_data.clone().get_client_ability_state(game, player)
        });

        if player.role(game) != new_role_data.role() {return}
        player.send_packet(game, ToClientPacket::YourRole {
            role: new_role_data.role()
        });
    }

    pub fn on_ability_creation(game: &mut Game, event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        let AbilityID::Role{player, role} = &event.id else {return};
        if priority != OnAbilityCreationPriority::SideEffect {return}
        if !role.should_inform_player_of_assignment() {return}
        player.add_private_chat_message(game, ChatMessageVariant::GainedRoleAbility{role: *role});
    }
}
impl PlayerReference{
    pub fn role(&self, game: &Game) -> Role {
        *game.role.get(*self)
    }
    pub fn set_new_role(&self, game: &mut Game, new_role_data: impl Into<RoleState>, delete_old: bool) {
        let old = self.role_state(game).clone();
        let new_role_data = new_role_data.into();
        if delete_old {
            AbilityID::Role { role: old.role(), player: *self }.delete_ability(game);
        }

        AbilityID::Role { role: new_role_data.role(), player: *self }
            .new_role_ability(game, new_role_data.clone());

        //this line must come after new_role_ability because the role requires the ability to exist
        RoleComponent::set_role_without_ability(*self, game, new_role_data.role());

        OnRoleSwitch::new(*self, old, self.role_state(game).clone()).as_invokable().invoke(game);
    }

    pub fn role_state_ability<'a>(&self, game: &'a Game) -> &'a Ability {
        AbilityID::Role { role: self.role(game), player: *self }
            .get_ability(game)
            .expect("Every player must have a role ability corresponding to their current role.")
    }
    pub fn role_state<'a>(&self, game: &'a Game) -> &'a RoleState {
        let Ability::Role(RoleAbility(role_state)) = self.role_state_ability(game) else { unreachable!("AbilityID::Role must correspond to a role") };
        
        role_state
    }
    pub fn edit_role_ability_helper(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let new_role_data = new_role_data.into();
        let role = new_role_data.role();
        AbilityID::Role { role, player: *self }
            .edit_role_ability(game, new_role_data);
    }
}
impl Role{
    pub fn should_inform_player_of_assignment(&self)->bool{
        !matches!(self, Role::Pawn|Role::Drunk)
    }
}
