use crate::game::{ 
    abilities_component::Abilities, components::{blocked::BlockedComponent, mafia::Mafia}, event::EventData, visit::Visit,
};

use super::on_midnight::OnMidnightFold;

#[must_use = "Event must be invoked"]
pub struct OnVisitWardblocked{
    pub visit: Visit
}
impl OnVisitWardblocked{
    pub fn new(visit: Visit) -> Self{
        Self{visit}
    }
}
impl EventData for OnVisitWardblocked{
    type FoldValue = OnMidnightFold;
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_visit_wardblocked,
        Mafia::on_visit_wardblocked,
        BlockedComponent::on_visit_wardblocked,
    ]}
}