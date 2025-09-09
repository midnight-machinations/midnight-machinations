use serde::{Deserialize, Serialize};

use crate::game::{player::PlayerReference, role_list_generation::template::Template};

use super::{role_list::Outline, Game};


pub type OutlineIndex = u8;
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Default, PartialOrd, Ord)]
pub struct RoleOutlineReference{
    index: OutlineIndex
}

pub type AssignedTemplateAndPlayer = (Template, PlayerReference);

impl RoleOutlineReference{
    pub fn new(game: &Game, index: OutlineIndex) -> Option<RoleOutlineReference>{
        if (index as usize) >= game.settings.role_list.0.len() { return None;} 
        unsafe { Some(RoleOutlineReference::new_unchecked(index)) }
    }
    /// # Safety
    /// If the index is too high, there might not be a role outline with that index.
    /// Make sure that the index is valid: it should be less than the number of role outlines
    pub unsafe fn new_unchecked(index: OutlineIndex) -> RoleOutlineReference {
        RoleOutlineReference { index }
    }
    pub fn index(&self) -> OutlineIndex {
        self.index
    }

    pub fn deref<'a>(&self, game: &'a Game)->&'a Outline{
        unsafe {
            game.settings.role_list.0.get_unchecked(self.index as usize)
        }
    }
    pub fn deref_as_role_and_player_originally_generated(&self, game: &Game)->AssignedTemplateAndPlayer{
        game.assignments
            .iter()
            .find(|(_, assignment)| assignment.role_outline_reference.index == self.index)
            .map(|(player, assignment)| (assignment.template, *player))
            .expect("RoleOutlineReference does not correspond to any role in the game")
    }

    pub fn all_outlines(game: &Game) -> RoleOutlineReferenceIterator {
        RoleOutlineReferenceIterator {
            current: 0,
            end: game.settings.role_list.0.len().try_into().unwrap_or(OutlineIndex::MAX)
        }
    }
}


impl Serialize for RoleOutlineReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_u8(self.index)
    }
}
impl<'a> Deserialize<'a> for RoleOutlineReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'a> {
        Ok(RoleOutlineReference {
            index: u8::deserialize(deserializer)?
        })
    }
}


pub struct RoleOutlineReferenceIterator {
    current: OutlineIndex,
    end: OutlineIndex
}

impl Iterator for RoleOutlineReferenceIterator {
    type Item = RoleOutlineReference;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current >= self.end {
                None
            } else {
                let ret: RoleOutlineReference = RoleOutlineReference::new_unchecked(self.current);
                if let Some(new) = self.current.checked_add(1) {
                    self.current = new;
                } else {
                    return None;
                }
                Some(ret)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.end.saturating_sub(self.current) as usize;
        (size, Some(size))
    }
}

impl ExactSizeIterator for RoleOutlineReferenceIterator {}