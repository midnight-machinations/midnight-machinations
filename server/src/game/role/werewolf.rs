use rand::seq::SliceRandom;
use serde::Serialize;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::ChatMessageVariant;
use crate::game::components::night_visits::{NightVisitsIterator, Visits};
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::tags::{TagSetID, Tags};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::phase::PhaseType;
use crate::game::Game;
use crate::game::abilities_component::ability_id::AbilityID;
use super::{ControllerID, ControllerParametersMap, PlayerListSelection, GetClientAbilityState, Role, RoleStateTrait};


#[derive(Clone, Debug, Default)]
pub struct Werewolf;

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

const ENRAGED_NUMERATOR: usize = 2;
const ENRAGED_DENOMINATOR: usize = 3;

impl RoleStateTrait for Werewolf {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Deception => {
                let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::Werewolf) else {return};

                let enraged = 
                    Tags::tagged(game, TagSetID::WerewolfTracked(actor_ref))
                        .count()
                        .saturating_mul(ENRAGED_DENOMINATOR) >= 
                    PlayerReference::all_players(game)
                        .filter(|p|p.alive(game)||*p==actor_ref)
                        .count()
                        .saturating_mul(ENRAGED_NUMERATOR);

                if enraged || !Visits::into_iter(midnight_variables)
                    .with_visitor(target)
                    .with_direct()
                    .collect::<Box<[Visit]>>()
                    .is_empty()
                {
                    Visits::iter_mut(midnight_variables)
                        .default_visit(actor_ref, Role::Werewolf)
                        .into_iter()
                        .for_each(|v|v.attack = true);
                }
            }
            OnMidnightPriority::Kill => {
                let Some(my_visit) = Visits::default_visit(midnight_variables, actor_ref, Role::Werewolf) else {return};

                //If player is untracked, track them
                if !Tags::has_tag(game, TagSetID::WerewolfTracked(actor_ref), my_visit.target) {
                    self.track_player(game, actor_ref, my_visit.target);
                } else {
                    //Dont attack or rampage first night
                    if game.day_number() <= 1 {return}
                        
                    //rampage target
                    my_visit.target.rampage(
                        game,
                        midnight_variables,
                        actor_ref,
                        GraveKiller::Role(Role::Werewolf),
                        AttackPower::ArmorPiercing,
                        true,
                        |v|*v != my_visit
                    );
                    
                    //If target visits or you are enraged, attack them
                    if my_visit.attack {
                        my_visit.target.try_night_kill_single_attacker(
                            actor_ref,
                            game,
                            midnight_variables,
                            GraveKiller::Role(Role::Werewolf),
                            AttackPower::ArmorPiercing,
                            true
                        );
                    } 
                }
                
            },
            OnMidnightPriority::Investigative => {
                //track sniffed players visits

                Tags::tagged(game, TagSetID::WerewolfTracked(actor_ref))
                    .into_iter()
                    .for_each(|player_ref|{

                    let mut players: Vec<PlayerReference> = player_ref.tracker_seen_players(midnight_variables).collect();
                    players.shuffle(&mut game.rng);

                    actor_ref.push_night_message(midnight_variables, 
                        ChatMessageVariant::WerewolfTrackingResult{
                            tracked_player: player_ref, 
                            players
                        }
                    );
                });
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Werewolf, 0))
                .single_player_selection_typical(actor_ref, false, true)
                .night_typical(actor_ref)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Werewolf, 1))
                .single_player_selection_typical(actor_ref, false, true)
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .reset_on_phase_start(PhaseType::Night)
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Werewolf, 0),
            false
        )
    }

    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {

                //Mark chosen player as tracked on phase start: night
                if 
                    let Some(PlayerListSelection(target)) = ControllerID::role(actor_ref, Role::Werewolf, 1)
                        .get_player_list_selection(game) &&
                    let Some(target) = target.first()
                {
                        self.track_player(game, actor_ref, *target);
                };

                for player in Tags::tagged(game, TagSetID::WerewolfTracked(actor_ref)).iter() {
                    player.add_private_chat_message(game, ChatMessageVariant::WerewolfTracked);
                }
            },
            _ => {}
        }
    }

    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Werewolf) || fold.cancelled {return}
        Tags::add_viewer(game, TagSetID::WerewolfTracked(actor_ref), actor_ref);
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Werewolf) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        Tags::remove_viewer(game, TagSetID::WerewolfTracked(actor_ref), actor_ref);
    }

}
impl Werewolf{
    fn track_player(&self, game: &mut Game, actor: PlayerReference, target: PlayerReference){
        Tags::add_tag(game, TagSetID::WerewolfTracked(actor), target);
    }
}
impl GetClientAbilityState<ClientRoleState> for Werewolf {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}