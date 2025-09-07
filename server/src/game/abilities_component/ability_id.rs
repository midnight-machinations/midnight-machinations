use crate::game::{
    abilities_component::{ability::Ability, Abilities}, player::PlayerReference, Game
};


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbilityID{
    Role{player: PlayerReference},
    Pitchfork,
    SyndicateGun
}
impl AbilityID{
    pub fn get<'a>(&self, game: &'a Game)->Option<&'a Ability>{
        game.abilities.abilities.get(self)
    }
    pub fn set(&self, game: &mut Game, new: Option<impl Into<Ability>>){
        Abilities::set_ability(game, self, new.map(|a|a.into()));
    }
    pub fn current_used_ids(game: &Game)->Box<[Self]>{
        game.abilities.abilities.iter().map(|(id,_)|id).cloned().collect()
    }
}