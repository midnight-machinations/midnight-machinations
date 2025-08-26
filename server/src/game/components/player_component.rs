use crate::game::player::PlayerReference;

pub struct PlayerComponent<T>{
    data: Box<[T]>
}
impl<T> PlayerComponent<T>{
    /// # Safety
    /// player_count <= the games real player count
    pub unsafe fn new_component_box(player_count: u8, map: impl FnMut(PlayerReference)->T)->Self{
        Self { data: unsafe { PlayerReference::all_players_from_count(player_count).map(map).collect() } }
    }

    pub fn get(&self, player: PlayerReference)->&T{
        self.data.get::<usize>(player.index().into()).expect("data.len() == player_count")
    }
    pub fn get_mut(&mut self, player: PlayerReference)->&mut T{
        self.data.get_mut::<usize>(player.index().into()).expect("data.len() == player_count")
    }
}