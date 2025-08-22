use serde::{Deserialize, Serialize};
use vec1::{
    vec1,
    Vec1
};

use crate::{game::player::PlayerIndex, vec_set::{vec_set, VecSet}};

use super::{components::{insider_group::InsiderGroupID}, game_conclusion::GameConclusion, role::Role};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleList(pub Vec<RoleOutline>);
impl RoleList {
    pub fn simplify(&mut self){
        for entry in self.0.iter_mut(){
            entry.simplify();
        }
    }
    pub fn sort(&mut self){
        self.0.sort_by_key(|r| r.get_all_roles().len());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoleOutline {
    pub options: Vec1<RoleOutlineOption>
}
impl Serialize for RoleOutline {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.options.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for RoleOutline {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        Ok(Self {
            options: Vec1::<RoleOutlineOption>::deserialize(deserializer)?
        })
    }
}

impl Default for RoleOutline {
    fn default() -> Self {
        Self {options: vec1![RoleOutlineOption{
            win_condition: Default::default(),
            insider_groups: Default::default(),
            roles: RoleOutlineOptionRoles::RoleSet { role_set: RoleSet::Any },
            player_pool: Default::default(),
        }]}
    }
}
impl RoleOutline{
    pub fn new_exact(role: Role)->RoleOutline{
        RoleOutline{options: vec1![RoleOutlineOption{
            win_condition: Default::default(),
            insider_groups: Default::default(),
            roles: RoleOutlineOptionRoles::Role{role},
            player_pool: Default::default(),
        }]}
    }
    pub fn get_all_roles(&self) -> Vec<Role>{
        self.options.iter()
            .flat_map(|outline_opt|outline_opt.roles.get_roles().into_iter())
            .collect()
    }
    pub fn simplify(&mut self){
        let mut new_options = self.options.to_vec();

        new_options = new_options.into_iter().collect::<VecSet<_>>().into_iter().collect();

        for option_a in self.options.iter(){
            for option_b in self.options.iter(){
                if option_a.roles.is_subset(&option_b.roles) && option_a != option_b {
                    new_options.retain(|r| r != option_a);
                }
            }
        }

        let mut new_options = Vec1::try_from_vec(new_options)
            .expect("It is impossible to have two sets that are not equal but are subsets of each other, role_list.rs: RoleOutline::simplify");

        new_options.sort();

        *self = RoleOutline{options: new_options};
    }
}


#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, PartialOrd, Ord)]
#[serde(untagged, rename_all = "camelCase")]
pub enum RoleOutlineOptionWinCondition {
    #[default] RoleDefault,
    #[serde(rename_all = "camelCase")]
    GameConclusionReached { win_if_any: VecSet<GameConclusion> },
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, PartialOrd, Ord)]
#[serde(untagged, rename_all = "camelCase")]
pub enum RoleOutlineOptionInsiderGroups {
    #[default] RoleDefault,
    #[serde(rename_all = "camelCase")]
    Custom { insider_groups: VecSet<InsiderGroupID> },
}

impl RoleOutlineOptionWinCondition {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::RoleDefault)
    }
}

impl RoleOutlineOptionInsiderGroups {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::RoleDefault)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct RoleOutlineOption {
    #[serde(flatten)]
    pub roles: RoleOutlineOptionRoles,
    #[serde(flatten, skip_serializing_if = "RoleOutlineOptionWinCondition::is_default")]
    pub win_condition: RoleOutlineOptionWinCondition,
    #[serde(flatten, skip_serializing_if = "RoleOutlineOptionInsiderGroups::is_default")]
    pub insider_groups: RoleOutlineOptionInsiderGroups,
    #[serde(skip_serializing_if = "VecSet::is_empty")]
    pub player_pool: VecSet<PlayerIndex>
}

/// Watch this!
impl<'de> Deserialize<'de> for RoleOutlineOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {

        let mut option = RoleOutlineOption::default();
        
        let json = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Object(map) = json {
            if let Some(value) = map.get("winIfAny") && let Ok(string_win_condition) = serde_json::to_string(value) && let Ok(win_if_any) = serde_json::from_str(string_win_condition.as_str()) {
                option.win_condition = RoleOutlineOptionWinCondition::GameConclusionReached { win_if_any}
            }
            
            if let Some(value) = map.get("insiderGroups") && let Ok(string_insider_groups) = serde_json::to_string(value) && let Ok(insider_groups) = serde_json::from_str(string_insider_groups.as_str()) {
                option.insider_groups = RoleOutlineOptionInsiderGroups::Custom { insider_groups }
            }
            
            if let Some(value) = map.get("playerPool") && let Ok(string_player_pool) = serde_json::to_string(value) && let Ok(player_pool) = serde_json::from_str(string_player_pool.as_str()) {
                option.player_pool = player_pool;
            }
            if let Some(value) = map.get("roleSet") {
                if let Ok(string_role_set) = serde_json::to_string(value) && let Ok(role_set) = serde_json::from_str(string_role_set.as_str()) {
                    option.roles = RoleOutlineOptionRoles::RoleSet { role_set }
                }
            } else if let Some(value) = map.get("role") && let Ok(string_role) = serde_json::to_string(value) && let Ok(role) = serde_json::from_str(string_role.as_str()) {
                option.roles = RoleOutlineOptionRoles::Role { role }
            }
        }

