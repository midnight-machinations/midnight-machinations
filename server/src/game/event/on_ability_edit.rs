use crate::game::{abilities_component::{ability::Ability, ability_id::AbilityID}, components::role::RoleComponent, event::EventData, Game};

pub struct OnAbilityEdit{
    pub id: AbilityID,
    pub new_ability: Option<Ability>,
}
impl EventData for OnAbilityEdit{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        RoleComponent::on_ability_edit
    ]}

    fn log(&self, game: &Game, _fold: &()) -> Option<serde_json::Value> {
        if !game.game_log.is_initialized() { return None; }
        Some(serde_json::json!({ "id": self.id, "deleted": self.new_ability.is_none() }))
    }
}
impl OnAbilityEdit{
    pub fn new(id: AbilityID, new_ability: Option<Ability>)->(Self, ()){
        (Self { id, new_ability, }, ())
    }
}