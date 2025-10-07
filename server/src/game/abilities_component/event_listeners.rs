use crate::game::{
    abilities::role_abilities::RoleAbility,
    abilities_component::{ability::Ability, ability_id::AbilityID, ability_trait::AbilityTrait, Abilities}, chat::ChatMessageVariant,
    controllers::ControllerParametersMap,
    event::{
        before_phase_end::BeforePhaseEnd, on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority}, on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority}, on_add_insider::OnAddInsider, on_any_death::OnAnyDeath, on_conceal_role::OnConcealRole, on_controller_selection_changed::OnControllerSelectionChanged, on_grave_added::OnGraveAdded, on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority}, on_phase_start::OnPhaseStart, on_remove_insider::OnRemoveInsider, on_role_switch::OnRoleSwitch, on_validated_ability_input_received::OnValidatedControllerInputReceived, on_whisper::{OnWhisper, WhisperFold, WhisperPriority}
    },
    Game
};

impl Abilities{
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_midnight(game, _event, midnight_variables, priority)
        }
    }
    pub fn on_whisper(game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_whisper(game, event, fold, priority)
        }
    }
    pub fn on_grave_added(game: &mut Game, event: &OnGraveAdded, fold: &mut (), priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_grave_added(game, event, fold, priority)
        }
    }
    pub fn on_validated_ability_input_received(game: &mut Game, event: &OnValidatedControllerInputReceived, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_validated_ability_input_received(game, event, fold, priority)
        }
    }
    pub fn on_controller_selection_changed(game: &mut Game, event: &OnControllerSelectionChanged, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_controller_selection_changed(game, event, fold, priority)
        }
    }
    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_phase_start(game, event, _fold, _priority);
        }
    }
    pub fn before_phase_end(game: &mut Game, event: &BeforePhaseEnd, _fold: &mut (), _priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.before_phase_end(game, event, _fold, _priority);
        }
    }
    pub fn on_conceal_role(game: &mut Game, event: &OnConcealRole, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_conceal_role(game, event, fold, priority);
        }
    }
    pub fn on_add_insider(game: &mut Game, event: &OnAddInsider, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_add_insider(game, event, fold, priority);
        }
    }
    pub fn on_remove_insider(game: &mut Game, event: &OnRemoveInsider, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_remove_insider(game, event, fold, priority);
        }
    }
    pub fn on_any_death(game: &mut Game, event: &OnAnyDeath, fold: &mut (), priority: ()) {
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_any_death(game, event, fold, priority);
        }
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
                if
                    let Ability::RoleAbility(RoleAbility(role)) = &fold.ability &&
                    let Some(player) = event.id.get_player_from_role_id() &&
                    role.role().should_inform_player_of_assignment()
                {
                    player.add_private_chat_message(game, ChatMessageVariant::RoleAssignment{role: role.role()});
                }
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
        }   
    }
    pub fn on_role_switch(game: &mut Game, event: &OnRoleSwitch, fold: &mut (), priority: ()){
        for (id, _ability) in game.abilities.abilities.clone() {
            id.on_role_switch(game, event, fold, priority);
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
    fn on_midnight(&self, game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        self.get_dyn_cloned_ability_expect(game).on_midnight(game, self, _event, midnight_variables, priority);
    }
    fn on_whisper(&self, game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority){
        self.get_dyn_cloned_ability_expect(game).on_whisper(game, self, event, fold, priority);
    }
    fn on_grave_added(&self, game: &mut Game, event: &OnGraveAdded, fold: &mut (), priority: ()){
        self.get_dyn_cloned_ability_expect(game).on_grave_added(game, self, event, fold, priority);
    }
    fn on_validated_ability_input_received(&self, game: &mut Game, event: &OnValidatedControllerInputReceived, fold: &mut (), priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_validated_ability_input_received(game, self, event, fold, priority);
    }
    fn on_controller_selection_changed(&self, game: &mut Game, event: &OnControllerSelectionChanged, fold: &mut (), priority: ()){
        self.get_dyn_cloned_ability_expect(game).on_controller_selection_changed(game, self, event, fold, priority);
    }
    fn on_phase_start(&self, game: &mut Game, event: &OnPhaseStart, fold: &mut (), priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_phase_start(game, self, event, fold, priority);
    }
    fn before_phase_end(&self, game: &mut Game, event: &BeforePhaseEnd, fold: &mut (), priority: ()) {
        self.get_dyn_cloned_ability_expect(game).before_phase_end(game, self, event, fold, priority);
    }
    fn on_conceal_role(&self, game: &mut Game, event: &OnConcealRole, fold: &mut (), priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_conceal_role(game, self, event, fold, priority);
    }
    fn on_add_insider(&self, game: &mut Game, event: &OnAddInsider, fold: &mut (), priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_add_insider(game, self, event, fold, priority);
    }
    fn on_remove_insider(&self, game: &mut Game, event: &OnRemoveInsider, fold: &mut (), priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_remove_insider(game, self, event, fold, priority);
    }
    fn on_any_death(&self, game: &mut Game, event: &OnAnyDeath, fold: &mut (), priority: ()) {
        self.get_dyn_cloned_ability_expect(game).on_any_death(game, self, event, fold, priority);
    }
    fn on_ability_creation(&self, game: &mut Game, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        self.get_dyn_cloned_ability_expect(game).on_ability_creation(game, self, event, fold, priority);
    }
    fn on_ability_deletion(&self, game: &mut Game, event: &OnAbilityDeletion, fold: &mut (), priority: OnAbilityDeletionPriority) {
        self.get_dyn_cloned_ability_expect(game).on_ability_deletion(game, self, event, fold, priority);
    }
    fn on_role_switch(&self, game: &mut Game, event: &OnRoleSwitch, fold: &mut (), priority: ()){
        self.get_dyn_cloned_ability_expect(game).on_role_switch(game, self, event, fold, priority);
    }


    fn controller_parameters_map(&self, game: &Game) -> ControllerParametersMap {
        self.get_dyn_cloned_ability_expect(game).controller_parameters_map(game, self)
    }

    
    fn get_dyn_cloned_ability_expect(&self, game: &Game)->Box<dyn AbilityTrait>{
        match self.get_ability(game).expect("Event called on abilityId not in event map should be impossible").clone() {
            Ability::RoleAbility(role_ability) => {Box::new(role_ability)},
            Ability::Pitchfork(pitchfork_ability) => {Box::new(pitchfork_ability)},
            Ability::SyndicateGun(syndicate_gun) => {Box::new(syndicate_gun)},
        }
    }
}