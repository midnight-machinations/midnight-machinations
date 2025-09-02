use super::{player::PlayerReference, role::Role};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Visit {
    pub visitor: PlayerReference,
    pub target: PlayerReference,

    pub tag: VisitTag,
    
    pub attack: bool,
    pub wardblockable: bool,
    pub transportable: bool,

    /// Things that cant touch indirect visits (Everything can see its OWN astral visit but maybe not others astral visits)
    /// Engineer Trap triggering
    /// Werewolf, Cop, Ambusher, Veteran, Marksman, Rampage 
    /// Snoop getting CT due to being visited
    pub indirect: bool
}
impl Visit {
    pub fn new_role(visitor: PlayerReference, target: PlayerReference, attack: bool, role: Role, id: u8) -> Self {
        Self {
            visitor,
            target,
            tag: VisitTag::Role{role, id},
            attack,
            wardblockable: true,
            transportable: true,
            indirect: false,
        }
    }
    pub fn new(
        visitor: PlayerReference,
        target: PlayerReference,
        tag: VisitTag,
        attack: bool,
        wardblockable: bool,
        transportable: bool,
        indirect: bool,
    ) -> Self {
        Self {
            visitor,
            target,
            tag,
            attack,
            wardblockable,
            transportable,
            indirect
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitTag{
    Role{role: Role, id: u8},
    SyndicateGunItem,
    SyndicateBackupAttack
}