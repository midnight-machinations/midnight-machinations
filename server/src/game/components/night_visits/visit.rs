use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::prelude::PlayerReference;

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
    pub fn new_appeared(visitor: PlayerReference, target: PlayerReference)->Self{
        Self {
            visitor,
            target,
            tag: VisitTag::Appeared,
            attack: false,
            wardblock_immune: true,
            transport_immune: false,
            investigate_immune: true,
            indirect: true
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum VisitTag{
    Ability{ability: AbilityID, id: u8},
    // Role{role: Role, id: u8},
    SyndicateGun,
    SyndicateBackupAttack,
    Appeared
}