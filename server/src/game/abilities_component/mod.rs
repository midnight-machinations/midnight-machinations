pub mod ability_id;
pub mod ability;
pub mod ability_trait;
pub mod event_listeners;

use crate::{
    game::{
        abilities::{pitchfork::PitchforkAbility, role_abilities::RoleAbility, syndicate_gun::SyndicateGun},
        abilities_component::{
            ability::Ability, ability_id::AbilityID
        },
        event::{
            on_ability_creation::OnAbilityCreation,
            on_ability_deletion::OnAbilityDeletion,
            on_ability_edit::OnAbilityEdit,
            Invokable as _, AsInvokable
        },
        Assignments,
        Game
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
            let id = AbilityID::Role { role: o.role, player: *player };
            abilities.insert(id.clone(), Ability::Role(RoleAbility(o.role.default_state())));
        }
        abilities.sort();
        Self{abilities}
    }
    pub fn set_abilities_states_to_default(game: &mut Game){
        for (id, _) in game.abilities.abilities.clone() {
            let new_state = id.new_state(game);
            Abilities::new_ability(game, &id, new_state);
        }
    }
    pub fn set_ability(game: &mut Game, id: &AbilityID, new: Option<impl Into<Ability>>){
        if let Some(new) = new{
            if game.abilities.abilities.contains(id){
                Self::edit_ability(game, id, new);
            }else{
                Self::new_ability(game, id, new);
            }
        }else{
            Self::delete_ability(game, id);
        }
    }
    pub fn new_ability(game: &mut Game, id: &AbilityID, new: impl Into<Ability>){
        Self::delete_ability(game, id);
        OnAbilityCreation::new(id.clone(), new.into()).as_invokable().invoke(game);
        game.abilities.abilities.sort();
    }
    pub fn delete_ability(game: &mut Game, id: &AbilityID){
        if game.abilities.abilities.contains(id) {
            OnAbilityDeletion::new(id.clone()).as_invokable().invoke(game);
        }
    }
    pub fn edit_ability(game: &mut Game, id: &AbilityID, new: impl Into<Ability>){
        let ability = new.into();
        game.abilities.abilities.insert(id.clone(), ability.clone());
        OnAbilityEdit::new(id.clone(), Some(ability)).as_invokable().invoke(game);
    }

    pub fn ids(game: &Game)->Box<[AbilityID]>{
        game.abilities.abilities.iter().map(|(id,_)|id).cloned().collect()
    }
}
impl AbilityID{
    fn new_state(&self, game: &mut Game)->Ability{
        match self {
            AbilityID::Role { role, .. } => {RoleAbility(role.new_state(game)).into()},
            AbilityID::Pitchfork => {PitchforkAbility::new_state(game).into()},
            AbilityID::SyndicateGun => {SyndicateGun::default().into()},
        }
    }
}