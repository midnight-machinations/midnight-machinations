use crate::game::{
    abilities_component::Abilities, components::insider_group::InsiderGroups, event::Event, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnConcealRole{
    pub player: PlayerReference,
    pub concealed_player: PlayerReference
}
impl OnConcealRole{
    pub fn new(player: PlayerReference, concealed_player: PlayerReference) -> Self{
        Self{ player, concealed_player }
    }
}
impl Event for OnConcealRole{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            Abilities::on_conceal_role,
            InsiderGroups::on_conceal_role
        ]
    }

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}