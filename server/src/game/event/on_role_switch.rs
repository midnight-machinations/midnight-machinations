use crate::game::{event::EventData, player::PlayerReference, role::RoleState};
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnRoleSwitch{
    pub player: PlayerReference,
    pub old: RoleState,
    pub new: RoleState,
}
impl OnRoleSwitch{
    pub fn new(player: PlayerReference, old: RoleState, new: RoleState) -> (Self, ()){
        (Self{ player, old, new }, ())
    }
}
impl EventData for OnRoleSwitch{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnRoleSwitch{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::components::role_reveal::RevealedPlayersComponent::on_role_switch,
        crate::game::components::cult::Cult::on_role_switch,
        crate::game::components::mafia::Mafia::on_role_switch,
        crate::game::components::synopsis::SynopsisTracker::on_role_switch,
        crate::game::abilities_component::Abilities::on_role_switch,
    ]}
}