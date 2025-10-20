use serde::Serialize;
use crate::game::components::night_visits::Visits;
use crate::game::controllers::ControllerParametersMap;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::components::poison::{Poison, PoisonAlert};
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority};
use crate::game::event::on_midnight::{OnMidnightFold, OnMidnightPriority};
use crate::game::components::tags::{TagSetID, Tags};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::Game;
use super::{ControllerID, GetClientAbilityState, Role, RoleStateTrait};

#[derive(Debug, Clone, Default)]
pub struct Spiral;

#[derive(Debug, Clone, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Spiral {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Poison { return };
        
        if Tags::tagged(game, TagSetID::UzumakiSpiral(actor_ref)).is_empty() && game.day_number() > 1 {
            if let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Spiral) {
                target_ref.try_night_kill_single_attacker(
                    actor_ref,
                    game,
                    midnight_variables,
                    GraveKiller::Role(Role::Spiral),
                    AttackPower::ArmorPiercing,
                    true
                );
                Spiral::spiral_visitors(game, midnight_variables, actor_ref, target_ref);
            }
        } else {
            for spiraling_player in Tags::tagged(game, TagSetID::UzumakiSpiral(actor_ref)).clone() {
                Spiral::remove_player_spiraling(game, actor_ref, spiraling_player);
                Spiral::spiral_visitors(game, midnight_variables, actor_ref, spiraling_player);
            }
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Spiral, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1 || !Tags::tagged(game, TagSetID::UzumakiSpiral(actor_ref)).is_empty())
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Spiral, 0),
            true
        )
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Spiral) || fold.cancelled {return}
        Tags::add_viewer(game, TagSetID::UzumakiSpiral(actor_ref), actor_ref);
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Spiral) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        Tags::remove_viewer(game, TagSetID::UzumakiSpiral(actor_ref), actor_ref);
    }
}

impl Spiral {
    fn start_player_spiraling(game: &mut Game, midnight_variables: &mut OnMidnightFold, actor_ref: PlayerReference, target_ref: PlayerReference) {
        if target_ref == actor_ref {return}
        let attackers = vec![actor_ref].into_iter().collect();
        Poison::poison_player(game,
            midnight_variables,
            target_ref, 
            AttackPower::ArmorPiercing, 
            GraveKiller::Role(Role::Spiral), 
            attackers, 
            true, 
            PoisonAlert::NoAlert,
        );

        Tags::add_tag(game, TagSetID::UzumakiSpiral(actor_ref), target_ref);
    }

    fn remove_player_spiraling(game: &mut Game, actor_ref: PlayerReference, target_ref: PlayerReference) {
        Tags::remove_tag(game, TagSetID::UzumakiSpiral(actor_ref), target_ref);
    }

    fn spiral_visitors(game: &mut Game, midnight_variables: &mut OnMidnightFold, actor_ref: PlayerReference, target: PlayerReference) {
        for visitor_to_spiraling in target.all_direct_night_visitors_cloned(midnight_variables)
            .filter(|other_player_ref|
                other_player_ref.alive(game) &&
                *other_player_ref != target // Let doctor self-heal
            ).collect::<Vec<PlayerReference>>()
        {
            Spiral::start_player_spiraling(game, midnight_variables, actor_ref, visitor_to_spiraling);
        }
    }
}

impl GetClientAbilityState<ClientRoleState> for Spiral {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}