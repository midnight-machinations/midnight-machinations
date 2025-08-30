use serde::{Deserialize, Serialize};

use crate::game::{components::graves::{grave::{GraveDeathCause, GraveInformation}, grave_reference::GraveReference}, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NoDeathCause;

impl From<&NoDeathCause> for ModifierID{
    fn from(_: &NoDeathCause) -> Self {
        ModifierID::NoDeathCause
    }
}
impl ModifierStateImpl for NoDeathCause{
    fn on_grave_added(self, game: &mut Game, grave: GraveReference) {
        match grave.deref(game).information.clone() {
            GraveInformation::Obscured => {},
            GraveInformation::Normal { role, will, death_notes, .. } => {
                grave.deref_mut(game).information = GraveInformation::Normal{
                    role,
                    will,
                    death_cause: GraveDeathCause::None,
                    death_notes
                }
            },
        }
    }
}
