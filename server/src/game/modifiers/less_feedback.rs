use serde::{Deserialize, Serialize};

use crate::game::chat::{ChatMessage, ChatMessageVariant};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct LessFeedback;

impl From<&LessFeedback> for ModifierID{
    fn from(_: &LessFeedback) -> Self {
        ModifierID::LessFeedback
    }
}
impl ModifierStateImpl for LessFeedback{}
impl LessFeedback {
    pub fn should_block_message(message: &ChatMessage) -> bool {
        match message.variant {
            ChatMessageVariant::RoleBlocked |
            ChatMessageVariant::Transported |
            ChatMessageVariant::Wardblocked |

            ChatMessageVariant::YouWereAttacked |
            ChatMessageVariant::YouSurvivedAttack |
            
            ChatMessageVariant::YouWereGuarded => true,

            // ChatMessageVariant::YouWerePossessed |
            // ChatMessageVariant::YouAttackedSomeone |
            // ChatMessageVariant::YouGuardedSomeone |
            _ => false
        }
    }
}
