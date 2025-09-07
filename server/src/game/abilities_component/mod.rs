pub mod ability_id;
pub mod ability;
pub mod ability_trait;
pub mod event_listeners;

use crate::{
    game::{
        abilities::{pitchfork::PitchforkAbility, role_abilities::RoleAbility, syndicate_gun::SyndicateGun}, abilities_component::{
            ability::Ability, ability_id::AbilityID
        }, Assignments, Game
    },
    vec_map::{vec_map, VecMap}
};

pub struct Abilities{
    abilities: VecMap<AbilityID, Ability>
}
impl Abilities{
    pub fn new(assignments: &Assignments)->Self{
        let mut abilities = vec_map!(
            (AbilityID::Pitchfork, Ability::Pitchfork(PitchforkAbility::default())),
            (AbilityID::SyndicateGun, Ability::SyndicateGun(SyndicateGun::default()))
        );
        for (player, o) in assignments.iter(){
            let id = AbilityID::Role { player: *player };
            abilities.insert(id.clone(), Ability::RoleAbility(RoleAbility(*player, o.role.default_state())));
        }
        abilities.sort();
        Self{abilities}
    }
    pub fn set_default_abilties(game: &mut Game){
        for (id, _) in game.abilities.abilities.clone() {
            Abilities::set_ability(game, &id, Some(id.new_state(game)));
        }
    }
    pub fn set_ability(game: &mut Game, id: &AbilityID, new: Option<impl Into<Ability>>){
        if let Some(new) = new{
            game.abilities.abilities.insert(id.clone(), new.into());
            game.abilities.abilities.sort();
        }else{
            game.abilities.abilities.remove(id);
        }
    }
}
impl AbilityID{
    fn new_state(&self, game: &Game)->Ability{
        match self {
            AbilityID::Role { player } => {RoleAbility(*player, player.role(game).new_state(game)).into()},
            AbilityID::Pitchfork => {PitchforkAbility::new_state(game).into()},
            AbilityID::SyndicateGun => {SyndicateGun::default().into()},
        }
    }
}