use serde::Serialize;

use super::{chat_group::ChatGroup, chat_message_variant::ChatMessageVariant};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage{
    pub variant: ChatMessageVariant,
    pub chat_group: Option<ChatGroup>,
}
impl ChatMessage{
    pub fn new(variant: ChatMessageVariant, chat_group: Option<ChatGroup>)->Self{
        Self{variant,chat_group}
    }
    pub fn new_private(variant: ChatMessageVariant)->Self{
        Self{variant, chat_group: None}
    }
    pub fn new_non_private(variant: ChatMessageVariant, chat_group: ChatGroup)->Self{
        Self{variant, chat_group: Some(chat_group)}
    }
    pub fn variant(&self)->&ChatMessageVariant{
        &self.variant
    }
}