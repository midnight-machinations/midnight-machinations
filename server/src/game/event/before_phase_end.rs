use crate::game::{
    abilities_component::Abilities,
    components::verdicts_today::VerdictsToday,
    event::Event, modifiers::ModifierSettings,
    phase::PhaseType, Game
};

#[must_use = "Event must be invoked"]
pub struct BeforePhaseEnd{
    pub phase: PhaseType
}
impl BeforePhaseEnd{
    pub fn new(phase: PhaseType)->Self{
        Self{phase}
    }
}
impl Event for BeforePhaseEnd{
    type FoldValue = ();

    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        VerdictsToday::before_phase_end,
        Abilities::before_phase_end,
        ModifierSettings::before_phase_end,
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}