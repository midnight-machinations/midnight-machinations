pub mod visit;
use std::{borrow::Borrow, fmt::Debug};
use crate::game::prelude::*;
use visit::*;

#[derive(Default, Clone)]
pub struct Visits{
    ledger: Vec<VisitLedgerEvent>
}
impl Debug for Visits{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visits = self.calculate_visits();
        f.debug_struct("Visits").field("ledger", &visits).finish()
    }
}


trait VisitLedgerEventGenerator: Fn(&mut Vec<Visit>) + Send {
    fn clone_box(&self) -> Box<dyn VisitLedgerEventGenerator>;
}
impl<T> VisitLedgerEventGenerator for T where T: Fn(&mut Vec<Visit>) + Clone + Send + 'static {
    fn clone_box(&self) -> Box<dyn VisitLedgerEventGenerator> {
        Box::new(self.clone())
    } 
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitLedgerEventID{
    InitialAddVisit{ability: AbilityID}
}
pub struct VisitLedgerEvent{
    id: Option<VisitLedgerEventID>,
    generator: Box<dyn VisitLedgerEventGenerator>,
}
impl Clone for VisitLedgerEvent {
    fn clone(&self) -> Self {
        Self { id: self.id, generator: self.generator.clone_box() }
    }
}
impl VisitLedgerEvent{
    fn new<T: VisitLedgerEventGenerator + 'static>(generator: T)->Self{
        Self { id: None, generator: Box::new(generator) }
    }
    fn new_id<T: VisitLedgerEventGenerator + 'static>(id: VisitLedgerEventID, generator: T)->Self{
        Self { id: Some(id), generator: Box::new(generator) }
    }
}



impl Visits{
    pub fn replace_ledger_event_id<T: Fn(&mut Vec<Visit>) + Send + Clone + 'static>(
        midnight_variables: &mut OnMidnightFold,
        id: VisitLedgerEventID,
        generator: T
    ){
        midnight_variables
            .visits_mut()
            .ledger
            .iter_mut()
            .filter(|event|event.id == Some(id))
            .for_each(|event|*event = VisitLedgerEvent::new_id(id, generator.clone()));
    }
    pub fn push_ledger_event_id<T: Fn(&mut Vec<Visit>) + Send + Clone + 'static>(midnight_variables: &mut OnMidnightFold, id: VisitLedgerEventID, generator: T){
        midnight_variables.visits_mut().ledger.push(VisitLedgerEvent::new_id(id, generator));
    }
    pub fn push_ledger_event<T: Fn(&mut Vec<Visit>) + Send + Clone + 'static>(midnight_variables: &mut OnMidnightFold, generator: T){
        midnight_variables.visits_mut().ledger.push(VisitLedgerEvent::new(generator));
    }
    pub fn add_visit(midnight_variables: &mut OnMidnightFold, visits: Visit){
        midnight_variables.visits_mut().ledger.push(VisitLedgerEvent::new(move |v|
            v.push(visits)
        ));
    }
    pub fn add_visits(midnight_variables: &mut OnMidnightFold, visits: impl Iterator<Item = Visit> + Clone + Send + 'static){
        midnight_variables.visits_mut().ledger.push(VisitLedgerEvent::new(move |v|
            v.extend(visits.clone())
        ));
    }

    //Only keeps elements where f is true
    pub fn retain(midnight_variables: &mut OnMidnightFold, f: impl FnMut(&Visit) -> bool + Clone + Send + 'static){
        midnight_variables.visits_mut().ledger.push(VisitLedgerEvent::new(move |v|
            v.retain(f.clone())
        ));
    }

    fn calculate_visits(&self)->Vec<Visit> {
        let mut out = Vec::new();
        for event in self.ledger.iter() {
            (event.generator)(&mut out);
        }
        out
    }


    pub fn into_iter(midnight_variables: &OnMidnightFold) -> impl Iterator<Item = Visit> + use<> {
        midnight_variables.visits().calculate_visits().into_iter()
    }
    // pub fn iter(midnight_variables: &OnMidnightFold) -> impl Iterator<Item = &Visit> {
    //     midnight_variables.visits().iter()
    // }
    // pub fn iter_mut(midnight_variables: &mut OnMidnightFold) -> impl Iterator<Item = &mut Visit>{
    //     midnight_variables.visits_mut().ledger.push(VisitLedgerEvent::new(move |v|
    //         v.iter_mut()
    //     ));
    // }
}

impl PlayerReference{
    /// Returns all vists where the player is the target
    pub fn all_direct_night_visitors_cloned(self, midnight_variables: &OnMidnightFold) -> impl Iterator<Item = PlayerReference> + use<> {
        Visits::into_iter(midnight_variables)
            .with_target(self)
            .with_direct()
            .map_visitor()
    }

