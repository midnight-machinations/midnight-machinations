use crate::event_priority;
use crate::game::prelude::*;
use super::EventData;

///runs before all players' night actions
#[must_use = "Event must be invoked"]
pub struct OnMidnight;
event_priority!(OnMidnightPriority{
    InitializeNight,

    TopPriority,
    PreWard,

    Transporter,
    Warper,

    Possess,
    Ward,
    Roleblock,

    Deception,  //set aura //set attack
    Heal,   //set protection

    Bodyguard,  //set protection //use attack 
    
    Kill,   //use attack //use protection
    Convert,    //role swap & win condition change //use protection
    Poison, //set poison
    Investigative,  //use aura

    DeleteMessages, //set messages

    StealMessages,  //use messages + set messages (specficially set stolen messages)

    FinalizeNight
});

impl OnMidnight{
    pub fn new(game: &Game) -> (Self, OnMidnightFold){(Self{}, OnMidnightFold::new(game))}
}
impl EventData for OnMidnight {
    type FoldValue = OnMidnightFold;
    type Priority = OnMidnightPriority;

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            Detained::on_midnight,
            Poison::on_midnight,
            PuppeteerMarionette::on_midnight,
            MafiaRecruits::on_midnight,
            ModifierSettings::on_midnight,
            Mafia::on_midnight,
            PlayerReference::on_midnight,
            Abilities::on_midnight,
            FragileVestsComponent::on_midnight,
            Guard::on_midnight,
        ]
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct OnMidnightFold {
    player_data: Vec<PlayerMidnightVariables>,
    visits: Vec<Visit>,
}

impl OnMidnightFold {
    pub fn new(game: &Game) -> Self {
        Self {
            player_data: PlayerReference::all_players(game)
                .map(|player_ref| PlayerMidnightVariables::new(game, player_ref))
                .collect(),
            visits: Vec::new()
        }
    }

    pub fn get(&self, player_ref: PlayerReference) -> &PlayerMidnightVariables {
        unsafe {
            self.player_data.get_unchecked(player_ref.index() as usize)
        }
    }

    pub fn get_mut(&mut self, player_ref: PlayerReference) -> &mut PlayerMidnightVariables {
        unsafe {
            self.player_data.get_unchecked_mut(player_ref.index() as usize)
        }
    }

    pub fn visits(&self)->&Vec<Visit>{
        &self.visits
    }
    pub fn visits_mut(&mut self)->&mut Vec<Visit>{
        &mut self.visits
    }
}

#[derive(Clone, Debug)]
pub struct PlayerMidnightVariables {
    pub died: bool,
    pub attacked: bool,
    pub blocked: bool,
    pub upgraded_defense: Option<DefensePower>,

    pub convert_role_to: Option<RoleState>,

    pub appeared_visits: bool,
    pub framed: bool,

    pub messages: Vec<ChatMessageVariant>,

    pub grave_role: Option<Role>,
    pub grave_killers: Vec<GraveKiller>,
    pub grave_will: String,
    pub grave_death_notes: Vec<String>,

    pub guarded_players: Vec<PlayerReference>
}

impl PartialEq for PlayerMidnightVariables {
    fn eq(&self, other: &Self) -> bool {
        self.died == other.died && 
        self.attacked == other.attacked && 
        self.blocked == other.blocked && 
        self.upgraded_defense == other.upgraded_defense && 
        self.framed == other.framed && 
        self.messages == other.messages && 
        self.grave_role == other.grave_role && 
        self.grave_killers == other.grave_killers && 
        self.grave_will == other.grave_will && 
        self.grave_death_notes == other.grave_death_notes
    }
}

impl Eq for PlayerMidnightVariables {}

impl PlayerMidnightVariables {
    pub fn new(game: &Game, player_ref: PlayerReference) -> Self {
        Self {
            died: false,
            attacked: false,
            blocked: false,
            upgraded_defense: None,

            convert_role_to: None,

            appeared_visits: false,
            framed: false,
            messages: Vec::new(),

            grave_role: None,
            grave_killers: Vec::new(),
            grave_will: player_ref.alibi(game).to_owned(),
            grave_death_notes: Vec::new(),
            guarded_players: Vec::new(),
        }
    }
}

impl PlayerReference {
    pub fn night_died(self, midnight_variables: &OnMidnightFold) -> bool {
        midnight_variables.get(self).died
    }
    pub fn set_night_died(self, midnight_variables: &mut OnMidnightFold, died: bool){
        midnight_variables.get_mut(self).died = died
    }

