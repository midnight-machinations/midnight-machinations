use crate::game::{
    components::player_component::PlayerComponent, event::{on_role_switch::OnRoleSwitch, Event}, player::PlayerReference, role::{Role, RoleState}, Assignments, Game
};

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
    }
}
impl PlayerReference{
    pub fn role(&self, game: &Game) -> Role {
        *game.role.get(*self)
    }
    pub fn set_role(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let new_role_data = new_role_data.into();
        let old = self.role_state(game).clone();

        self.set_role_state(game, new_role_data.clone());

        OnRoleSwitch::new(*self, old, self.role_state(game).clone()).invoke(game);
    }
}