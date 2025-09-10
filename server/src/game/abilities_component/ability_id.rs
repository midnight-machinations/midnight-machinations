use crate::game::{
    abilities_component::{ability::Ability, Abilities}, player::PlayerReference, role::Role, Game
};


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbilityID{
    Role{role: Role, player: PlayerReference},
    Pitchfork,
    SyndicateGun
}
impl AbilityID{
    pub fn get_ability<'a>(&self, game: &'a Game)->Option<&'a Ability>{
        game.abilities.abilities.get(self)
    }
    pub fn set_ability(&self, game: &mut Game, new: Option<impl Into<Ability>>){
        Abilities::set_ability(game, self, new);
    }
    pub fn new_ability(&self, game: &mut Game, new: impl Into<Ability>){
        Abilities::new_ability(game, self, new);
    }
    pub fn delete_ability(&self, game: &mut Game){
        Abilities::delete_ability(game, self);
    }
    pub fn edit_ability(&self, game: &mut Game, new: impl Into<Ability>){
        Abilities::edit_ability(game, self, new);
    }
    pub fn current_used_ids(game: &Game)->Box<[Self]>{
        game.abilities.abilities.iter().map(|(id,_)|id).cloned().collect()
    }

    pub fn is_players_role(&self, player: PlayerReference, role: Role)->bool{
        if let Self::Role { role: role_on_id, player: player_on_id } = self {
            *player_on_id == player && *role_on_id == role
        }else{
            false
        }
    }
}