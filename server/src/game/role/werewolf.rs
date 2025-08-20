use rand::seq::SliceRandom;
use serde::Serialize;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::ChatMessageVariant;
use crate::game::components::night_visits::NightVisits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::tags::{TagSetID, Tags};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::{PlayerIndex, PlayerReference};
use crate::game::visit::{Visit, VisitTag};
use crate::game::phase::PhaseType;
use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, PlayerListSelection, GetClientRoleState, Role, RoleStateImpl};


#[derive(Clone, Debug, Default)]
pub struct Werewolf;

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

const ENRAGED_NUMERATOR: usize = 2;
const ENRAGED_DENOMINATOR: usize = 3;

impl RoleStateImpl for Werewolf {
    type ClientRoleState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Deception => {
                let visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(first_visit) = visits.first() else {return};

                let target_ref = first_visit.target;
                let enraged = 
                    Tags::tagged(game, TagSetID::WerewolfTracked(actor_ref))
                        .count()
                        .saturating_mul(ENRAGED_DENOMINATOR) >= 
                    PlayerReference::all_players(game)
                        .filter(|p|p.alive(game)||*p==actor_ref)
                        .count()
                        .saturating_mul(ENRAGED_NUMERATOR);

                if !enraged && target_ref.all_night_visits_cloned(midnight_variables).is_empty() {return}
                    
                NightVisits::all_visits_mut(midnight_variables)
                    .filter(|visit| 
                        visit.visitor == actor_ref && visit.target == target_ref && visit.tag == VisitTag::Role{role: Role::Werewolf, id: 0}
                    ).for_each(|visit| {
                        visit.attack = true;
                    });
            }
            OnMidnightPriority::Kill => {
                let visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(werewolf_visit) = visits.first() else {return};
                let target_ref = werewolf_visit.target;

                //If player is untracked, track them
                if !Tags::has_tag(game, TagSetID::WerewolfTracked(actor_ref), target_ref) {
                    self.track_player(game, actor_ref, target_ref);
                } else {
                    //Dont attack or rampage first night
                    if game.day_number() <= 1 {return}
                
                    //rampage target
                    for other_player in NightVisits::all_visits(midnight_variables).into_iter()
                        .filter(|visit|
                            *werewolf_visit != **visit &&
                            visit.target == target_ref
                        )
                        .map(|v|v.visitor)
                        .collect::<Vec<_>>()
                    {
                        other_player.try_night_kill_single_attacker(
                            actor_ref,
                            game,
                            midnight_variables,
                            GraveKiller::Role(Role::Werewolf),
                            AttackPower::ArmorPiercing,
                            true
                        );
                    }
                    
                    //If target visits or you are enraged, attack them
                    if werewolf_visit.attack {
                        target_ref.try_night_kill_single_attacker(
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

                    let mut players: Vec<PlayerIndex> = player_ref.tracker_seen_visits(game, midnight_variables).into_iter().map(|p|p.target.index()).collect();
                    players.shuffle(&mut rand::rng());

                    actor_ref.push_night_message(midnight_variables, 
                        ChatMessageVariant::WerewolfTrackingResult{
                            tracked_player: player_ref.index(), 
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
                if let Some(PlayerListSelection(target)) = ControllerID::role(actor_ref, Role::Werewolf, 1)
                    .get_player_list_selection(game)
                {
                    if let Some(target) = target.first() {
                        self.track_player(game, actor_ref, *target);
                    };
                };

                for player in Tags::tagged(game, TagSetID::WerewolfTracked(actor_ref)).iter() {
                    player.add_private_chat_message(game, ChatMessageVariant::WerewolfTracked);
                }
            },
            _ => {}
        }
    }

    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {
        Tags::add_viewer(game, TagSetID::WerewolfTracked(actor_ref), actor_ref);
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _new: super::RoleState, _old: super::RoleState) {
        if actor_ref != player {return}
        Tags::remove_viewer(game, TagSetID::WerewolfTracked(actor_ref), actor_ref);
    }

}
impl Werewolf{
    fn track_player(&self, game: &mut Game, actor: PlayerReference, target: PlayerReference){
        Tags::add_tag(game, TagSetID::WerewolfTracked(actor), target);
    }
}
impl GetClientRoleState<ClientRoleState> for Werewolf {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}