use std::collections::HashSet;

use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::ability_input::AvailablePlayerListSelection;
use crate::game::attack_power::DefensePower;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::tags::TagSetID;
use crate::game::components::tags::Tags;
use crate::game::event::on_midnight::MidnightVariables;
use crate::game::event::on_midnight::OnMidnightPriority;
use crate::game::game_conclusion::GameConclusion;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use crate::vec_set::VecSet;
use super::RoleState;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Seeker {
    pub followers: HashSet<PlayerReference>,
    pub new_follower: Option<PlayerReference>,
    pub left_town: bool
}

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Seeker {
    type ClientRoleState = Seeker;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        for follower in self.followers.iter() {
            actor_ref.possess_night_action(game, midnight_variables, priority, Some(*follower));
        }
        match (priority, &self.new_follower) {
            (OnMidnightPriority::Investigative, _) => {
                let Some(hiding_spot) = actor_ref.untagged_night_visits(midnight_variables).first().map(|v|v.target) else {return};
                let Some(&hider) = ControllerID::role(actor_ref, Role::Seeker, 0)
                    .get_player_list_selection(game)
                    .cloned()
                    .unwrap_or_default()
                    .0.first()
                        else {return};
                if hiding_spot.all_night_visitors_cloned(midnight_variables).iter().any(|p|*p==hider) {
                    self.new_follower = Some(hider);
                    actor_ref.push_night_message(midnight_variables, ChatMessageVariant::SeekerCaught {
                        hider: hider.index(),
                        #[expect(clippy::cast_possible_truncation, reason="come on")]
                        //2 not 3 because the player you just caught isn't in the list yet
                        players_left: 2i8.saturating_sub(self.followers.len() as i8),
                        role_state_win_con: actor_ref.win_condition(game).is_role_state()
                    });
                    hider.push_night_message(midnight_variables, ChatMessageVariant::CaughtBySeeker);
                    actor_ref.set_role_state(game, self);
                }
            }
            (OnMidnightPriority::FinalizeNight, Some(follower)) => {
                //not self.won because if their win condition is different they should not leave the game 
                //but doesn't check for role state won because its easier to explain in the manual as any win con and it really doesn't matter for any of the other win cons.
                self.followers.insert(*follower);
                Tags::add_tag(game, TagSetID::Follower(actor_ref), *follower);
                self.new_follower = None;
                if actor_ref.get_won_game(game) {
                    actor_ref.leave_town(game);
                    if let Some(&apprentice) = self.followers
                        .iter()
                        .filter(|p|
                            p.win_condition(game).is_loyalist_for(GameConclusion::Town)
                        ).collect::<Vec<&PlayerReference>>()
                        .choose(&mut rand::rng())
                        .copied()
                        .or_else(||
                            self.followers.iter()
                                .collect::<Vec<&PlayerReference>>()
                                .choose(&mut rand::rng())
                                .copied()
                        ) {
                            apprentice.set_night_convert_role_to(midnight_variables, Some(Role::Seeker.default_state()));
                    }
                }
                actor_ref.set_role_state(game, self);
            }
            _=>(),
        }
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            // Mark
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Seeker, 0))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|p|
                            p.alive(game) && 
                            *p != actor_ref &&
                            !self.followers.contains(p)
                        )
                        .collect::<VecSet<_>>(),
                    can_choose_duplicates: false,
                    max_players: Some(1)
                })
                .night_typical(actor_ref)
                .add_grayed_out_condition(self.won())
                .build_map(),
            // Camp
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Seeker, 1))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|p|p.alive(game))
                        .collect::<VecSet<_>>(),
                    can_choose_duplicates: false,
                    max_players: Some(1)
                })
                //not self.won because if their win condition is different they still be able to possess players
                //but doesn't check for role state won because its easier to explain in the manual as any win con and it really doesn't matter for any of the other ones.
                .add_grayed_out_condition(actor_ref.get_won_game(game))
                .night_typical(actor_ref)
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Seeker, 1),
            false
        )
    }
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {
        Tags::add_viewer(game, TagSetID::Follower(actor_ref), actor_ref);
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _new: RoleState, _old: RoleState) {
        if actor_ref != player {return}
        Tags::delete_tag(game, TagSetID::Follower(actor_ref));
    }
    // fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player: PlayerReference) {
    //     if actor_ref == dead_player && self.left_town {
    //         self.followers
    //             .iter()
    //             .collect::<Vec<&PlayerReference>>()
    //             .choose(&mut rand::rng())
    //             .inspect(|p|p.set_role(game, Role::Seeker));
    //     }
    // }
}

impl Seeker {
    pub fn won(&self) -> bool {
        self.followers.len() >= 3 ||
        self.new_follower.is_some_and(|f|
            self.followers.len() == 2 && !self.followers.contains(&f)
        )
    }
}