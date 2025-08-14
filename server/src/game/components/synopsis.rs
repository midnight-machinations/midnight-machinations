use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::game::{game_conclusion::GameConclusion, phase::PhaseType, player::PlayerReference, role::Role, Game};

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
                        unsafe { PlayerReference::new_unchecked(player_index as u8).get_won_game(game, conclusion) }
                    )
                ).collect(),
            conclusion
        }
    }

    pub fn on_role_switch(game: &mut Game, player: PlayerReference, _: Role, role: Role) {
        let night = if matches!(game.current_phase().phase(), PhaseType::Night | PhaseType::Obituary) { 
            Some(game.day_number())
        } else {
            None
        };

        let win_condition = player.win_condition(game).clone();

        game.synopsis_tracker.add_crumb_to_player(player, night, role, win_condition);
    }

    pub fn on_convert(game: &mut Game, player: PlayerReference, _: WinCondition, win_condition: WinCondition) {
        let night = if matches!(game.current_phase().phase(), PhaseType::Night | PhaseType::Obituary) { 
            Some(game.day_number())
        } else {
            None
        };

        let role = player.role(game);
        
        game.synopsis_tracker.add_crumb_to_player(player, night, role, win_condition);
    }

    fn add_crumb_to_player(&mut self, player: PlayerReference, night: Option<u8>, role: Role, win_condition: WinCondition) {
        if let Some(ref mut synopsis) = self.player_synopses.get_mut(player.index() as usize) {
            synopsis.add_crumb(SynopsisCrumb { night, role, win_condition });
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

    fn get(&self, won: bool) -> PlayerSynopsis {
        PlayerSynopsis {
            crumbs: self.crumbs.clone(),
            won
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SynopsisCrumb {
    night: Option<u8>,
    role: Role,
    win_condition: WinCondition,
}