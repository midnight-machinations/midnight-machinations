use crate::game::event::EventData;
use super::{EventListenerFunction, LegacyEventData, on_midnight::OnMidnightFold};

#[must_use = "Event must be invoked"]
pub struct OnVisitWardblocked{
    pub visit: crate::game::prelude::Visit
}
impl OnVisitWardblocked{
    pub fn new(visit: crate::game::prelude::Visit) -> Self{
        Self{visit}
    }
}
impl EventData for OnVisitWardblocked{
    type FoldValue = OnMidnightFold;
}
#[allow(deprecated)]
impl LegacyEventData for OnVisitWardblocked{
    type FoldValue = OnMidnightFold;
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::abilities_component::Abilities::on_visit_wardblocked,
        crate::game::components::mafia::Mafia::on_visit_wardblocked,
        crate::game::components::blocked::BlockedComponent::on_visit_wardblocked,
    ]}
}