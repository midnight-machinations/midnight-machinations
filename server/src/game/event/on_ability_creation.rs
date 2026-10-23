use crate::{event_priority, game::{abilities_component::{Abilities, ability::Ability, ability_id::AbilityID}, components::role::RoleComponent, event::EventData}};

pub struct OnAbilityCreation{
    pub id: AbilityID,
    pub ability: Ability,
}
event_priority!(OnAbilityCreationPriority{
    Edit,
    SetAbility,
    SideEffect
});
pub struct OnAbilityCreationFold{
    pub ability: Ability,
}
impl EventData for OnAbilityCreation{
    type FoldValue = OnAbilityCreationFold;
    type Priority = OnAbilityCreationPriority;

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        RoleComponent::on_ability_creation,
        Abilities::on_ability_creation,
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
            }
        )
    }
}