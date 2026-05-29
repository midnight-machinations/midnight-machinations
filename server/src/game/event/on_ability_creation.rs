use crate::{event_priority, game::{abilities_component::{ability::Ability, ability_id::AbilityID}, event::EventData}};

pub struct OnAbilityCreation{
    pub id: AbilityID,
    pub ability: Ability,
}
event_priority!(OnAbilityCreationPriority{
    Edit = 0,
    SetAbility = 1,
    SideEffect = 2
});
pub struct OnAbilityCreationFold{
    pub ability: Ability,
}
impl EventData for OnAbilityCreation{
    type FoldValue = OnAbilityCreationFold;
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
            }
        )
    }
}