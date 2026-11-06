use serde::Serialize;

use super::{chat_group::ChatGroup, chat_message_variant::ChatMessageVariant};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage{
    pub variant: ChatMessageVariant,
    pub chat_group: Option<ChatGroup>,
    #[serde(skip_serializing_if = "is_false")]
    pub night_result: bool,
}
fn is_false(value: &bool) -> bool { !value }

impl ChatMessage{
    pub fn new(variant: ChatMessageVariant, chat_group: Option<ChatGroup>)->Self{
        Self{variant,chat_group,night_result: false}
    }
    pub fn new_private(variant: ChatMessageVariant)->Self{
        Self::new(variant, None)
    }
    pub fn new_night_result(variant: ChatMessageVariant)->Self{
        Self { night_result: true, ..Self::new_private(variant) }
    }
    pub fn new_non_private(variant: ChatMessageVariant, chat_group: ChatGroup)->Self{
        Self::new(variant, Some(chat_group))
    }
    pub fn variant(&self)->&ChatMessageVariant{
        &self.variant
    }
}