    pub fn tracker_seen_players(self, midnight_variables: &OnMidnightFold) -> impl Iterator<Item = PlayerReference> {
        Visits::into_iter(midnight_variables)
            .with_visitor(self)
            .with_appeared(midnight_variables)
            .map_target()
    }
    pub fn lookout_seen_players(self, midnight_variables: &OnMidnightFold, lookout_visit: Visit) -> impl Iterator<Item = PlayerReference> {
        Visits::into_iter(midnight_variables)
            .with_target(self)
            .with_appeared(midnight_variables)
            .without_visit(lookout_visit)
            .map_visitor()
    }
}


impl Visits{
    pub fn default_target(midnight_variables: &OnMidnightFold, actor: PlayerReference, role: Role) -> Option<PlayerReference>{
        Self::into_iter(midnight_variables)
            .default_target(actor, role)
    }
    pub fn default_visit(midnight_variables: &OnMidnightFold, actor: PlayerReference, role: Role) -> Option<Visit>{
        Self::into_iter(midnight_variables)
            .default_role_visit(actor, role)
    }
}
pub trait NightVisitsIterator: Sized {
    type Item;

    fn with_visitor(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>;
    fn without_visitor(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>;
    fn with_target(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>;
    fn without_visit(self, visit: Visit) -> impl Iterator<Item = Self::Item>;
    fn with_alive_visitor(self, game: &Game) -> impl Iterator<Item = Self::Item>;
    fn with_insider_visitor(self, game: &Game, insider_group_id: InsiderGroupID) -> impl Iterator<Item = Self::Item>;
    fn with_loyalist_visitor(self, game: &Game, conclusion: GameConclusion) -> impl Iterator<Item = Self::Item>;
    fn without_loyalist_visitor(self, game: &Game, conclusion: GameConclusion) -> impl Iterator<Item = Self::Item>;
    fn with_investigatable(self) -> impl Iterator<Item = Self::Item>;
    fn with_direct(self) -> impl Iterator<Item = Self::Item>;
    fn with_tag(self, visit_tag: VisitTag) -> impl Iterator<Item = Self::Item>;
    fn without_tag(self, visit_tag: VisitTag) -> impl Iterator<Item = Self::Item>;

    fn map_visitor(self) -> impl Iterator<Item = PlayerReference>;
    fn map_target(self) -> impl Iterator<Item = PlayerReference>;
    fn map_tag(self) -> impl Iterator<Item = VisitTag>;

    fn default_role_visit(self, actor: PlayerReference, role: Role) -> Option<Self::Item>;
    fn default_role_visits(self, actor: PlayerReference, role: Role) -> impl Iterator<Item = Self::Item>;
    fn default_target(self, actor: PlayerReference, role: Role) -> Option<PlayerReference>;
    fn default_targets(self, actor: PlayerReference, role: Role) -> impl Iterator<Item = PlayerReference>;

    fn with_appeared(self, midnight_variables: &OnMidnightFold) -> impl Iterator<Item = Self::Item>;
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
    fn without_visitor(self, player: PlayerReference) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|v.borrow().visitor != player)
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
    fn without_tag(self, visit_tag: VisitTag) -> impl Iterator<Item = Self::Item>{
        self.filter(move |v|v.borrow().tag != visit_tag)
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

    fn default_role_visit(self, actor: PlayerReference, role: Role) -> Option<Self::Item>{
        self
            .default_role_visits(actor, role)
            .next()

    }
    fn default_role_visits(self, actor: PlayerReference, role: Role) -> impl Iterator<Item = Self::Item>{
        self
            .with_visitor(actor)
            .filter(move |v|
                if let VisitTag::Ability{
                    ability: AbilityID::Role { role: r, player },
                    ..
                } = v.borrow().tag {
                    r == role && player == actor
                } else {false}
            )
    }
    fn default_target(self, actor: PlayerReference, role: Role) -> Option<PlayerReference>{
        self
            .default_targets(actor, role)
            .next()
    }
    fn default_targets(self, actor: PlayerReference, role: Role) -> impl Iterator<Item = PlayerReference>{
        self
            .default_role_visits(actor, role)
            .map(|v|v.borrow().target)
    }

    fn with_appeared(self, midnight_variables: &OnMidnightFold) -> impl Iterator<Item = Self::Item> {
        self
            .filter(|v|
                if v.borrow().visitor.night_appeared_visits(midnight_variables) {v.borrow().tag == VisitTag::Appeared} else {!v.borrow().investigate_immune}
            )
    }
}