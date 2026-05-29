use crate::game::{event::EventData, player::PlayerReference};
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnConcealRole{
    pub player: PlayerReference,
    pub concealed_player: PlayerReference
}
impl OnConcealRole{
    pub fn new(player: PlayerReference, concealed_player: PlayerReference) -> (Self, ()){
        (Self{ player, concealed_player }, ())
    }
}
impl EventData for OnConcealRole{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnConcealRole{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {
        vec![
            crate::game::abilities_component::Abilities::on_conceal_role,
            crate::game::components::insider_group::InsiderGroups::on_conceal_role
        ]
    }
}