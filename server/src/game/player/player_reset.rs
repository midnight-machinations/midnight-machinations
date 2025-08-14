use std::time::Duration;
use crate::{client_connection::ClientConnection, game::{modifiers::{ModifierType, Modifiers}, phase::PhaseType, verdict::Verdict, Game}};
use super::PlayerReference;


impl PlayerReference{
    pub fn tick(&self, game: &mut Game, time_passed: Duration){
        match &self.deref(game).connection {
            ClientConnection::Connected(_) => self.send_repeating_data(game),
            ClientConnection::CouldReconnect { disconnect_timer } => {
                match disconnect_timer.saturating_sub(time_passed) {
                    Duration::ZERO => {
                        self.quit(game);
                    },
                    time_remaining => {
                        self.deref_mut(game).connection = ClientConnection::CouldReconnect { disconnect_timer: time_remaining }
                    }
                }
            },
            _ => {}
        }
    }

    pub fn on_phase_start(&self, game: &mut Game, phase: PhaseType){
        match phase {
            PhaseType::Briefing => {},
            PhaseType::Obituary => {},
            PhaseType::Discussion => {},
            PhaseType::Nomination => {
                self.set_verdict(
                    game, 
                    if Modifiers::is_enabled(game, ModifierType::Abstaining) {
                        Verdict::Abstain
                    } else {
                        Verdict::Innocent
                    }
                );
            },
            PhaseType::Testimony => {},
            PhaseType::Judgement => {},
            PhaseType::FinalWords => {},
            PhaseType::Dusk => {},
            PhaseType::Night => {},
            PhaseType::Recess => {}
        }

        self.set_fast_forward_vote(game, false);
        self.role_state(game).clone().on_phase_start(game, *self, phase)
    }
}


