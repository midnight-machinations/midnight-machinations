pub mod ability_id;
pub mod ability;
pub mod ability_trait;

use crate::{
    game::{
        abilities::role_abilities::RoleAbility, abilities_component::{
            ability::Ability, ability_id::AbilityID
        }, Assignments, Game
    },
    vec_map::VecMap
};

pub struct Abilities{
    abilities: VecMap<AbilityID, Ability>
}
impl Abilities{
    pub fn new(assignments: &Assignments)->Self{
        let mut out = Self{
            abilities: VecMap::new(),
        };
        for (player, o) in assignments.iter(){
            let id = AbilityID::Role { player: *player };
            out.abilities.insert(id.clone(), Ability::Role(RoleAbility(*player, o.role.default_state())));
        }
        out
    }
    pub fn set_ability(game: &mut Game, id: &AbilityID, new: Option<impl Into<Ability>>){
        if let Some(new) = new{
            game.abilities.abilities.insert(id.clone(), new.into());
        }else{
            game.abilities.abilities.remove(id);
        }
    }
}