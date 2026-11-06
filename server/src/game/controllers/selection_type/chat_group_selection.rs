use serde::{Deserialize, Serialize};

use crate::{game::{chat::ChatGroup, Game}, vec_set::VecSet};
use super::AvailableSelectionKind;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatGroupSelection(pub Option<ChatGroup>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AvailableChatGroupSelection {
    pub groups: VecSet<ChatGroup>,
}

impl AvailableSelectionKind for AvailableChatGroupSelection {
    type Selection = ChatGroupSelection;

    fn validate_selection(&self, _game: &Game, selection: &ChatGroupSelection) -> bool {
        match selection.0 {
            Some(group) => self.groups.contains(&group),
            None => self.groups.is_empty(),
        }
    }

    fn default_selection(&self, _game: &Game) -> ChatGroupSelection {
        ChatGroupSelection(ChatGroup::select_available(&self.groups, None))
    }
}
