use crate::{event_priority, game::{abilities_component::{ability::Ability, ability_id::AbilityID, Abilities}, event::EventData}};

pub struct OnAbilityCreation{
    pub id: AbilityID,
    pub ability: Ability,
}
event_priority!(OnAbilityCreationPriority{
    CancelOrEdit,
    SetAbility,
    SideEffect
});
pub struct OnAbilityCreationFold{
    pub ability: Ability,
    pub cancelled: bool,
}
impl EventData for OnAbilityCreation{
    type FoldValue = OnAbilityCreationFold;
    type Priority = OnAbilityCreationPriority;

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_ability_creation
    ]}
}
impl OnAbilityCreation{
    pub fn new(id: AbilityID, ability: Ability)->(OnAbilityCreation, OnAbilityCreationFold){
        (
            Self {
                id,
                ability: ability.clone()
            },
            OnAbilityCreationFold{
                ability,
                cancelled: false
            }
        )
    }
}