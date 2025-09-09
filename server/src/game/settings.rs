use std::time::Duration;

use serde::{Serialize, Deserialize};

use crate::{game::{modifiers::ModifierSettings, role::Role, role_list::TemplateSet, role_list_generation::template::Template, Game}, vec_set::VecSet};

use super::{phase::PhaseType, role_list::OutlineList};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings{
    pub role_list: OutlineList,
    pub phase_times: PhaseTimeSettings,
    pub enabled_templates: EnabledTemplates,
    pub modifiers: ModifierSettings,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnabledTemplates(VecSet<Template>);
impl EnabledTemplates{
    pub fn new(enabled_templates: VecSet<Template>)->Self{
        Self(enabled_templates)
    }
    pub fn from_game(game: &Game)->&Self{
        &game.settings.enabled_templates
    }
    pub fn get_roles(&self)->VecSet<Role>{
        self.0.iter().filter_map(|t|t.get_role()).collect()
    }
    pub fn enabled(&self, template: impl Into<Template>)->bool{
        self.0.contains(&template.into())
    }
    pub fn intersect_set(&self, other: TemplateSet)->VecSet<Template>{
        self.intersection(other.values())
    }
    pub fn intersection(&self, other: VecSet<Template>)->VecSet<Template>{
        self.0.intersection(&other)
    }
}
impl IntoIterator for EnabledTemplates{
    type Item = Template;

    type IntoIter = std::iter::Map<std::vec::IntoIter<(Template, ())>, fn((Template, ())) -> Template>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseTimeSettings{
    pub briefing: u16,
    pub obituary: u16,
    pub discussion: u16,
    pub nomination: u16,
    pub testimony: u16,
    pub judgement: u16,
    pub final_words: u16,
    pub adjournment: u16,
    pub dusk: u16,
    pub night: u16,
}
impl PhaseTimeSettings {
    pub fn get_time_for(&self, phase: PhaseType) -> Option<Duration> {
        match phase {
            PhaseType::Briefing => Some(Duration::from_secs(self.briefing as u64)),
            PhaseType::Discussion => Some(Duration::from_secs(self.discussion as u64)),
            PhaseType::FinalWords => Some(Duration::from_secs(self.final_words as u64)),
            PhaseType::Dusk => Some(Duration::from_secs(self.dusk as u64)),
            PhaseType::Judgement => Some(Duration::from_secs(self.judgement as u64)),
            PhaseType::Obituary => Some(Duration::from_secs(self.obituary as u64)),
            PhaseType::Night => Some(Duration::from_secs(self.night as u64)),
            PhaseType::Testimony => Some(Duration::from_secs(self.testimony as u64)),
            PhaseType::Nomination => Some(Duration::from_secs(self.nomination as u64)),
            PhaseType::Adjournment => Some(Duration::from_secs(self.adjournment as u64)),
            PhaseType::Recess => None,
        }
    }
    pub fn game_ends_instantly(&self)->bool{
        [self.obituary, self.discussion, self.nomination, self.night, self.dusk].iter().all(|t| *t == 0)
    }
}
impl Default for PhaseTimeSettings{
    fn default() -> Self {
        Self{
            briefing:45,
            obituary:20,
            discussion:100,
            nomination:100,
            adjournment:60,
            testimony:30,
            judgement:30,
            final_words:10,
            dusk:30,
            night:60,
        }
    }
}