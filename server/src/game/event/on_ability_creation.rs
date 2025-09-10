use crate::{event_priority, game::{abilities_component::{ability::Ability, ability_id::AbilityID, Abilities}, event::Event}};

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
impl Event for OnAbilityCreation{
    type FoldValue = OnAbilityCreationFold;
    type Priority = OnAbilityCreationPriority;

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_ability_creation
    ]}

    fn initial_fold_value(&self, _game: &crate::game::Game) -> Self::FoldValue {OnAbilityCreationFold{
        ability: self.ability.clone(), cancelled: false
    }}
}
impl OnAbilityCreation{
    pub fn new(id: AbilityID, ability: Ability)->Self{Self { id, ability, }}
}