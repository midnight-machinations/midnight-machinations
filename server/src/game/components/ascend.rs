use crate::game::{
    components::{graves::grave::Grave, win_condition::WinCondition}, event::on_phase_start::OnPhaseStart,
    game_conclusion::GameOverCheckPlayer, player::PlayerReference, Game
};

pub struct Ascend;

impl Ascend{
    fn ascend_players(game: &mut Game){
        if game.game_is_over() {return}
        PlayerReference::all_players(game)
            .for_each(|p|Ascend::ascend_player(game, p));
    }
    
    fn ascend_player(game: &mut Game, player: PlayerReference){
        if !Self::should_ascend(game, player) {return}
        player.die_and_add_grave(game, Grave::from_player_leave_town(game, player));
    }
    fn should_ascend(game: &Game, player: PlayerReference)->bool{
        if !player.alive(game) {return false}
        WinCondition::won_with_role_state(game, player) ||
        (
            player.win_condition(game).win_if_any_conclusions().is_some() &&
            PlayerReference::all_players(game)
                .filter(|p|p.keeps_game_running(game))
                .all(|p|WinCondition::are_friends(p.win_condition(game), player.win_condition(game)))
        )
    }

    pub fn any_instantly_ascends(players: Vec<GameOverCheckPlayer>) -> bool {
        players.iter().any(|p|Self::should_ascend_no_game(&players, p))
    }
    fn should_ascend_no_game(players: &[GameOverCheckPlayer], player: &GameOverCheckPlayer)->bool{
        if !player.alive {return false}
        
        player.win_condition.win_if_any_conclusions().is_some() &&
        players.iter()
            .filter(|p|p.keeps_game_running())
            .all(|p|WinCondition::are_friends(&p.win_condition, &player.win_condition))
    }
    

    pub fn on_tick(game: &mut Game){
        Self::ascend_players(game);
    }
    pub fn on_phase_start(game: &mut Game, _event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        Self::ascend_players(game);
    }
}