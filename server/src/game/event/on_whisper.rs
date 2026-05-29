use crate::{event_priority, game::player::PlayerReference};
use super::EventData;

#[derive(Clone)]
pub struct OnWhisper {
    pub sender: PlayerReference,
    pub receiver: PlayerReference,
    pub message: String,
}
pub struct WhisperFold {
    pub cancelled: bool,
    pub hide_broadcast: bool
}
event_priority!(WhisperPriority{
    Cancel = 0,
    Broadcast = 1,
    Send = 2
});

impl OnWhisper {
    pub fn new(sender: PlayerReference, receiver: PlayerReference, message: String) -> (Self, WhisperFold) {
        (
            Self {
                sender,
                receiver,
                message,
            },
            WhisperFold {
                cancelled: false,
                hide_broadcast: false,
            }
        )
    }
}

impl EventData for OnWhisper {
    type FoldValue = WhisperFold;
}