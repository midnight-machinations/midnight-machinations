
use serde::Serialize;

use crate::game::ability_input::*;
use crate::game::chat::ChatMessageVariant;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::grave::GraveInformation;
use crate::game::phase::PhaseType;
use crate::game::{attack_power::DefensePower, player::PlayerReference};

use crate::game::visit::Visit;

use crate::game::Game;
use crate::vec_set::{vec_set, VecSet};
use super::{InsiderGroupID, Role, RoleStateImpl};


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Disguiser{
    pub current_target: Option<PlayerReference>,
    pub last_role_selection: Role,
}
impl Default for Disguiser {
    fn default() -> Self {
        Self { current_target: None, last_role_selection: Role::Disguiser }
    }
}
pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Disguiser {
    type ClientRoleState = Disguiser;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {

        if priority != OnMidnightPriority::Deception {return}
                
        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let Some(first_visit) = actor_visits.first() else {return};
        
        if !InsiderGroupID::in_same_group(game, actor_ref, first_visit.target) {return}

        self.current_target = Some(first_visit.target);
        self.last_role_selection = Self::disguised_role(&self, game, actor_ref);

        actor_ref.set_role_state(game, self);
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Disguiser, 0),
            false
        )
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Disguiser, 0))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|p|
                            p.alive(game) &&
                            InsiderGroupID::in_same_group(game, actor_ref, *p)
                        )
                        .collect(),
                    can_choose_duplicates: false,
                    max_players: Some(1)
                })
                .night_typical(actor_ref)
                .default_selection(PlayerListSelection::one(self.current_target))
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Disguiser, 1))
                .single_role_selection_typical(game, |_|true)
                .default_selection(RoleListSelection(vec!(self.last_role_selection)))
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .allow_players(self.players_with_disguiser_menu(actor_ref))
                .build_map()
        ])
    }
    fn on_any_death(mut self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference) {
        if
            self.current_target.is_some_and(|p|p == dead_player_ref) || 
            actor_ref == dead_player_ref
        {
            self.current_target = None;
            actor_ref.set_role_state(game, self);
        }
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        match phase {
            PhaseType::Night => {
                self.current_target = None;
                actor_ref.set_role_state(game, self);
            },
            _ => {}
        }
    }
    fn on_grave_added(self, game: &mut Game, actor_ref: PlayerReference, grave: crate::game::grave::GraveReference) {
        let grave_ref = grave;
        
        if
            self.current_target.is_some_and(|p|p == grave.deref(game).player) && (
                actor_ref.alive(game) ||
                self.current_target.is_some_and(|p|p == actor_ref)
            )
        {
            actor_ref.add_private_chat_message(game, ChatMessageVariant::PlayerRoleAndAlibi{
                player: grave.deref(game).player,
                role: grave.deref(game).player.role(game),
                will: grave.deref(game).player.alibi(game).to_string(),
            });

            
            let mut grave = grave_ref.deref(game).clone();
            *grave_ref.deref_mut(game) = match grave.information {
                GraveInformation::Normal{role: _, will, death_cause, death_notes} => {
                    grave.information = GraveInformation::Normal{
                        role: Self::disguised_role(&self, game, actor_ref),
                        will,
                        death_cause,
                        death_notes
                    };
                    grave
                },
                _ => grave
            };
        }
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}

impl Disguiser{
    fn players_with_disguiser_menu(&self, actor_ref: PlayerReference)->VecSet<PlayerReference>{
        let mut players = vec_set!(actor_ref);
        if let Some(disguised) = self.current_target{
            players.insert(disguised);
        }
        players
    }
    fn disguised_role(&self, game: &Game, actor_ref: PlayerReference)->Role{
        if let Some(role) = ControllerID::role(actor_ref, Role::Disguiser, 1)
            .get_role_list_selection(game)
            .and_then(|selection| selection.0.first().copied())
        {
            role
        }else{
            Role::Disguiser
        }
    }
}