use serde::{Deserialize, Serialize};

use crate::game::{
    components::graves::grave::{GraveDeathCause, GraveInformation},
    event::on_grave_added::OnGraveAdded, role_list::RoleSet, Game
};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct RoleSetGraveKillers;

impl From<&RoleSetGraveKillers> for ModifierID{
    fn from(_: &RoleSetGraveKillers) -> Self {
        ModifierID::RoleSetGraveKillers
    }
}
impl ModifierStateImpl for RoleSetGraveKillers{
    fn on_grave_added(self, game: &mut Game, event: &OnGraveAdded, _fold: &mut (), _priority: ()) {
        let grave = event.grave;
        match grave.deref(game).information.clone() {
            GraveInformation::Obscured => {},
            GraveInformation::Normal { role, alibi: will, calling_cards, death_causes } => {
                let death_causes = death_causes
                    .into_iter()
                    .map(|killer|{
                        let GraveDeathCause::Role(role) = killer else {return killer};
                        let Some(role_set) = [
                            RoleSet::Town,
                            RoleSet::Mafia,
                            RoleSet::Cult,
                            RoleSet::Fiends,
                            RoleSet::Minions,
                            RoleSet::Neutral,
                        ].iter().find(|set| set.get_roles().contains(&role)).cloned() else {return killer};

                        GraveDeathCause::RoleSet(role_set)
                    })
                    .collect();

                grave.deref_mut(game).information = GraveInformation::Normal{
                    role,
                    alibi: will,
                    death_causes,
                    calling_cards
                }
            },
        }
    }
}
