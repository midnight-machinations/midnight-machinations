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
        if game.game_is_over() {
            Self::set_host_setting(game, FastForwardSetting::None);
            return;
        }
        let host_wants_skip = match game.host_fast_forward {
            FastForwardSetting::Skip => true,
            FastForwardSetting::Phase { phase, day } =>
                Self::can_skip(game.current_phase().phase(), game.day_number(), phase, day),
            FastForwardSetting::None => false,
        };
        if host_wants_skip || Self::all_players_want_skip(game) {
            Self::skip(game)
        }
    }

    fn set_host_setting(game: &mut Game, setting: FastForwardSetting) {
        if game.host_fast_forward != setting {
            game.host_fast_forward = setting;
            game.resend_host_data_to_all_hosts();
        }
    }

    pub fn host_skip(game: &mut Game, setting: FastForwardSetting) {
        if game.game_is_over() { return; }
        if let FastForwardSetting::Phase { phase, day } = setting {
            // Match the regular skip destinations; trial phases may never occur.
            if !matches!(phase, PhaseType::Discussion | PhaseType::Nomination | PhaseType::Night)
                || day == u8::MAX
                || (day == 1 && phase != PhaseType::Night)
                || !Self::can_skip(game.current_phase().phase(), game.day_number(), phase, day) {
                return;
            }
        }
        let skip = setting != FastForwardSetting::None;
        Self::set_host_setting(game, setting);
        if skip { Self::attempt_skip(game); }
    }
    pub fn skip(game: &mut Game){
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::PhaseFastForwarded);
        game.phase_machine.time_remaining = Some(std::time::Duration::from_secs(0));
        Self::reset_votes(game);
    }

    fn reset_votes(game: &mut Game){
        if match game.host_fast_forward {
            FastForwardSetting::Skip => true,
            FastForwardSetting::Phase { phase, day } =>
                !Self::can_skip(game.current_phase().phase(), game.day_number(), phase, day),
            FastForwardSetting::None => false,
        } {
            Self::set_host_setting(game, FastForwardSetting::None);
        }
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
