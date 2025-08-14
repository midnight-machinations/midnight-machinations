use crate::game::{components::graves::{grave::GraveInformation, grave_reference::GraveReference}, Game};

use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct ObscuredGraves;

impl From<&ObscuredGraves> for ModifierType{
    fn from(_: &ObscuredGraves) -> Self {
        ModifierType::ObscuredGraves
    }
}

impl ModifierTrait for ObscuredGraves{
    fn on_grave_added(self, game: &mut Game, grave: GraveReference) {
        grave.deref_mut(game).information = GraveInformation::Obscured;
    }
}
