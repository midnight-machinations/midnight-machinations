use crate::game::{components::graves::{grave::{GraveDeathCause, GraveInformation, GraveKiller}, grave_reference::GraveReference}, role_list::RoleSet, Game};

use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct RoleSetGraveKillers;

impl From<&RoleSetGraveKillers> for ModifierType{
    fn from(_: &RoleSetGraveKillers) -> Self {
        ModifierType::RoleSetGraveKillers
    }
}
impl ModifierTrait for RoleSetGraveKillers{
    fn on_grave_added(self, game: &mut Game, grave: GraveReference) {
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
                                ].iter().find(|set| set.get_roles().contains(&killer_role));
    
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
