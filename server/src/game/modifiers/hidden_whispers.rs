use serde::{Deserialize, Serialize};

use crate::game::{event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority}, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct HiddenWhispers;

/*
    There is modifier specific code in the on_client_message::on_client_message() function
    Specifically in the ToServerPacket::SendWhisper branch of the match statement
*/
impl From<&HiddenWhispers> for ModifierID{
    fn from(_: &HiddenWhispers) -> Self {
        ModifierID::HiddenWhispers
    }
}

impl ModifierStateImpl for HiddenWhispers {
    fn on_whisper(self, _game: &mut Game, _event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        if WhisperPriority::Cancel == priority {
            fold.hide_broadcast = true;
        }
    }
}