use serde::{Deserialize, Serialize};

use crate::game::{
    components::graves::grave::{GraveDeathCause, GraveInformation, GraveKiller},
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
            GraveInformation::Normal { role, will, death_notes, death_cause } => {
                if let GraveDeathCause::Killers(killers) = death_cause {
                    let mut new_killers = Vec::new();

                    for killer in killers {
                        new_killers.push(
                            if let GraveKiller::Role(killer_role) = killer {
                                let killer_role_set = [
                                    RoleSet::Town,
                                    RoleSet::Mafia,
                                    RoleSet::Cult,
                                    RoleSet::Fiends,
                                    RoleSet::Minions,
                                    RoleSet::Neutral,
                                ].iter().find(|set| set.get_roles_static().contains(&killer_role));
    
                                if let Some(role_set) = killer_role_set {
                                    GraveKiller::RoleSet(role_set.clone())
                                } else {
                                    killer
                                }
                            } else {
                                killer
                            }
                        );
                    }

                    grave.deref_mut(game).information = GraveInformation::Normal{
                        role,
                        will,
                        death_cause: GraveDeathCause::Killers(new_killers),
                        death_notes
                    }
                }
            },
        }
    }
}
