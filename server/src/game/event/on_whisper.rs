use crate::{event_priority, game::{abilities_component::Abilities, chat::ChatComponent, modifiers::ModifierSettings, player::PlayerReference}};
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
event_priority!(WhisperPriority{Cancel, Broadcast, Send});

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
    type Priority = WhisperPriority;

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            ChatComponent::on_whisper,
            ModifierSettings::on_whisper,
            Abilities::on_event,
        ]
    }
}