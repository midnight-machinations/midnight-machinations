use serde::{Deserialize, Serialize};

use crate::game::{event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority}, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NoWhispers;

impl From<&NoWhispers> for ModifierID{
    fn from(_: &NoWhispers) -> Self {
        ModifierID::NoWhispers
    }
}

impl ModifierStateImpl for NoWhispers {
    fn on_whisper(self, _game: &mut Game, _event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        if WhisperPriority::Cancel == priority {
            fold.cancelled = true;
            fold.hide_broadcast = true;
        }
    }
}
