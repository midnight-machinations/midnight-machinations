use super::{player::PlayerReference, role::Role};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Visit {
    pub visitor: PlayerReference,
    pub target: PlayerReference,

    pub tag: VisitTag,
    
    pub attack: bool,
    pub wardblock_immune: bool,
    pub transport_immune: bool,
    pub investigate_immune: bool,
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
            wardblock_immune: false,
            transport_immune: false,
            investigate_immune: false,
            indirect: false,
        }
    }
    pub fn new_none(visitor: PlayerReference, target: PlayerReference)->Self{
        Self {
            visitor, target,
            tag: VisitTag::None,
            attack: false,
            wardblock_immune: false,
            transport_immune: false,
            investigate_immune: false,
            indirect: false
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitTag{
    None,
    Role{role: Role, id: u8},
    SyndicateGunItem,
    SyndicateBackupAttack
}