use serde::{Deserialize, Serialize};

use crate::game::{components::graves::{grave::GraveInformation, grave_reference::GraveReference}, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ObscuredGraves;

impl From<&ObscuredGraves> for ModifierID{
    fn from(_: &ObscuredGraves) -> Self {
        ModifierID::ObscuredGraves
    }
}

impl ModifierStateImpl for ObscuredGraves{
    fn on_grave_added(self, game: &mut Game, grave: GraveReference) {
        grave.deref_mut(game).information = GraveInformation::Obscured;
    }
}
