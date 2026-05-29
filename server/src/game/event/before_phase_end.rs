use crate::game::{event::EventData, phase::PhaseType};
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct BeforePhaseEnd{
    pub phase: PhaseType
}
impl BeforePhaseEnd{
    pub fn new(phase: PhaseType)->(Self, ()){
        (Self{phase}, ())
    }
}
impl EventData for BeforePhaseEnd{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for BeforePhaseEnd{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::components::verdicts_today::VerdictsToday::before_phase_end,
        crate::game::abilities_component::Abilities::before_phase_end,
        crate::game::modifiers::ModifierSettings::before_phase_end,
    ]}
}