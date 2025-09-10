use serde::{Serialize, Deserialize};
use crate::game::Game;
use super::Player;

pub type PlayerIndex = u8;
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Default, PartialOrd, Ord)]
pub struct PlayerReference {
    index: PlayerIndex
}

#[derive(Debug)]
pub struct InvalidPlayerReferenceError;

impl PlayerReference{
    pub fn new(game: &Game, index: PlayerIndex) -> Result<PlayerReference, InvalidPlayerReferenceError>{
        if (index as usize) >= game.players.len() { return Err(InvalidPlayerReferenceError);} 
        // This unsafe should be fine because we just checked that the index is valid
        unsafe {
            Ok(PlayerReference::new_unchecked(index))
        }
    }
    /// # Safety
    /// Check to make sure the index is less than the number of players in the game,
    /// otherwise, this could cause a panic.
    pub unsafe fn new_unchecked(index: PlayerIndex) -> PlayerReference {
        PlayerReference { index }
    }
    pub fn deref<'a>(&self, game: &'a Game)->&'a Player{
        unsafe { 
            game.players.get_unchecked(self.index as usize)
        }
    }
    pub fn deref_mut<'a>(&self, game: &'a mut Game)->&'a mut Player{
        unsafe {
            game.players.get_unchecked_mut(self.index as usize)
        }
    }
    pub fn index(&self) -> PlayerIndex {
        self.index
    }
    

    pub fn all_players(game: &Game) -> PlayerReferenceIterator {
        unsafe { Self::all_players_from_count(
            game.players.len().try_into().unwrap_or(u8::MAX)
        )}
    }
    /// # Safety
    /// player_count is less than or equal to the number of players in the game
    pub unsafe fn all_players_from_count(player_count: u8) -> PlayerReferenceIterator {
        PlayerReferenceIterator {
            current: 0,
            end: player_count
        }
    }
}


impl From<PlayerReference> for PlayerIndex {
    fn from(player_ref: PlayerReference) -> PlayerIndex {
        player_ref.index()
    }
}


pub struct PlayerReferenceIterator {
    current: PlayerIndex,
    end: PlayerIndex
}

impl Iterator for PlayerReferenceIterator {
    type Item = PlayerReference;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            None
        } else {
            // This unsafe should be fine as long as the iterator itself is fine
            let ret = unsafe {PlayerReference::new_unchecked(self.current)};
            if let Some(new) = self.current.checked_add(1) {
                self.current = new;
            } else {
                return None
            }
            Some(ret)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.end.saturating_sub(self.current) as usize;
        (size, Some(size))
    }
}

impl ExactSizeIterator for PlayerReferenceIterator {}

impl Serialize for PlayerReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_u8(self.index)
    }
}
impl<'a> Deserialize<'a> for PlayerReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'a> {
        Ok(PlayerReference {
            index: u8::deserialize(deserializer)?
        })
    }
}