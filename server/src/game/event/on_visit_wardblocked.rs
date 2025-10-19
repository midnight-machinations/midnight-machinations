use crate::game::{ 
    abilities::syndicate_gun::SyndicateGun, abilities_component::Abilities, chat::ChatMessageVariant,
    components::{blocked::BlockedComponent, mafia::Mafia}, visit::Visit, Game
};

use super::on_midnight::MidnightVariables;

#[must_use = "Event must be invoked"]
pub struct OnVisitWardblocked{
    pub visit: Visit
}
impl OnVisitWardblocked{
    pub fn new(visit: Visit) -> Self{
        Self{visit}
    }
    pub fn invoke(self, game: &mut Game, midnight_variables: &mut MidnightVariables){
        self.visit.visitor.set_night_blocked(midnight_variables, true);
        self.visit.visitor.push_night_message(midnight_variables, ChatMessageVariant::Wardblocked);

        Abilities::on_visit_wardblocked(game, &self, midnight_variables, ());
        Mafia::on_visit_wardblocked(game, midnight_variables, self.visit);
        SyndicateGun::on_visit_wardblocked(game, midnight_variables, self.visit);
        BlockedComponent::set_blocked(game, self.visit.visitor);
    }
}