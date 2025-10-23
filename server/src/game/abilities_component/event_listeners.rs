use crate::game::{
    abilities_component::{ability::Ability, ability_id::AbilityID, ability_trait::{AbilityIDAndAbility, AbilityTrait, AbilityTraitOld}, Abilities}, 
    controllers::ControllerParametersMap, 
    event::prelude::*, 
    prelude::EventListener, Game
};

impl Abilities{
    pub fn on_event<E: EventData>(game: &mut Game, data: &E, fold: &mut E::FoldValue, priority: E::Priority)
        where dyn AbilityTrait: EventListener<E>
    {
        game
            .abilities
            .abilities
            .clone()
            .into_iter()
            .map(AbilityIDAndAbility::new)
            .for_each(|e|e.as_invokable().on_event(game, data, fold, priority));
    }
    pub fn on_ability_creation(game: &mut Game, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority == OnAbilityCreationPriority::CancelOrEdit {
            game.abilities.abilities.insert(event.id.clone(), fold.ability.clone());
        }
        if priority == OnAbilityCreationPriority::SetAbility{
            if fold.cancelled {
                game.abilities.abilities.remove(&event.id);
            }else{
                game.abilities.abilities.insert(event.id.clone(), fold.ability.clone());
                OnAbilityEdit::new(event.id.clone(), Some(fold.ability.clone())).as_invokable().invoke(game);
            }
        }

        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_ability_creation(game, event, fold, priority);
        }
    }
    pub fn on_ability_deletion(game: &mut Game, event: &OnAbilityDeletion, fold: &mut (), priority: OnAbilityDeletionPriority) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_ability_deletion(game, event, fold, priority);
        }

        if priority == OnAbilityDeletionPriority::DeleteAbility {
            game.abilities.abilities.remove(&event.id);
            OnAbilityEdit::new(event.id.clone(), None).as_invokable().invoke(game);
        }   
    }
    pub fn on_role_switch(game: &mut Game, event: &OnRoleSwitch, fold: &mut (), priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_role_switch(game, event, fold, priority);
        }
    }

    pub fn on_player_possessed(game: &mut Game, event: &OnPlayerPossessed, fold: &mut OnMidnightFold, priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_player_possessed(game, event, fold, priority);
        }
    }

    
    pub fn on_player_roleblocked(game: &mut Game, event: &OnPlayerRoleblocked, fold: &mut OnMidnightFold, priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_player_roleblocked(game, event, fold, priority)
        }
    }
    pub fn on_visit_wardblocked(game: &mut Game, event: &OnVisitWardblocked, fold: &mut OnMidnightFold, priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_visit_wardblocked(game, event, fold, priority)
        }
    }


    pub fn controller_parameters_map(game: &Game) -> ControllerParametersMap {
        ControllerParametersMap::combine(
            game.abilities.abilities
                .clone()
                .into_iter()
                .map(|a|a.0.controller_parameters_map(game))
        )
    }
}

impl AbilityID{
    fn on_ability_creation(&self, game: &mut Game, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        self.get_dyn_cloned_ability_expect(game).on_ability_creation(game, self, event, fold, priority);
    }
    fn on_ability_deletion(&self, game: &mut Game, event: &OnAbilityDeletion, fold: &mut (), priority: OnAbilityDeletionPriority) {
        self.get_dyn_cloned_ability_expect(game).on_ability_deletion(game, self, event, fold, priority);
    }
    fn on_role_switch(&self, game: &mut Game, event: &OnRoleSwitch, fold: &mut (), priority: ()){
        self.get_dyn_cloned_ability_expect(game).on_role_switch(game, self, event, fold, priority);
    }

    fn on_player_possessed(&self, game: &mut Game, event: &OnPlayerPossessed, fold: &mut OnMidnightFold, priority: ()){
        self.get_dyn_cloned_ability_expect(game).on_player_possessed(game, self, event, fold, priority);
    }
    pub fn on_player_roleblocked(&self, game: &mut Game, event: &OnPlayerRoleblocked, fold: &mut OnMidnightFold, priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_player_roleblocked(game, self, event, fold, priority);
    }
    pub fn on_visit_wardblocked(&self, game: &mut Game, event: &OnVisitWardblocked, fold: &mut OnMidnightFold, priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_visit_wardblocked(game, self, event, fold, priority);
    }

    fn controller_parameters_map(&self, game: &Game) -> ControllerParametersMap {
        self.get_dyn_cloned_ability_expect(game).controller_parameters_map(game, self)
    }

    
    fn get_dyn_cloned_ability_expect(&self, game: &Game)->Box<dyn AbilityTraitOld>{
        match self.get_ability(game).expect("Event called on abilityId not in event map should be impossible").clone() {
            Ability::Role(role_ability) => {Box::new(role_ability)},
            Ability::Pitchfork(pitchfork_ability) => {Box::new(pitchfork_ability)},
            Ability::SyndicateGun(syndicate_gun) => {Box::new(syndicate_gun)},
        }
    }
}