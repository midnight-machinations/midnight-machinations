use crate::{event_priority, game::{
    abilities_component::{ability_id::AbilityID, Abilities}, event::Event, Game
}};
event_priority!(OnAbilityDeletionPriority{
    BeforeSideEffect,
    DeleteAbility
});
#[derive(Clone)]
#[must_use = "Event must be invoked"]
pub struct OnAbilityDeletion{
    pub id: AbilityID,
}
impl OnAbilityDeletion{
    pub fn new(id: AbilityID) -> Self{
        Self{ id }
    }
}
impl Event for OnAbilityDeletion {
    type FoldValue = ();
    type Priority = OnAbilityDeletionPriority;

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_ability_deletion
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}