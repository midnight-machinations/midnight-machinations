use serde::{Deserialize, Serialize};

use crate::game::{components::graves::grave::Grave, Game};

pub type GraveIndex = u8;
#[derive(Clone, Copy, Debug)]
pub struct GraveReference{
    index: GraveIndex
}
impl GraveReference{
    pub fn new(game: &Game, index: u8)->Option<GraveReference> {
        if (index as usize) < game.graves.graves.len() {
            Some(GraveReference { index })
        }else{
            None
        }
    }
    /// # Safety
    /// index must be in bounds
    pub unsafe fn new_unchecked(index: u8)->GraveReference {
        GraveReference { index }
    }
    pub fn deref(self, game: &Game)->&Grave{
        unsafe {
            game.graves.graves.get_unchecked(self.index as usize)
        }
    }
    pub fn deref_mut(self, game: &mut Game)->&mut Grave{
        unsafe {
            game.graves.graves.get_unchecked_mut(self.index as usize)
        }
    }
    pub fn all_graves(game: &Game)->std::iter::Map<std::ops::Range<u8>, impl FnMut(u8) -> GraveReference>{
        unsafe{
            (0..game.graves.graves.len()
                .try_into()
                .expect("Game has less than 256 players")
            )
                .map(|i|GraveReference::new_unchecked(i))
        }
    }
}

impl Serialize for GraveReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_u8(self.index)
    }
}
impl<'a> Deserialize<'a> for GraveReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'a> {
        Ok(GraveReference {
            index: u8::deserialize(deserializer)?
        })
    }
}