    pub fn night_attacked(self, midnight_variables: &OnMidnightFold, ) -> bool {
        midnight_variables.get(self).attacked
    }
    pub fn set_night_attacked(self, midnight_variables: &mut OnMidnightFold, attacked: bool){
        midnight_variables.get_mut(self).attacked = attacked;
    }

    pub fn night_blocked(self, midnight_variables: &OnMidnightFold, ) -> bool {
        midnight_variables.get(self).blocked
    }
    pub fn set_night_blocked(self, midnight_variables: &mut OnMidnightFold, roleblocked: bool){
        midnight_variables.get_mut(self).blocked = roleblocked;
    }

    pub fn night_defense(self, game: &Game, midnight_variables: &OnMidnightFold) -> DefensePower {
        midnight_variables.get(self).upgraded_defense.unwrap_or(self.normal_defense(game))
    }
    pub fn set_night_upgraded_defense(self, midnight_variables: &mut OnMidnightFold, defense: Option<DefensePower>){
        midnight_variables.get_mut(self).upgraded_defense = defense;
    }

    pub fn night_framed(self, midnight_variables: &OnMidnightFold, ) -> bool {
        midnight_variables.get(self).framed
    }
    pub fn set_night_framed(self, midnight_variables: &mut OnMidnightFold, framed: bool){
        midnight_variables.get_mut(self).framed = framed;
    }

    pub fn night_convert_role_to(self, midnight_variables: &OnMidnightFold) -> &Option<RoleState> {
        &midnight_variables.get(self).convert_role_to
    }
    pub fn set_night_convert_role_to(self, midnight_variables: &mut OnMidnightFold, convert_role_to: Option<RoleState>){
        midnight_variables.get_mut(self).convert_role_to = convert_role_to;
    }

    pub fn night_appeared_visits(self, midnight_variables: &OnMidnightFold) -> bool{
        midnight_variables.get(self).appeared_visits
    }
    pub fn set_night_appeared_visits(self, midnight_variables: &mut OnMidnightFold, appeared_visits: bool){
        midnight_variables.get_mut(self).appeared_visits = appeared_visits;
    }
    
    pub fn night_messages(self, midnight_variables: &OnMidnightFold) -> &Vec<ChatMessageVariant> {
        &midnight_variables.get(self).messages
    }
    pub fn push_night_message(self, midnight_variables: &mut OnMidnightFold, message: ChatMessageVariant){
        midnight_variables.get_mut(self).messages.push(message);
    }
    pub fn set_night_messages(self, midnight_variables: &mut OnMidnightFold, messages: Vec<ChatMessageVariant>){
        midnight_variables.get_mut(self).messages = messages;
    }

    pub fn night_grave_role(self, midnight_variables: &OnMidnightFold) -> &Option<Role> {
        &midnight_variables.get(self).grave_role
    }
    pub fn set_night_grave_role(self, midnight_variables: &mut OnMidnightFold, grave_role: Option<Role>){
        midnight_variables.get_mut(self).grave_role = grave_role;
    }

    pub fn night_grave_killers(self, midnight_variables: &OnMidnightFold) -> &Vec<GraveKiller> {
        &midnight_variables.get(self).grave_killers
    }
    pub fn push_night_grave_killers(self, midnight_variables: &mut OnMidnightFold, grave_killer: GraveKiller){
        midnight_variables.get_mut(self).grave_killers.push(grave_killer);
    }
    pub fn set_night_grave_killers(self, midnight_variables: &mut OnMidnightFold, grave_killers: Vec<GraveKiller>){
        midnight_variables.get_mut(self).grave_killers = grave_killers;
    }

    pub fn night_grave_will(self, midnight_variables: &OnMidnightFold) -> &String {
        &midnight_variables.get(self).grave_will
    }
    pub fn set_night_grave_will(self, midnight_variables: &mut OnMidnightFold, grave_will: String){
        midnight_variables.get_mut(self).grave_will = grave_will;
    }

    pub fn night_grave_death_notes(self, midnight_variables: &OnMidnightFold) -> &Vec<String> {
        &midnight_variables.get(self).grave_death_notes
    }
    pub fn push_night_grave_death_notes(self, midnight_variables: &mut OnMidnightFold, death_note: String){
        midnight_variables.get_mut(self).grave_death_notes.push(death_note);
    }
    pub fn set_night_grave_death_notes(self, midnight_variables: &mut OnMidnightFold, grave_death_notes: Vec<String>){
        midnight_variables.get_mut(self).grave_death_notes = grave_death_notes;
    }
}