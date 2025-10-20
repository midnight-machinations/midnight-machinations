use crate::{event_priority, game::{
    abilities_component::{ability_id::AbilityID, Abilities}, event::EventData
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
    pub fn new(id: AbilityID) -> (Self, ()){
        (Self{ id }, ())
    }
}
impl EventData for OnAbilityDeletion {
    type FoldValue = ();
    type Priority = OnAbilityDeletionPriority;

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_ability_deletion
    ]}
}