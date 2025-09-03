use serde::{Deserialize, Serialize};

use crate::{
    game::{
        chat::{ChatGroup, ChatMessageVariant},
        components::player_component::PlayerComponent,
        event::on_phase_start::OnPhaseStart, phase::PhaseType,
        player::PlayerReference, Game
    },
    packet::ToClientPacket
};

pub type FastForwardComponent = PlayerComponent<FastForwardSetting>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum FastForwardSetting{
    None,
    Skip,
    Phase{phase: PhaseType, day: u8}
}

impl FastForwardComponent {
    /// # Safety
    /// player_count <= the games real player count
    pub unsafe fn new(num_players: u8) -> Self {
        unsafe {
            PlayerComponent::<FastForwardSetting>::new_component_box(
                num_players,
                |_player| FastForwardSetting::None
            )
        }
    }
    pub fn on_phase_start(game: &mut Game, _event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        Self::reset_votes(game);
        Self::attempt_skip(game);
    }

    fn attempt_skip(game: &mut Game){
        if Self::all_players_want_skip(game) {
            Self::skip(game)
        }
    }
    pub fn skip(game: &mut Game){
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::PhaseFastForwarded);
        game.phase_machine.time_remaining = Some(std::time::Duration::from_secs(0));
        Self::reset_votes(game);
    }

    fn reset_votes(game: &mut Game){
        for player in PlayerReference::all_players(game){
            if 
                match player.fast_forward_vote(game) {
                    FastForwardSetting::Skip => true,
                    FastForwardSetting::Phase { phase, day } => 
                        !Self::can_skip(game.phase_machine.current_state.phase(), game.day_number(), phase, day),
                    FastForwardSetting::None => false,
                }
            {
                player.set_fast_forward_vote(game, FastForwardSetting::None);
            }
        }
    }

    fn can_skip(current_phase: PhaseType, current_day: u8, to_phase: PhaseType, to_day: u8)->bool{
        current_day < to_day || (current_day == to_day && current_phase < to_phase)
    }

    fn all_players_want_skip(game: &Game)->bool{
        PlayerReference::all_players(game)
            .filter(|p|p.alive(game)&&(p.could_reconnect(game)||p.is_connected(game)))
            .all(|p|
                match p.fast_forward_vote(game) {
                    FastForwardSetting::Skip => true,
                    FastForwardSetting::Phase { phase, day } => 
                        Self::can_skip(game.phase_machine.current_state.phase(), game.day_number(), phase, day),
                    FastForwardSetting::None => false,
                }
            )
    }
}

impl PlayerReference {
    pub fn set_fast_forward_vote(&self, game: &mut Game, fast_forward_vote: FastForwardSetting) {
        *game.fast_forward.get_mut(*self) = fast_forward_vote.clone();

        self.send_packet(game, ToClientPacket::YourVoteFastForwardPhase { fast_forward: fast_forward_vote.clone() });

        FastForwardComponent::attempt_skip(game);
    }
    pub fn fast_forward_vote(&self, game: &Game) -> FastForwardSetting{
        game.fast_forward.get(*self).clone()
    }
}