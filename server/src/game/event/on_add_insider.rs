use crate::game::{
    player::PlayerReference
};
use super::EventData;

#[derive(Clone)]
pub struct OnAddInsider {
    pub player: PlayerReference,
    pub group: crate::game::components::insider_group::InsiderGroupID
}

impl OnAddInsider {
    pub fn new(player: PlayerReference, group: crate::game::components::insider_group::InsiderGroupID) -> Self {
        Self { player, group }
    }
}

impl EventData for OnAddInsider {
    type FoldValue = ();
}