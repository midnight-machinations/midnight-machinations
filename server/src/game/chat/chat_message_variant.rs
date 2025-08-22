use serde::{Deserialize, Serialize};

use crate::{game::{
    controllers::*, attack_power::DefensePower, components::{graves::grave::Grave, synopsis::Synopsis, tags::Tag, win_condition::WinCondition}, phase::PhaseState, player::{PlayerIndex, PlayerReference}, role::{
            auditor::AuditorResult, engineer::TrapState, kira::KiraResult, krampus::KrampusAbility,
            santa_claus::SantaListKind, spy::SpyBug, Role
        }, role_list::RoleOutline, role_outline_reference::OutlineIndex, verdict::Verdict
}, vec_set::VecSet};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum MessageSender {
    Player{player: PlayerIndex},
    Jailor,
    Reporter,
    LivingToDead{player: PlayerIndex},
}

// Determines message color
#[derive(PartialOrd, Ord, Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
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
        from_player_index: PlayerIndex, 
        to_player_index: PlayerIndex, 
        text: String
    },

    BroadcastWhisper {
        whisperer: PlayerIndex, 
        whisperee: PlayerIndex 
    },

    RoleAssignment{role: Role},
    PlayerDied{grave: Grave},
    PlayersRoleRevealed{player: PlayerIndex, role: Role},
    PlayersRoleConcealed{player: PlayerIndex},
    TagAdded{player: PlayerReference, tag: Tag},
    TagRemoved{player: PlayerReference, tag: Tag},
    
    #[serde(rename_all = "camelCase")]
    GameOver { synopsis: Synopsis },
    #[serde(rename_all = "camelCase")]
    PlayerQuit{player_index: PlayerIndex, game_over: bool},


    
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
        voter: PlayerIndex, 
        votee: Option<PlayerIndex> 
    },
    #[serde(rename_all = "camelCase")]
    PlayerNominated{
        player_index: PlayerIndex,
        players_voted: Vec<PlayerIndex>
    },
    #[serde(rename_all = "camelCase")]
    JudgementVerdict{
        voter_player_index: PlayerIndex, 
        verdict: Verdict
    },
    #[serde(rename_all = "camelCase")]
    TrialVerdict {
        player_on_trial: PlayerIndex, 
        innocent: u8, 
        guilty: u8 
    },
    
    /* Misc */
    #[serde(rename_all = "camelCase")]
    AbilityUsed{
        player: PlayerIndex,
        ability_id: ControllerID,
        selection: ControllerSelection
    },

    #[serde(rename_all = "camelCase")]
    PhaseFastForwarded,

    /* Role-specific */
    #[serde(rename_all = "camelCase")]
    MayorRevealed{player_index: PlayerIndex},
    InvalidWhisper,
    #[serde(rename_all = "camelCase")]
    PoliticianCountdownStarted,
    #[serde(rename_all = "camelCase")]
    ReporterReport{report: String},
    #[serde(rename_all = "camelCase")]
    PlayerIsBeingInterviewed{player_index: PlayerIndex},
    #[serde(rename_all = "camelCase")]
    JailedTarget{player_index: PlayerIndex},
    #[serde(rename_all = "camelCase")]
    JailedSomeone{player_index: PlayerIndex},
    MediumHauntStarted{medium: PlayerIndex, player: PlayerIndex},
    MediumExists,
    #[serde(rename_all = "camelCase")]
    DeputyKilled{shot_index: PlayerIndex},
    #[serde(rename_all = "camelCase")]
    DeputyShotYou,
    #[serde(rename_all = "camelCase")]
    WardenPlayersImprisoned{players: Vec<PlayerReference>},
    WerewolfTracked,

    PuppeteerPlayerIsNowMarionette{player: PlayerIndex},
    RecruiterPlayerIsNowRecruit{player: PlayerIndex},

    YourConvertFailed,
    CultConvertsNext,
    CultKillsNext,

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

    SheriffResult {suspicious: bool},
    LookoutResult{players: Vec<PlayerIndex>},
    TrackerResult{players: Vec<PlayerIndex>},
    SeerResult{enemies: bool},
    SpyMafiaVisit{players: Vec<PlayerIndex>},
    SpyBug{bug: SpyBug},
    PsychicGood{player: PlayerReference},
    PsychicEvil{first: PlayerReference, second: PlayerReference},
    PsychicFailed,
    #[serde(rename_all = "camelCase")]
    AuditorResult{outline_index: OutlineIndex, role_outline: RoleOutline, result: AuditorResult},
    SnoopResult{townie: bool},
    PolymathSnoopResult{inno: bool},
    GossipResult{enemies: bool},
    #[serde(rename_all = "camelCase")]
    TallyClerkResult{evil_count: u8},
    PercolatorResult{sieve: VecSet<PlayerReference>},
    // This has to send u8s because f64 doesn't implement Ord
    #[serde(rename_all = "camelCase")]
    PercolatorProbabilities {
        // 0-255
        enemy_filter_probability: u8,
        friend_filter_probability: u8
    },

    EngineerVisitorsRole{role: Role},
    TrapState{state: TrapState},
    TrapStateEndOfNight{state: TrapState},
    
    #[serde(rename_all = "camelCase")]
    FragileVestBreak{player_with_vest: PlayerReference, defense: DefensePower},

    Transported,

    Silenced,
    Brained,
    #[serde(rename_all = "camelCase")]
    GodfatherBackup{backup: Option<PlayerIndex>},
    #[serde(rename_all = "camelCase")]
    GodfatherBackupKilled{backup: PlayerIndex},
    

    #[serde(rename_all = "camelCase")]
    PlayerRoleAndAlibi { player: PlayerReference, role: Role, will: String },
    #[serde(rename_all = "camelCase")]
    InformantResult{player: PlayerReference, role: Role, visited_by: Vec<PlayerIndex>, visited: Vec<PlayerIndex>},
    #[serde(rename_all = "camelCase")]
    ScarecrowResult{players: Vec<PlayerIndex>},
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
    WerewolfTrackingResult{tracked_player: PlayerIndex, players: Vec<PlayerIndex>},

    JesterWon,
    RevolutionaryWon,
    ChronokaiserSpeedUp{percent: u32},
    DoomsayerWon,
    DoomsayerFailed,
    MercenaryYouAreAHit,
    MercenaryResult{hit: bool},
    MercenaryHits{roles: VecSet<Role>},
    KiraResult{result: KiraResult},
    MartyrRevealed { martyr: PlayerIndex },
    MartyrWon,
    MartyrFailed,
    WildcardConvertFailed{ role: Role },
}