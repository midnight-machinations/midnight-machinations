use crate::game::{
    components::player_component::PlayerComponent, event::on_phase_start::OnPhaseStart, phase::PhaseType,
    player::PlayerReference, Game
};


pub type BlockedComponent = PlayerComponent<Blocked>;
pub struct Blocked{
    blocked: bool
}
impl Blocked{
    fn new()->Self{
        Self { blocked: false }
    }
}
impl BlockedComponent{
    /// # Safety
    /// player_count is correct
    pub unsafe fn new(player_count: u8)->Self{
        unsafe { PlayerComponent::new_component_box(player_count, |_|Blocked::new()) }
    }

    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        if !matches!(event.phase.phase(), PhaseType::Night) {return}
        for player in PlayerReference::all_players(game) {
            game.blocked.get_mut(player).blocked = false;
        }
    }

    pub fn blocked(game: &Game, player: PlayerReference)->bool{
        game.blocked.get(player).blocked
    }

    pub fn set_blocked(game: &mut Game, player: PlayerReference){
        game.blocked.get_mut(player).blocked = true;
    }
}