use crate::game::{
    player::PlayerReference,
};
use super::EventData;

#[derive(Clone)]
pub struct OnRemoveInsider {
    pub player: PlayerReference,
    pub group: crate::game::components::insider_group::InsiderGroupID,
}

impl OnRemoveInsider {
    pub fn new(player: PlayerReference, group: crate::game::components::insider_group::InsiderGroupID) -> Self {
        Self { player, group }
    }
}

impl EventData for OnRemoveInsider {
    type FoldValue = ();
}