use serde::Serialize;
use crate::game::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mortician {
    cremations_remaining: u8,
    blocked: bool
}
impl Default for Mortician {
    fn default() -> Self {
        Self {
            cremations_remaining: 3,
            blocked: false
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;


impl RoleStateTrait for Mortician {
    type ClientAbilityState = Mortician;
    fn new_state(game: &mut Game) -> Self {
        Self{
            cremations_remaining: crate::game::role::common_role::standard_charges(game),
            blocked: false
        }
    }
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return;}
        let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::Mortician) else {return};
        Tags::add_tag(
            game,
            TagSetID::MorticianTag(actor_ref),
            target
        );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Mortician, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|p| *p != actor_ref)
                    .filter(|player| 
                        player.alive(game) &&
                        !Tags::has_tag(game, TagSetID::MorticianTag(actor_ref), *player)
                    )
                    .collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Mortician, 0),
            false
        )
    }
    fn on_grave_added(mut self, game: &mut Game, actor_ref: PlayerReference, grave_ref: GraveReference){
        if
            !BlockedComponent::blocked(game, actor_ref) &&
            !actor_ref.ability_deactivated_from_death(game) &&
            Tags::has_tag(game, TagSetID::MorticianTag(actor_ref), grave_ref.deref(game).player) &&
            self.cremations_remaining > 0
        {
            actor_ref.add_private_chat_message(game, ChatMessageVariant::PlayerRoleAndAlibi{
                player: grave_ref.deref(game).player,
                role: grave_ref.deref(game).player.role(game),
                will: grave_ref.deref(game).player.alibi(game).to_string(),
            });
            self.cremations_remaining = self.cremations_remaining.saturating_sub(1);

            grave_ref.deref_mut(game).information = GraveInformation::Obscured;
            
            actor_ref.edit_role_ability_helper(game, self);
        }
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Mortician) || fold.cancelled {return}
        Tags::add_viewer(game, TagSetID::MorticianTag(actor_ref), actor_ref);
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority){
        if !event.id.is_players_role(actor_ref, Role::Mortician) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        Tags::remove_viewer(game, TagSetID::MorticianTag(actor_ref), actor_ref);
    }
}
