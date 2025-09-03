use crate::{
    game::{
        chat::ChatMessageVariant, controllers::{ControllerID, PlayerListSelection},
        event::{on_midnight::MidnightVariables, on_phase_start::OnPhaseStart},
        phase::PhaseState, player::PlayerReference, Game
    },
    packet::ToClientPacket, vec_set::VecSet
};

impl Game {
    fn silenced(&self)->&Silenced{
        &self.silenced
    }
    fn silenced_mut(&mut self) -> &mut Silenced {
        &mut self.silenced
    }
}
#[derive(Default, Clone)]
pub struct Silenced {
    silenced_players: VecSet<PlayerReference>,
}
impl Silenced {
    pub fn silence_night(game: &mut Game, midnight_variables: &mut MidnightVariables, player: PlayerReference) {
        game.silenced_mut().silenced_players.insert(player);

        player.push_night_message(midnight_variables, ChatMessageVariant::Silenced);
        player.send_packet(game, ToClientPacket::YourSendChatGroups { send_chat_groups: 
            player.get_current_send_chat_groups(game).into_iter().collect()
        });
    }
    pub fn unsilence(game: &mut Game, player: PlayerReference) {
        game.silenced_mut().silenced_players.remove(&player);
    }
    pub fn silenced(game: &Game, player: PlayerReference) -> bool {
        game.silenced().silenced_players.contains(&player)
    }
    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()) {
        match event.phase {
            PhaseState::Testimony {player_on_trial, .. } => {
                if !Silenced::silenced(game, player_on_trial) {return;};
                ControllerID::CallWitness { player: player_on_trial }.set_selection(
                    game,
                    None,
                    PlayerListSelection(
                        PlayerReference::all_players(game).filter(|p|p.alive(game)).collect()
                    ),
                    true
                );
            },
            PhaseState::Night => {
                for player in PlayerReference::all_players(game) {
                    Silenced::unsilence(game, player);
                }
            },
            _ => {}
        }
    }
}