        Ok(option)
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum RoleOutlineOptionRoles {
    #[serde(rename_all = "camelCase")]
    RoleSet{role_set: RoleSet},
    #[serde(rename_all = "camelCase")]
    Role{role: Role},
}
impl Default for RoleOutlineOptionRoles {
    fn default() -> Self {
        Self::RoleSet { role_set: RoleSet::Any }
    }
}
impl RoleOutlineOptionRoles{
    pub fn get_roles(&self) -> VecSet<Role> {
        match self {
            RoleOutlineOptionRoles::RoleSet { role_set } => {
                role_set.get_roles()
            }
            RoleOutlineOptionRoles::Role { role } => 
                vec_set![*role]
        }
    }
    pub fn is_subset(&self, other: &RoleOutlineOptionRoles) -> bool {
        self.get_roles().iter().all(|r|other.get_roles().contains(r))
    }
}
impl PartialOrd for RoleOutlineOptionRoles {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RoleOutlineOptionRoles {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.get_roles().count().cmp(&self.get_roles().count())
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum RoleSet {
    Any,

    Town,
    TownCommon,
    TownInvestigative,
    TownProtective,
    TownKilling,
    TownSupport,

    Mafia,
    MafiaSupport,
    MafiaKilling,

    Cult,
    Fiends,
    
    Neutral,
    Minions
}
impl RoleSet{
    pub fn get_roles(&self) -> VecSet<Role> {
        match self {
            RoleSet::Any => Role::values(),
            RoleSet::Town => 
                vec![
                    Role::Jailor, Role::Villager, Role::Drunk
                ].into_iter().chain(
                    RoleSet::TownCommon.get_roles()
                ).collect(),
            RoleSet::TownCommon => {
                RoleSet::TownInvestigative.get_roles().into_iter()
                .chain(
                    RoleSet::TownProtective.get_roles()
                ).chain(
                    RoleSet::TownKilling.get_roles()
                ).chain(
                    RoleSet::TownSupport.get_roles()
                ).collect()
            },
            RoleSet::TownInvestigative => 
                vec_set![
                    Role::Detective, Role::Philosopher, Role::Gossip, 
                    Role::Psychic, Role::Auditor, Role::Spy, 
                    Role::Lookout, Role::Tracker, Role::Snoop,
                    Role::TallyClerk, Role::Percolator
                ],
            RoleSet::TownProtective => 
                vec_set![
                    Role::Bodyguard, Role::Cop, Role::Doctor,
                    Role::Bouncer, Role::Engineer, Role::Armorsmith,
                    Role::Steward
                ],
            RoleSet::TownKilling => 
                vec_set![
                    Role::Vigilante, Role::Veteran, Role::Deputy, Role::Marksman, Role::Rabblerouser
                ],
            RoleSet::TownSupport => 
                vec_set![
                    Role::Medium, Role::Coxswain, Role::Retributionist,
                    Role::Transporter, Role::Porter,
                    Role::Mayor, Role::Reporter,
                    Role::Courtesan, Role::Escort,
                    Role::Polymath
                ],
            RoleSet::Mafia =>
                vec_set![
                    Role::Goon, Role::MafiaSupportWildcard, Role::MafiaKillingWildcard
                ].into_iter().chain(
                    RoleSet::MafiaKilling.get_roles()
                ).chain(
                    RoleSet::MafiaSupport.get_roles()
                ).collect(),
            RoleSet::MafiaKilling => 
                vec_set![
                    Role::Godfather, Role::Counterfeiter,
                    Role::Impostor, Role::Recruiter,
                    Role::Mafioso
                ],
            RoleSet::MafiaSupport => 
                vec_set![
                    Role::Blackmailer, Role::Cerenovous, Role::Informant, Role::Hypnotist, Role::Consort,
                    Role::Forger, Role::Framer, Role::Mortician, Role::Disguiser,
                    Role::MafiaWitch, Role::Necromancer, Role::Reeducator,
                    Role::Ambusher
                ],
            RoleSet::Minions => 
                vec_set![
                    Role::Witch, Role::Scarecrow, Role::Warper, Role::Kidnapper
                ],
            RoleSet::Neutral =>
                vec_set![
                    Role::Jester, Role::Revolutionary, Role::Politician, Role::Doomsayer, Role::Mercenary,
                    Role::Martyr, Role::Chronokaiser, Role::SantaClaus, Role::Krampus
                ],
            RoleSet::Fiends =>
                vec_set![
                    Role::Arsonist, Role::Werewolf, Role::Ojo,
                    Role::Puppeteer, Role::Pyrolisk, Role::Kira,
                    Role::SerialKiller, Role::FiendsWildcard,
                    Role::Spiral, Role::Warden, Role::Yer
                ],
            RoleSet::Cult =>
                vec_set![
                    Role::Apostle, Role::Disciple, Role::Zealot
                ],
        }
    }
}



pub fn role_enabled_and_not_taken(role: Role, enabled_roles: &VecSet<Role>, taken_roles: &[Role]) -> bool {
    if !enabled_roles.contains(&role) {
        return false;
    }

    match role.maximum_count() {
        Some(max) => taken_roles.iter().filter(|r|**r==role).count() < max.into(),
        None => true,
    }
}