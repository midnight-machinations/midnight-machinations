use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{game::{Game, chat::{ChatGroup, ChatMessage, ChatMessageVariant, MessageSender}, components::{graves::grave::Grave, insider_group::InsiderGroupID}, event::{on_any_death::OnAnyDeath, on_convert::OnConvert, on_grave_added::OnGraveAdded, on_role_switch::OnRoleSwitch}, game_conclusion::GameConclusion, phase::PhaseType, player::PlayerReference, role::Role, role_list_generation::OutlineAssignment, role_outline_reference::RoleOutlineReference}, vec_set::VecSet};

use super::win_condition::WinCondition;

pub struct SynopsisTracker {
    player_synopses: Vec<PartialPlayerSynopsis>
}

impl SynopsisTracker {
    pub fn new(num_players: u8) -> Self {
        SynopsisTracker {
            player_synopses: (0..num_players).map(|_|
                PartialPlayerSynopsis {
                    latest_alibi: String::new(),
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

    pub fn on_role_switch(game: &mut Game, event: &OnRoleSwitch, _fold: &mut (), _priority: ()) {
        SynopsisTracker::add_crumb_to_player(game, event.player, SynopsisCrumbDatum::RoleChange(event.new.role()));
    }

    pub fn on_convert(game: &mut Game, event: &OnConvert, _fold: &mut (), _priority: ()) {
        if event.old == event.new {
            return
        }
        SynopsisTracker::add_crumb_to_player(game, event.player, SynopsisCrumbDatum::WinConditionChange(event.new.clone()));
    }

    pub fn on_add_insider(game: &mut Game, player: PlayerReference, _: InsiderGroupID) {
        SynopsisTracker::add_crumb_to_player(game, player, SynopsisCrumbDatum::InsiderGroupChange(InsiderGroupID::all_groups_with_player(game, player)));
    }

    pub fn on_remove_insider(game: &mut Game, player: PlayerReference, _: InsiderGroupID) {
        SynopsisTracker::add_crumb_to_player(game, player, SynopsisCrumbDatum::InsiderGroupChange(InsiderGroupID::all_groups_with_player(game, player)));
    }

    pub fn on_any_death(game: &mut Game, event: &OnAnyDeath, _fold: &mut (), _priority: ()) {
        SynopsisTracker::add_crumb_to_player(game, event.dead_player, SynopsisCrumbDatum::Died);
    }

    pub fn on_grave_added(game: &mut Game, event: &OnGraveAdded, _fold: &mut (), _priority: ()) {
        let grave = event.grave.deref(game);
        SynopsisTracker::add_crumb_to_player(game, grave.player, SynopsisCrumbDatum::Grave(grave.clone()));
    }

    pub fn on_chat_message_added(game: &mut Game, _: PlayerReference, message: ChatMessage) {
        if 
            let ChatMessage{
                variant: ChatMessageVariant::Normal {
                    message_sender: MessageSender::Player { player },
                    text,
                    block: true
                },
                chat_group: Some(ChatGroup::All)
            } = message &&
            let Some(ref mut synopsis) = SynopsisTracker::player_synopses(game).get_mut(player.index() as usize)
        {
            synopsis.latest_alibi = text
        }
    }

    fn add_crumb_to_player(game: &mut Game, player: PlayerReference, datum: SynopsisCrumbDatum) {
        let night = if matches!(game.current_phase().phase(), PhaseType::Night | PhaseType::Obituary) { 
            Some(game.day_number())
        } else {
            None
        };

        if let Some(ref mut synopsis) = SynopsisTracker::player_synopses(game).get_mut(player.index() as usize) {
            synopsis.add_crumb(SynopsisCrumb { night, datum });
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

impl Synopsis {
    pub fn get_player_synopsis(&self, player_ref: PlayerReference) -> &PlayerSynopsis {
        self.player_synopses.get(player_ref.index() as usize).expect("Player synopses should be initialized with the same number as amount of players")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSynopsis {
    outline_assignment: OutlineAssignment,
    latest_alibi: String,
    crumbs: Vec<SynopsisCrumb>,
    #[serde(rename = "index")]
    player: PlayerReference,
    won: bool
}

pub struct PartialPlayerSynopsis {
    latest_alibi: String,
    crumbs: Vec<SynopsisCrumb>
}

impl PartialPlayerSynopsis {
    fn add_crumb(&mut self, crumb: SynopsisCrumb) {
        self.crumbs.push(crumb);
    }

    fn get(&self, player_ref: PlayerReference, game: &Game, conclusion: GameConclusion) -> PlayerSynopsis {
        PlayerSynopsis {
            crumbs: self.crumbs.clone(),
            latest_alibi: self.latest_alibi.clone(),
            won: player_ref.get_won_game(game, conclusion),
            player: player_ref,
            #[expect(clippy::unwrap_used, reason = "Player must have an assignment")]
            outline_assignment: game.assignments.get(&player_ref).unwrap().clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SynopsisCrumb {
    night: Option<u8>,
    #[serde(flatten)]
    datum: SynopsisCrumbDatum,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SynopsisCrumbDatum {
    RoleChange(Role),
    WinConditionChange(WinCondition),
    InsiderGroupChange(VecSet<InsiderGroupID>),
    Died,
    Grave(Grave),
}