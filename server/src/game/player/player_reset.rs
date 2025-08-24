use std::time::Duration;
use crate::{client_connection::ClientConnection, game::{phase::PhaseType, Game}};
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
        self.set_fast_forward_vote(game, false);
        self.role_state(game).clone().on_phase_start(game, *self, phase)
    }
}


