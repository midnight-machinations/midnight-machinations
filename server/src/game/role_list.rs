use serde::{Deserialize, Serialize};
use vec1::{
    vec1,
    Vec1
};

use crate::{game::{player::PlayerIndex, role_list_generation::template::Template, settings::Settings}, vec_set::{vec_set, VecSet}};

use super::{components::{insider_group::InsiderGroupID}, game_conclusion::GameConclusion, role::Role};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineList(pub Vec<Outline>);
impl OutlineList {
    pub fn simplify(&mut self){
        for entry in self.0.iter_mut(){
            entry.simplify();
        }
    }
    pub fn sort(&mut self){
        self.0.sort_by_key(|r| r.get_all_templates().len());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Outline {
    pub options: Vec1<OutlineOption>
}
impl Serialize for Outline {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.options.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Outline {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        Ok(Self {
            options: Vec1::<OutlineOption>::deserialize(deserializer)?
        })
    }
}

impl Default for Outline {
    fn default() -> Self {
        Self {options: vec1![OutlineOption{
            win_condition: Default::default(),
            insider_groups: Default::default(),
            templates: OutlineOptionTemplates::TemplateSet { set: TemplateSet::Any },
            player_pool: Default::default(),
        }]}
    }
}
impl Outline{
    pub fn new_exact(template: Template)->Outline{
        Outline{options: vec1![OutlineOption{
            win_condition: Default::default(),
            insider_groups: Default::default(),
            templates: OutlineOptionTemplates::Template{template},
            player_pool: Default::default(),
        }]}
    }
    pub fn get_all_templates(&self) -> Vec<Template>{
        self.options.iter()
            .flat_map(|outline_opt|outline_opt.templates.values().into_iter())
            .collect()
    }
    pub fn simplify(&mut self){
        let mut new_options = self.options.to_vec();

        new_options = new_options.into_iter().collect::<VecSet<_>>().into_iter().collect();

        for option_a in self.options.iter(){
            for option_b in self.options.iter(){
                if option_a.templates.is_subset(&option_b.templates) && option_a != option_b {
                    new_options.retain(|r| r != option_a);
                }
            }
        }

        let mut new_options = Vec1::try_from_vec(new_options)
            .expect("It is impossible to have two sets that are not equal but are subsets of each other, role_list.rs: RoleOutline::simplify");

        new_options.sort();

        *self = Outline{options: new_options};
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
pub struct OutlineOption {
    #[serde(flatten)]
    pub templates: OutlineOptionTemplates,
    #[serde(flatten, skip_serializing_if = "RoleOutlineOptionWinCondition::is_default")]
    pub win_condition: RoleOutlineOptionWinCondition,
    #[serde(flatten, skip_serializing_if = "RoleOutlineOptionInsiderGroups::is_default")]
    pub insider_groups: RoleOutlineOptionInsiderGroups,
    #[serde(skip_serializing_if = "VecSet::is_empty")]
    pub player_pool: VecSet<PlayerIndex>
}

/// Watch this!
impl<'de> Deserialize<'de> for OutlineOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {

        let mut option = OutlineOption::default();
        
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
                    option.templates = OutlineOptionTemplates::TemplateSet { set: role_set }
                }
            } else if let Some(value) = map.get("role") && let Ok(string_role) = serde_json::to_string(value) && let Ok(role) = serde_json::from_str(string_role.as_str()) {
                option.templates = OutlineOptionTemplates::Template { template: role }
            }
        }

        Ok(option)
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum OutlineOptionTemplates {
    #[serde(rename_all = "camelCase")]
    TemplateSet{set: TemplateSet},
    #[serde(rename_all = "camelCase")]
    Template{template: Template},
}
impl Default for OutlineOptionTemplates {
    fn default() -> Self {
        Self::TemplateSet { set: TemplateSet::Any }
    }
}
impl OutlineOptionTemplates{
    pub fn values(&self) -> VecSet<Template> {
        match self {
            OutlineOptionTemplates::TemplateSet { set } => {
                set.values()
            }
            OutlineOptionTemplates::Template { template } => 
                vec_set![*template]
        }
    }
    pub fn is_subset(&self, other: &OutlineOptionTemplates) -> bool {
        self.values().iter().all(|r|other.values().contains(r))
    }
}
impl PartialOrd for OutlineOptionTemplates {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OutlineOptionTemplates {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.values().count().cmp(&self.values().count())
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum TemplateSet {
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
impl TemplateSet{
    pub fn values(&self) -> VecSet<Template> {
        match self {
            TemplateSet::Any => Template::values(),
            TemplateSet::Town => 
                vec![
                    Role::Jailor.into(), Role::Villager.into(), Template::Drunk
                ].into_iter().chain(
                    TemplateSet::TownCommon.values()
                ).collect(),
            TemplateSet::TownCommon => {
                TemplateSet::TownInvestigative.values().into_iter()
                .chain(
                    TemplateSet::TownProtective.values()
                ).chain(
                    TemplateSet::TownKilling.values()
                ).chain(
                    TemplateSet::TownSupport.values()
                ).collect()
            },
            TemplateSet::TownInvestigative => 
                vec_set![
                    Role::Detective, Role::Philosopher, Role::Gossip, 
                    Role::Psychic, Role::Auditor, Role::Spy, 
                    Role::Lookout, Role::Tracker, Role::Snoop,
                    Role::TallyClerk
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::TownProtective => 
                vec_set![
                    Role::Bodyguard, Role::Cop, Role::Doctor,
                    Role::Bouncer, Role::Engineer, Role::Armorsmith,
                    Role::Steward
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::TownKilling => 
                vec_set![
                    Role::Vigilante, Role::Veteran, Role::Deputy, Role::Marksman, Role::Rabblerouser
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::TownSupport => 
                vec_set![
                    Role::Medium, Role::Retributionist,
                    Role::Transporter, Role::Porter,
                    Role::Mayor, Role::Reporter,
                    Role::Courtesan, Role::Escort,
                    Role::Polymath
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::Mafia =>
                vec_set![
                    Role::Goon.into(), Role::MafiaSupportWildcard.into(), Role::MafiaKillingWildcard.into()
                ].into_iter().chain(
                    TemplateSet::MafiaKilling.values()
                ).chain(
                    TemplateSet::MafiaSupport.values()
                ).collect(),
            TemplateSet::MafiaKilling => 
                vec_set![
                    Role::Godfather, Role::Counterfeiter,
                    Role::Impostor, Role::Recruiter,
                    Role::Mafioso
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::MafiaSupport => 
                vec_set![
                    Role::Blackmailer, Role::Cerenovous, Role::Informant, Role::Hypnotist, Role::Consort,
                    Role::Forger, Role::Framer, Role::Mortician, Role::Disguiser,
                    Role::Necromancer, Role::Reeducator,
                    Role::Ambusher
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::Minions => 
                vec_set![
                    Role::Witch.into(), Role::Scarecrow.into(), Role::Warper.into(), Role::Kidnapper.into(), Template::Pawn
                ],
            TemplateSet::Neutral =>
                vec_set![
                    Role::Jester, Role::Revolutionary, Role::Politician, Role::Doomsayer, Role::Mercenary,
                    Role::Martyr, Role::Chronokaiser, Role::SantaClaus, Role::Krampus
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::Fiends =>
                vec_set![
                    Role::Arsonist, Role::Werewolf, Role::Ojo,
                    Role::Puppeteer, Role::Pyrolisk, Role::Kira,
                    Role::SerialKiller, Role::FiendsWildcard,
                    Role::Spiral, Role::Warden, Role::Yer
                ].into_iter().map(|r|r.into()).collect(),
            TemplateSet::Cult =>
                vec_set![
                    Role::Apostle, Role::Disciple, Role::Zealot
                ].into_iter().map(|r|r.into()).collect(),
        }
    }
    
    pub fn contains(&self, template: impl Into<Template>)->bool{
        self.values().contains(&template.into())
    }
}



pub fn role_enabled_and_not_taken(role: Role, settings: &Settings, taken_roles: &[Role]) -> bool {
    if !settings.enabled_templates.enabled(&role) {
        return false;
    }

    match role.maximum_count(settings) {
        Some(max) => taken_roles.iter().filter(|r|**r==role).count() < max.into(),
        None => true,
    }
}