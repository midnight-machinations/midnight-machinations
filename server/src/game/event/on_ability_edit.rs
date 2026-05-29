use crate::game::event::EventData;

pub struct OnAbilityEdit{
    pub id: crate::game::abilities_component::ability_id::AbilityID,
    pub new_ability: Option<crate::game::abilities_component::ability::Ability>,
}
impl EventData for OnAbilityEdit{
    type FoldValue = ();
}
impl OnAbilityEdit{
    pub fn new(id: crate::game::abilities_component::ability_id::AbilityID, new_ability: Option<crate::game::abilities_component::ability::Ability>)->Self{
        Self { id, new_ability }
    }
}