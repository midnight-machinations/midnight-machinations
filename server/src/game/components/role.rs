use crate::game::{
    components::player_component::PlayerComponent, player::PlayerReference, 
    role::Role, Assignments, Game
};

pub type RoleComponent = PlayerComponent::<Role>;
impl RoleComponent{
    /// # Safety
    /// num_players must be correct
    pub unsafe fn new(num_players: u8, assignments: &Assignments) -> Self {
        unsafe {
            PlayerComponent::<Role>::new_component_box(
                num_players,
                |player| assignments.get(&player).expect("Already checked this was fine").template
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
}