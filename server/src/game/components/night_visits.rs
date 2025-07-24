use crate::game::{
    event::on_midnight::MidnightVariables, player::PlayerReference, visit::{Visit, VisitTag}
};

#[derive(Default)]
pub struct NightVisits;


impl NightVisits{
    // mutators
    fn clear_visits_from_visitor(midnight_variables: &mut MidnightVariables, visitor: PlayerReference){
        Self::retain(midnight_variables, |visit| visit.visitor != visitor);
    }
    pub fn add_visit(midnight_variables: &mut MidnightVariables, visits: Visit){
        midnight_variables.visits_mut().push(visits);
    }
    fn add_visits(midnight_variables: &mut MidnightVariables, visits: Vec<Visit>){
        midnight_variables.visits_mut().extend(visits);
    }

    pub fn all_visits(midnight_variables: &MidnightVariables) -> Vec<&Visit>{
        midnight_variables.visits().iter().collect()
    }
    pub fn all_visits_mut(midnight_variables: &mut MidnightVariables) -> impl Iterator<Item = &mut Visit>{
        midnight_variables.visits_mut().iter_mut()
    }
    pub fn all_visits_cloned(midnight_variables: &mut MidnightVariables) -> Vec<Visit>{
        midnight_variables.visits_mut().to_vec()
    }

    //Only keeps elements where f is true
    pub fn retain(midnight_variables: &mut MidnightVariables, f: impl FnMut(&Visit) -> bool){
        midnight_variables.visits_mut().retain(f);
    }

    //accessors
    fn get_untagged_visits_from_visitor(midnight_variables: &MidnightVariables, visitor: PlayerReference) -> Vec<&Visit>{
        midnight_variables.visits().iter()
            .filter(|visit| visit.visitor == visitor)
            .filter(|visit| matches!(visit.tag, VisitTag::Role{..}))
            .collect()
    }
    fn get_untagged_visits_from_visitor_mut(midnight_variables: &mut MidnightVariables, visitor: PlayerReference) -> Vec<&mut Visit>{
        midnight_variables.visits_mut().iter_mut()
            .filter(|visit| visit.visitor == visitor)
            .filter(|visit| matches!(visit.tag, VisitTag::Role{..}))
            .collect()
    }
}

impl PlayerReference{
    pub fn untagged_night_visits<'a>(&self, midnight_variables: &'a MidnightVariables) -> Vec<&'a Visit>{
        NightVisits::get_untagged_visits_from_visitor(midnight_variables, *self)
    }
    pub fn untagged_night_visits_mut<'a>(&self, midnight_variables: &'a mut MidnightVariables) -> Vec<&'a mut Visit>{
        NightVisits::get_untagged_visits_from_visitor_mut(midnight_variables, *self)
    }
    pub fn untagged_night_visits_cloned(&self, midnight_variables: &MidnightVariables) -> Vec<Visit>{
        NightVisits::get_untagged_visits_from_visitor(midnight_variables, *self)
            .into_iter()
            .copied()
            .collect()
    }
    /// Returns all visits where the player is the visitor
    pub fn all_night_visits_cloned(&self, midnight_variables: &MidnightVariables) -> Vec<Visit>{
        NightVisits::all_visits(midnight_variables)
            .into_iter()
            .filter(|visit| visit.visitor == *self)
            .copied()
            .collect()
    }
    /// Returns all visits where the player is the target
    pub fn all_night_visitors_cloned(self, midnight_variables: &MidnightVariables) -> Vec<PlayerReference> {
        NightVisits::all_visits(midnight_variables)
            .into_iter()
            .filter(|visit| visit.target == self)
            .map(|visit| visit.visitor)
            .collect()
    }
    pub fn set_night_visits(&self, midnight_variables: &mut MidnightVariables, visits: Vec<Visit>){
        NightVisits::clear_visits_from_visitor(midnight_variables, *self);
        NightVisits::add_visits(midnight_variables, visits);
    }
}