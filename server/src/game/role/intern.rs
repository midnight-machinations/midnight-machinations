use rand::seq::IteratorRandom;
use serde::Serialize;
use crate::{game::prelude::*, vec_set::VecSet};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Intern{
    current_role: Option<AbilityID>,
    next_role_pool: VecSet<Role>
}

impl RoleStateTrait for Intern {
    type ClientAbilityState = Intern;
    fn new_state(game: &mut Game) -> Self {
        Intern{
            current_role: None,
            next_role_pool: Self::generate_role_pool(game)
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Intern, 0))
            .available_selection(AvailableRoleListSelection{
                available_roles: self.next_role_pool,
                can_choose_duplicates: false,
                max_roles: Some(1)
            })
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn create_visits_initialize_night(self, game: &Game, _id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Intern, 0),
            false
        )
    }
    fn on_ability_creation(mut self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect {return}
        if event.id != (AbilityID::Role { role: Role::Intern, player: actor_ref }) {return}
        //give random role
        if let Some(chosen_role) = Self::generate_role_pool(game).into_iter().choose(&mut game.rng)
        {
            self.current_role = Some(AbilityID::Role { role: chosen_role, player: actor_ref });
            let new_state = chosen_role.new_state(game);
            AbilityID::Role { role: chosen_role, player: actor_ref }.new_role_ability(game, new_state);
            actor_ref.edit_role_ability_helper(game, self);
        }

    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        if phase != PhaseType::Obituary {return}
        if actor_ref.ability_deactivated_from_death(game) {return}
        if BlockedComponent::blocked(game, actor_ref) {return}

        //remove previous role
        if let Some(ability_id) = self.current_role { ability_id.delete_ability(game) }
        self.current_role = None;

        //give new role
        if let Some(chosen_role) = ControllerID::role(actor_ref, Role::Intern, 0)
            .get_role_list_selection_first(game)
            .copied()
            .or_else(|| self.next_role_pool.into_iter().choose(&mut game.rng))
        {
            self.current_role = Some(AbilityID::Role { role: chosen_role, player: actor_ref });
            let new_state = chosen_role.new_state(game);
            AbilityID::Role { role: chosen_role, player: actor_ref }.new_role_ability(game, new_state);
        }


        //give new role pool
        self.next_role_pool = Self::generate_role_pool(game);

        actor_ref.edit_role_ability_helper(game, self);
    }
}

impl Intern{
    fn generate_role_pool(game: &mut Game)->VecSet<Role>{
        game.settings.enabled_roles
            .clone()
            .into_iter()
            .filter(|r|RoleSet::TownCommon.get_roles().contains(r))
            .filter(|r|matches!(r, Role::Engineer))
            .choose_multiple(&mut game.rng, 3)
            .into_iter()
            .collect()
    }
}