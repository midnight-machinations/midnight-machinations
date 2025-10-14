use std::borrow::Borrow;

use crate::game::{
    components::insider_group::InsiderGroupID, event::on_midnight::MidnightVariables, game_conclusion::GameConclusion, player::PlayerReference, visit::{Visit, VisitTag}, Game
};

#[derive(Default)]
pub struct Visits;


impl Visits{
    // mutators
    fn clear_visits_from_visitor(midnight_variables: &mut MidnightVariables, visitor: PlayerReference){
        Self::retain(midnight_variables, |visit| visit.visitor != visitor);
    }
    pub fn add_visit(midnight_variables: &mut MidnightVariables, visits: Visit){
        midnight_variables.visits_mut().push(visits);
    }
    pub fn add_visits(midnight_variables: &mut MidnightVariables, visits: impl IntoIterator<Item = Visit>){
        midnight_variables.visits_mut().extend(visits);
    }

    //Only keeps elements where f is true
    pub fn retain(midnight_variables: &mut MidnightVariables, f: impl FnMut(&Visit) -> bool){
        midnight_variables.visits_mut().retain(f);
    }
}

impl PlayerReference{
    pub fn role_night_visits_cloned(&self, midnight_variables: &MidnightVariables) -> Vec<Visit>{
        Visits::iter(midnight_variables)
            .with_visitor(*self)
            .filter(|visit| matches!(visit.tag, VisitTag::Role{..}))
            .copied()
            .collect()
    }
    /// Returns all vists where the player is the target
    pub fn all_direct_night_visitors_cloned(self, midnight_variables: &MidnightVariables) -> impl Iterator<Item = PlayerReference> + use<> {
        Visits::into_iter(midnight_variables)
            .with_target(self)
            .with_direct()
            .map_visitor()
    }
    pub fn set_night_visits(&self, midnight_variables: &mut MidnightVariables, visits: Vec<Visit>){
        Visits::clear_visits_from_visitor(midnight_variables, *self);
        Visits::add_visits(midnight_variables, visits);
    }


    pub fn tracker_seen_players(self, midnight_variables: &MidnightVariables) -> impl Iterator<Item = PlayerReference> {
        Visits::into_iter(midnight_variables)
            .with_visitor(self)
            .with_appeared(midnight_variables)
            .map_target()
    }
    pub fn lookout_seen_players(self, midnight_variables: &MidnightVariables, lookout_visit: Visit) -> impl Iterator<Item = PlayerReference> {
        Visits::into_iter(midnight_variables)
            .with_target(self)
            .with_appeared(midnight_variables)
            .without_visit(lookout_visit)
            .map_visitor()
    }
}


impl Visits{
    pub fn into_iter(midnight_variables: &MidnightVariables) -> impl Iterator<Item = Visit> + use<> {
        midnight_variables.visits().clone().into_iter()
    }
    pub fn iter(midnight_variables: &MidnightVariables) -> impl Iterator<Item = &Visit> {
        midnight_variables.visits().iter()
    }
    pub fn iter_mut(midnight_variables: &mut MidnightVariables) -> impl Iterator<Item = &mut Visit>{
        midnight_variables.visits_mut().iter_mut()
    }


    pub fn default_target(game: &Game, midnight_variables: &MidnightVariables, actor: PlayerReference) -> Option<PlayerReference>{
        Self::into_iter(midnight_variables)
            .default_target(game, actor)
    }
    pub fn default_visit(game: &Game, midnight_variables: &MidnightVariables, actor: PlayerReference) -> Option<Visit>{
        Self::into_iter(midnight_variables)
            .default_visit(game, actor)
    }
}
pub trait NightVisitsIterator: Sized {
    type Item;

    fn with_visitor(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>;
    fn with_target(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>;
    fn without_visit(self, visit: Visit) -> impl Iterator<Item = Self::Item>;
    fn with_alive_visitor(self, game: &Game) -> impl Iterator<Item = Self::Item>;
    fn with_insider_visitor(self, game: &Game, insider_group_id: InsiderGroupID) -> impl Iterator<Item = Self::Item>;
    fn with_loyalist_visitor(self, game: &Game, conclusion: GameConclusion) -> impl Iterator<Item = Self::Item>;
    fn without_loyalist_visitor(self, game: &Game, conclusion: GameConclusion) -> impl Iterator<Item = Self::Item>;
    fn with_investigatable(self) -> impl Iterator<Item = Self::Item>;
    fn with_direct(self) -> impl Iterator<Item = Self::Item>;
    fn with_tag(self, visit_tag: VisitTag) -> impl Iterator<Item = Self::Item>;

    fn map_visitor(self) -> impl Iterator<Item = PlayerReference>;
    fn map_target(self) -> impl Iterator<Item = PlayerReference>;
    fn map_tag(self) -> impl Iterator<Item = VisitTag>;

    fn default_visit(self, game: &Game, actor: PlayerReference) -> Option<Self::Item>;
    fn default_target(self, game: &Game, actor: PlayerReference) -> Option<PlayerReference>;

    fn with_appeared(self, midnight_variables: &MidnightVariables) -> impl Iterator<Item = Self::Item>;
}
impl<T> NightVisitsIterator for T 
where 
    T: Iterator,
    T::Item: Borrow<Visit>
{
    type Item = T::Item;
    fn with_visitor(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|v.borrow().visitor == player)
    }
    fn with_target(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|v.borrow().target == player)
    }
    fn without_visit(self, visit: Visit) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|*v.borrow() != visit)
    }
    fn with_alive_visitor(self, game: &Game) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|v.borrow().visitor.alive(game))
    }
    fn with_insider_visitor(self, game: &Game, insider_group_id: InsiderGroupID) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|insider_group_id.contains_player(game, v.borrow().visitor))
    }
    fn with_loyalist_visitor(self, game: &Game, conclusion: GameConclusion) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|v.borrow().visitor.win_condition(game).is_loyalist_for(conclusion))
    }
    fn without_loyalist_visitor(self, game: &Game, conclusion: GameConclusion) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|!v.borrow().visitor.win_condition(game).is_loyalist_for(conclusion))
    }
    fn with_direct(self) -> impl Iterator<Item = Self::Item>{
        self.filter(|v|!v.borrow().indirect)
    }
    fn with_investigatable(self) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|!v.borrow().investigate_immune)
    }
    fn with_tag(self, visit_tag: VisitTag) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|v.borrow().tag == visit_tag)
    }

    fn map_visitor(self) -> impl Iterator<Item = PlayerReference>{
        self.map(|v|v.borrow().visitor)
    }
    fn map_tag(self) -> impl Iterator<Item = VisitTag> {
        self.map(|v|v.borrow().tag)
    }
    fn map_target(self) -> impl Iterator<Item = PlayerReference> {
        self.map(|v|v.borrow().target)
    }

    fn default_visit(self, game: &Game, actor: PlayerReference) -> Option<Self::Item>{
        self
            .with_visitor(actor)
            .with_tag(VisitTag::Role { role: actor.role(game), id: 0 })
            .next()

    }
    fn default_target(self, game: &Game, actor: PlayerReference) -> Option<PlayerReference>{
        self
            .default_visit(game, actor)
            .map(|v|v.borrow().target)
    }

    fn with_appeared(self, midnight_variables: &MidnightVariables) -> impl Iterator<Item = Self::Item> {
        self
            .filter(|v|
                if v.borrow().visitor.night_appeared_visits(midnight_variables) {v.borrow().tag == VisitTag::Appeared} else {!v.borrow().investigate_immune}
            )
    }
}