use crate::game::{components::graves::{grave::{GraveDeathCause, GraveInformation}, grave_reference::GraveReference}, Game};

use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct NoDeathCause;

impl From<&NoDeathCause> for ModifierType{
    fn from(_: &NoDeathCause) -> Self {
        ModifierType::NoDeathCause
    }
}
impl ModifierTrait for NoDeathCause{
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
