use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{game::{components::insider_group::InsiderGroupID, game_conclusion::GameConclusion, phase::PhaseType, player::PlayerReference, role::Role, role_outline_reference::RoleOutlineReference, Game}, vec_set::VecSet};

use super::win_condition::WinCondition;

pub struct SynopsisTracker {
    player_synopses: Vec<PartialPlayerSynopsis>
}

impl SynopsisTracker {
    pub fn new(num_players: u8) -> Self {
        SynopsisTracker {
            player_synopses: (0..num_players).map(|_|
                PartialPlayerSynopsis {
                    crumbs: Vec::new()
                }
            ).collect(),
        }
    }

    pub fn get(game: &Game, conclusion: GameConclusion) -> Synopsis {
        Synopsis {
            player_synopses: game.synopsis_tracker.player_synopses.iter()
                .enumerate()
                .map(|(player_index, player_synopsis)|
                    player_synopsis.get(
                        #[expect(clippy::cast_possible_truncation, reason = "Game can only have 255 players")]
                        unsafe { PlayerReference::new_unchecked(player_index as u8) },
                        game,
                        conclusion
                    )
                ).collect(),
            conclusion
        }
    }

    fn player_synopses(game: &mut Game) -> &mut Vec<PartialPlayerSynopsis> {
        &mut game.synopsis_tracker.player_synopses
    }

    pub fn on_role_switch(game: &mut Game, player: PlayerReference, _: Role, _: Role) {
        SynopsisTracker::add_crumb_to_player(player, game);
    }

    pub fn on_convert(game: &mut Game, player: PlayerReference, _: WinCondition, _: WinCondition) {
        SynopsisTracker::add_crumb_to_player(player, game);
    }

    pub fn on_add_insider(game: &mut Game, player: PlayerReference, _: InsiderGroupID) {
        SynopsisTracker::add_crumb_to_player(player, game);
    }

    pub fn on_remove_insider(game: &mut Game, player: PlayerReference, _: InsiderGroupID) {
        SynopsisTracker::add_crumb_to_player(player, game);
    }

    fn add_crumb_to_player(player: PlayerReference, game: &mut Game) {
        let night = if matches!(game.current_phase().phase(), PhaseType::Night | PhaseType::Obituary) { 
            Some(game.day_number())
        } else {
            None
        };

        let role = player.role(game);
        let win_condition = player.win_condition(game).clone();
        let insider_groups = InsiderGroupID::all_groups_with_player(game, player);

        if let Some(ref mut synopsis) = SynopsisTracker::player_synopses(game).get_mut(player.index() as usize) {
            synopsis.add_crumb(SynopsisCrumb { night, role, win_condition, insider_groups });
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Synopsis {
    player_synopses: Vec<PlayerSynopsis>,
    conclusion: GameConclusion,
}

// Don't ask
impl PartialEq for Synopsis {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Synopsis {}

impl PartialOrd for Synopsis {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Synopsis {
    fn cmp(&self, _: &Self) -> Ordering {
        Ordering::Equal
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSynopsis {
    outline_assignment: RoleOutlineReference,
    crumbs: Vec<SynopsisCrumb>,
    won: bool
}

pub struct PartialPlayerSynopsis {
    crumbs: Vec<SynopsisCrumb>
}

impl PartialPlayerSynopsis {
    fn add_crumb(&mut self, crumb: SynopsisCrumb) {
        // Remove duplicates from each night
        if let Some((index, _)) = self.crumbs.iter()
            .enumerate()
            .find(|(_, c)| c.night.is_some() && c.night == crumb.night)
        {
            self.crumbs.drain(index..);
        }
        if self.crumbs.last().cloned() == Some(crumb.clone()) {
            self.crumbs.pop();
        }
        self.crumbs.push(crumb);
    }

    fn get(&self, player_ref: PlayerReference, game: &Game, conclusion: GameConclusion) -> PlayerSynopsis {
        PlayerSynopsis {
            crumbs: self.crumbs.clone(),
            won: player_ref.get_won_game(game, conclusion),
            #[expect(clippy::unwrap_used, reason = "Player must have an assignment")]
            outline_assignment: game.assignments.get(&player_ref).unwrap().0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SynopsisCrumb {
    night: Option<u8>,
    role: Role,
    win_condition: WinCondition,
    insider_groups: VecSet<InsiderGroupID>,
}