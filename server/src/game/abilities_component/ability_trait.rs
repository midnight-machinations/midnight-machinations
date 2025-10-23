use crate::game::{
    abilities_component::{ability::Ability, ability_id::AbilityID}, controllers::ControllerParametersMap, event::prelude::*, prelude::EventListener, Game
};

pub trait AbilityTraitOld {
    fn on_ability_creation(&self, _game: &mut Game, _id: &AbilityID, _event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, _priority: OnAbilityCreationPriority) {}
    fn on_ability_deletion(&self, _game: &mut Game, _id: &AbilityID, _event: &OnAbilityDeletion, _fold: &mut (), _priority: OnAbilityDeletionPriority) {}
    fn on_role_switch(&self, _game: &mut Game, _id: &AbilityID, _event: &OnRoleSwitch, _fold: &mut (), _priority: ()) {}
    fn on_player_possessed(&self, _game: &mut Game, _id: &AbilityID, _event: &OnPlayerPossessed, _fold: &mut OnMidnightFold, _priority: ()) {}
    fn on_player_roleblocked(&self, _game: &mut Game, _id: &AbilityID, _event: &OnPlayerRoleblocked, _fold: &mut OnMidnightFold, _priority: ()) {}
    fn on_visit_wardblocked(&self, _game: &mut Game, _id: &AbilityID, _event: &OnVisitWardblocked, _fold: &mut OnMidnightFold, _priority: ()) {}

    fn controller_parameters_map(&self, _game: &Game, _id: &AbilityID)  -> ControllerParametersMap {ControllerParametersMap::default()}
}
pub trait AbilityTrait:
    EventListener<OnMidnight> +
    EventListener<OnWhisper> +
    EventListener<OnGraveAdded> +
    EventListener<OnValidatedControllerInputReceived> +
    EventListener<OnControllerSelectionChanged> +
    EventListener<OnPhaseStart> +
    EventListener<BeforePhaseEnd> +
    EventListener<OnConcealRole> +
    EventListener<OnAddInsider> +
    EventListener<OnRemoveInsider> +
    EventListener<OnAnyDeath> +
{}

pub struct AbilityIDAndAbility<A>(AbilityID, A);
impl<A> AbilityIDAndAbility<A> {
    pub fn new((id, abilities): (AbilityID, A))->Self{
        Self(id, abilities)
    }
    pub fn id(&self)->&AbilityID{
        &self.0
    }
    pub fn ability(&self)->&A{
        &self.1
    }
}

impl AbilityIDAndAbility<Ability> {
    pub fn as_invokable(self)->Box<dyn AbilityTrait>{
        match self.1 {
            Ability::Role(role_ability) => Box::new(AbilityIDAndAbility(self.0, role_ability)),
            Ability::Pitchfork(pitchfork_ability) => Box::new(AbilityIDAndAbility(self.0, pitchfork_ability)),
            Ability::SyndicateGun(syndicate_gun) => Box::new(AbilityIDAndAbility(self.0, syndicate_gun)),
        }
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! impl_ability_events {
        ($ty:ty, $($event:ty),*) => {
            $(
                impl EventListener<$event> for $ty {}
            )*
            impl AbilityTrait for $ty {}
        }
    }
}