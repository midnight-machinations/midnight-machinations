use crate::game::{
    abilities_component::Abilities, components::{cult::Cult, mafia::Mafia, role_reveal::RevealedPlayersComponent, synopsis::SynopsisTracker}, event::Event, player::PlayerReference, role::RoleState, Game
};

#[must_use = "Event must be invoked"]
pub struct OnRoleSwitch{
    pub player: PlayerReference,
    pub old: RoleState,
    pub new: RoleState,
}
impl OnRoleSwitch{
    pub fn new(player: PlayerReference, old: RoleState, new: RoleState) -> Self{
        Self{ player, old, new }
    }
}
impl Event for OnRoleSwitch{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        RevealedPlayersComponent::on_role_switch,
        Cult::on_role_switch,
        Mafia::on_role_switch,
        SynopsisTracker::on_role_switch,
        Abilities::on_role_switch,
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}