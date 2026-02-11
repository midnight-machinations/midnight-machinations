use mafia_server::game::{modifiers::ModifierState, phase::{PhaseState, PhaseStateMachine, PhaseType}, Game};

pub struct TestGame (*mut Game);

impl std::ops::Deref for TestGame {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        unsafe {&*self.0}
    }
}

impl std::ops::DerefMut for TestGame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut *self.0}
    }
}

impl TestGame {
    pub fn new(game: &mut Game) -> Self {
        TestGame(game as *mut Game)
    }

    /// Advance the game naturally, passing through all the phases, until the given day and phase is met.
    /// ### Panics:
    /// * When the supplied phase doesn't always happen, like Judgement.
    /// * When the specified phase *cannot* happen, like Discussion 1.
    /// * When the specified day and phase is in the past.
    /// * If this would take the game to a day past the maximum day
    pub fn skip_to(&mut self, phase: PhaseType, day_number: u8) -> &PhaseState {
        // The only briefing phase is Briefing 1
        if phase == PhaseType::Briefing && day_number != 1 {
            panic!("The only Briefing phase is Briefing 1. Tried to go to Briefing {day_number} from {:?} {}.", self.current_phase().phase(), self.day_number()); 
        }
    
        // The only phases that occur day 1 are Briefing, Dusk and Night.
        if day_number == 1 && !matches!(phase, PhaseType::Briefing | PhaseType::Dusk | PhaseType::Night) {
            panic!("There is no {phase:?} 1. Tried to go to {phase:?} 1 from {:?} {}. The only phases that occur day 1 are Briefing, Dusk, & Night.", self.current_phase().phase(), self.day_number()); 
        }

        // If the phase & day is in the past
        if self.day_number() > day_number || (self.day_number() == day_number && self.current_phase().phase() > phase) {
            panic!("Can't skip back in time! Tried to go to {phase:?} {day_number}, but was already on {:?} {}, skip_to", self.current_phase().phase(), self.day_number());
        }

        let origin_phase = self.current_phase().phase();
        let origin_day = self.day_number();

        while self.day_number() != day_number || self.current_phase().phase() != phase {
            if self.day_number() == u8::MAX - 1 && self.current_phase().phase() == PhaseType::Night {
                panic!("Can't go above the maximum day. skip_to called during {origin_phase:?} {origin_day}.");
            }

            if self.day_number() > day_number {
                panic!("Phase {phase:?} {day_number} never occurs in the future!. skip_to called during {origin_phase:?} {origin_day}.");
            }

            self.next_phase();

            if self.day_number() > day_number || (self.day_number() == day_number && self.current_phase().phase() > phase) {
                panic!("That phase was skipped past. Maybe your test subjects need to vote someone? Tried to go to {phase:?} {day_number}, but was already on {:?} {}. skip_to called during {origin_phase:?} {origin_day}.", self.current_phase().phase(), self.day_number(),);
            }
        }

        self.current_phase()
    }
    pub fn next_phase(&mut self){
        PhaseStateMachine::next_phase(self, None);
    }

    /// Set a modifier on the game settings
    pub fn set_modifier(&mut self, state: ModifierState) {
        self.settings.modifiers.set_modifier(state);
    }
}