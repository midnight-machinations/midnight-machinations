use serde::Serialize;

use crate::{game::{
    controllers::*, attack_power::DefensePower, components::{graves::grave::Grave, synopsis::Synopsis, tags::Tag, win_condition::WinCondition}, phase::PhaseState, player::PlayerReference, role::{
        auditor::AuditorResult, engineer::TrapState, kira::KiraResult, krampus::KrampusAbility,
        santa_claus::SantaListKind, Role
    }, role_outline_reference::OutlineIndex, verdict::Verdict
}, vec_set::VecSet};


#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum MessageSender {
    Player{player: PlayerReference},
    Jailor,
    Reporter,
    LivingToDead{player: PlayerReference},
}

// Determines message color
#[derive(PartialOrd, Ord, Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ChatMessageVariant {
    LobbyMessage {
        sender: String,
        text: String
    },

    #[serde(rename_all = "camelCase")]
    Normal{
        message_sender: MessageSender, 
        text: String,
        block: bool,
    },

    #[serde(rename_all = "camelCase")]
    Whisper{
        from_player_index: PlayerReference, 
        to_player_index: PlayerReference, 
        text: String
    },

    BroadcastWhisper {
        whisperer: PlayerReference, 
        whisperee: PlayerReference 
    },

    RoleAssignment{role: Role},
    PlayerDied{grave: Grave},
    PlayersRoleRevealed{player: PlayerReference, role: Role},
    PlayersRoleConcealed{player: PlayerReference},
    TagAdded{player: PlayerReference, tag: Tag},
    TagRemoved{player: PlayerReference, tag: Tag},
    
    #[serde(rename_all = "camelCase")]
    GameOver { synopsis: Synopsis },
    #[serde(rename_all = "camelCase")]
    PlayerQuit{player_index: PlayerReference, game_over: bool},


    
    #[serde(rename_all = "camelCase")]
    PhaseChange{
        phase: PhaseState, 
        day_number: u8
    },
    /* Trial */
    #[serde(rename_all = "camelCase")]
    TrialInformation{
        required_votes: u8, 
        trials_left: u8
    },

    #[serde(rename_all = "camelCase")]
    Voted {
        voter: PlayerReference, 
        votee: Option<PlayerReference> 
    },
    #[serde(rename_all = "camelCase")]
    PlayerNominated{
        player_index: PlayerReference,
        players_voted: Vec<PlayerReference>
    },
    #[serde(rename_all = "camelCase")]
    JudgementVerdict{
        voter_player_index: PlayerReference, 
        verdict: Verdict
    },
    #[serde(rename_all = "camelCase")]
    TrialVerdict {
        player_on_trial: PlayerReference, 
        innocent: u8, 
        guilty: u8 
    },
    #[serde(rename_all = "camelCase")]
    WitnessesCalled {
        player_on_trial: PlayerReference,
        witnesses: Vec<PlayerReference> 
    },
    
    /* Misc */
    #[serde(rename_all = "camelCase")]
    AbilityUsed{
        player: PlayerReference,
        ability_id: ControllerID,
        selection: ControllerSelection
    },

    #[serde(rename_all = "camelCase")]
    PhaseFastForwarded,

    /* Role-specific */
    #[serde(rename_all = "camelCase")]
    PlayerEnfranchised{player_index: PlayerReference},
    InvalidWhisper,
    #[serde(rename_all = "camelCase")]
    PoliticianCountdownStarted,
    #[serde(rename_all = "camelCase")]
    ReporterReport{report: String},
    #[serde(rename_all = "camelCase")]
    PlayerIsBeingInterviewed{player_index: PlayerReference},
    #[serde(rename_all = "camelCase")]
    JailedTarget{player_index: PlayerReference},
    #[serde(rename_all = "camelCase")]
    JailedSomeone{player_index: PlayerReference},
    MediumHauntStarted{medium: PlayerReference, player: PlayerReference},
    MediumSeance{medium: PlayerReference, player: PlayerReference},
    MediumExists,
    #[serde(rename_all = "camelCase")]
    DeputyKilled{shot: PlayerReference},
    #[serde(rename_all = "camelCase")]
    DeputyShotYou,
    #[serde(rename_all = "camelCase")]
    WardenPlayersImprisoned{players: Vec<PlayerReference>},
    WerewolfTracked,

    PuppeteerPlayerIsNowMarionette{player: PlayerReference},
    RecruiterPlayerIsNowRecruit{player: PlayerReference},

    YourConvertFailed,

    NextSantaAbility { ability: SantaListKind },
    AddedToNiceList,
    NextKrampusAbility { ability: KrampusAbility },
    AddedToNaughtyList,
    SantaAddedPlayerToNaughtyList { player: PlayerReference },

    SomeoneSurvivedYourAttack,
    YouSurvivedAttack,
    YouGuardedSomeone,
    YouWereGuarded,
    YouDied,
    YouWereAttacked,
    YouAttackedSomeone,

    YouArePoisoned,

    /*
    Night Information
    */
    RoleBlocked,
    Wardblocked,

    DetectiveResult {suspicious: bool},
    LookoutResult{players: Vec<PlayerReference>},
    TrackerResult{players: Vec<PlayerReference>},
    SeerResult{enemies: bool},
    SpyMafiaVisit{players: Vec<PlayerReference>},
    SpyBug{roles: Vec<Role>},
    PsychicGood{player: PlayerReference},
    PsychicEvil{first: PlayerReference, second: PlayerReference},
    PsychicFailed,
    #[serde(rename_all = "camelCase")]
    AuditorResult{outline_index: OutlineIndex, result: AuditorResult},
    SnoopResult{townie: bool},
    PolymathSnoopResult{inno: bool},
    GossipResult{enemies: bool},
    #[serde(rename_all = "camelCase")]
    TallyClerkResult{evil_count: u8},

    EngineerVisitorsRole{role: Role},
    TrapState{state: TrapState},
    TrapStateEndOfNight{state: TrapState},
    
    #[serde(rename_all = "camelCase")]
    FragileVestBreak{player_with_vest: PlayerReference, defense: DefensePower},

    Transported,

    Silenced,
    Brained,
    #[serde(rename_all = "camelCase")]
    GodfatherBackup{backup: Option<PlayerReference>},
    #[serde(rename_all = "camelCase")]
    GodfatherBackupKilled{backup: PlayerReference},
    

    #[serde(rename_all = "camelCase")]
    PlayerRoleAndAlibi { player: PlayerReference, role: Role, will: String },
    #[serde(rename_all = "camelCase")]
    InformantResult{player: PlayerReference, role: Role, visited_by: Vec<PlayerReference>, visited: Vec<PlayerReference>},
    #[serde(rename_all = "camelCase")]
    AmbusherCaught{ambusher: PlayerReference},

    TargetIsPossessionImmune,
    YouWerePossessed { immune: bool },
    TargetsMessage{message: Box<ChatMessageVariant>},
    PlayerForwardedMessage{forwarder: PlayerReference, message: Box<ChatMessageVariant>},
    TargetHasRole { role: Role },
    #[serde(rename_all = "camelCase")]
    TargetHasWinCondition { win_condition: WinCondition },

    #[serde(rename_all = "camelCase")]
    WerewolfTrackingResult{tracked_player: PlayerReference, players: Vec<PlayerReference>},

    JesterWon,
    RevolutionaryWon,
    ChronokaiserSpeedUp{percent: u32},
    DoomsayerWon,
    DoomsayerFailed,
    MercenaryYouAreAHit,
    MercenaryResult{hit: bool},
    MercenaryHits{roles: VecSet<Role>},
    PawnRole{role: Role},
    KiraResult{result: KiraResult},
    MartyrRevealed { martyr: PlayerReference },
    MartyrWon,
    MartyrFailed,
    WildcardConvertFailed{ role: Role },
}