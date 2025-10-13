use crate::game::{abilities_component::{ability::Ability, ability_id::AbilityID}, components::role::RoleComponent, event::Event};

pub struct OnAbilityEdit{
    pub id: AbilityID,
    pub new_ability: Option<Ability>,
}
impl Event for OnAbilityEdit{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        RoleComponent::on_ability_edit
    ]}

    fn initial_fold_value(&self, _game: &crate::game::Game) -> Self::FoldValue {}
}
impl OnAbilityEdit{
    pub fn new(id: AbilityID, new_ability: Option<Ability>)->Self{Self { id, new_ability, }}
}