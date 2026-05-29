use crate::{event_priority, game::{
    abilities_component::ability_id::AbilityID, event::EventData
}};
event_priority!(OnAbilityDeletionPriority{
    BeforeSideEffect = 0,
    DeleteAbility = 1
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
impl EventData for OnAbilityDeletion {
    type FoldValue = ();
}