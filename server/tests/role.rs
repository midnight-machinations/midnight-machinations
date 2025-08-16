mod kit;
use std::{ops::Deref, vec};


pub(crate) use kit::{assert_contains, assert_not_contains};

use mafia_server::{game::{attack_power::DefensePower, components::{graves::{grave::{Grave, GraveDeathCause, GraveInformation, GraveKiller, GravePhase}, grave_reference::GraveReference}, syndicate_gun_item::SyndicateGunItem}, role_list::{RoleList, RoleOutline, RoleOutlineOption, RoleOutlineOptionInsiderGroups, RoleOutlineOptionRoles, RoleOutlineOptionWinCondition}, settings::{PhaseTimeSettings, Settings}, test::mock_game}, vec_set};
pub use mafia_server::game::{
    ability_input::{ControllerID, IntegerSelection, PlayerListSelection, RoleListSelection},
    game_conclusion::GameConclusion,
    role::engineer::Trap,
    chat::{ChatMessageVariant, MessageSender, ChatGroup},
    ability_input::{
        selection_type::{
            two_role_option_selection::TwoRoleOptionSelection,
            two_role_outline_option_selection::TwoRoleOutlineOptionSelection
        },
        AbilityInput,
    }, 
    components::{cult::CultAbility, insider_group::InsiderGroupID},  
    role_list::RoleSet, 
    role_outline_reference::RoleOutlineReference,
     
    player::PlayerReference,
    verdict::Verdict,
    role::{
        Role,
        RoleState,

        jailor::Jailor,
        villager::Villager,
        
        detective::Detective,
        snoop::Snoop,
        lookout::Lookout,
        spy::{Spy, SpyBug},
        tracker::Tracker,
        philosopher::Philosopher,
        psychic::Psychic,
        gossip::Gossip,
        auditor::Auditor,
        
        doctor::Doctor,
        bodyguard::Bodyguard,
        cop::Cop,
        bouncer::Bouncer,
        engineer::Engineer,

        vigilante::Vigilante,
        veteran::Veteran,
        deputy::Deputy,
        marksman::Marksman,
        rabblerouser::Rabblerouser,
        
        transporter::Transporter,
        escort::Escort,
        mayor::Mayor,
        medium::Medium,
        retributionist::Retributionist,
        polymath::Polymath,

        godfather::Godfather,
        impostor::Impostor,
        recruiter::Recruiter,
        counterfeiter::Counterfeiter,
        mafioso::Mafioso,
        
        framer::Framer,
        hypnotist::Hypnotist,
        blackmailer::Blackmailer,
        informant::Informant,
        mafia_witch::MafiaWitch,
        necromancer::Necromancer,
        mortician::Mortician,
        mafia_support_wildcard::MafiaSupportWildcard, 
        ambusher::Ambusher,

        jester::Jester,
        revolutionary::Revolutionary,
        witch::Witch,
        politician::Politician,
        doomsayer::{Doomsayer, DoomsayerGuess},
        wild_card::Wildcard,
        martyr::Martyr,
        santa_claus::SantaClaus,
        krampus::Krampus,

        apostle::Apostle,
        zealot::Zealot,
        
        warden::Warden,
        arsonist::Arsonist,
        werewolf::Werewolf,
        spiral::Spiral,
        pyrolisk::Pyrolisk,
        puppeteer::Puppeteer,
        yer::Yer,
        fiends_wildcard::FiendsWildcard,

        armorsmith::Armorsmith, auditor::AuditorResult,
        drunk::Drunk, ojo::Ojo,
        scarecrow::Scarecrow, tally_clerk::TallyClerk,
        warper::Warper
    }, 
    phase::{
        PhaseState, 
        PhaseType::{self, *}
    }
};
// Pub use so that submodules don't have to reimport everything.
pub use mafia_server::packet::ToServerPacket;


#[test]
fn detective_basic() {
    kit::scenario!(game in Night 1 where
        sher: Detective,
        mafia: Mafioso,
        townie: Detective
    );
    sher.send_ability_input_player_list_typical(mafia);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );

    game.skip_to(Night, 2);
    sher.send_ability_input_player_list_typical(townie);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(2),
        ChatMessageVariant::SheriffResult { suspicious: false }
    );
}

#[test]
fn detective_neutrals(){
    kit::scenario!(game in Night 1 where
        sher: Detective,
        _mafia: Godfather,
        witch: Witch,
        jester: Jester,
        politician: Politician
    );

    sher.send_ability_input_player_list_typical(witch);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );
    
    game.skip_to(Night, 2);
    sher.send_ability_input_player_list_typical(jester);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(2),
        ChatMessageVariant::SheriffResult { suspicious: false }
    );
    
    game.skip_to(Night, 3);
    sher.send_ability_input_player_list_typical(politician);
    game.next_phase();
    assert_contains!(
        sher.get_messages_after_night(3),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );

}

#[test]
fn mortician_obscures_on_stand(){
    kit::scenario!(game in Night 1 where
        mortician: Mortician,
        townie: Detective,
        jail: Jailor,
        gf: Godfather
    );

    mortician.send_ability_input_player_list_typical(townie);
    game.skip_to(Nomination, 2);
    
    jail.vote_for_player(Some(townie));
    gf.vote_for_player(Some(townie));
    mortician.vote_for_player(Some(townie));

    game.skip_to(Judgement, 2);
    jail.set_verdict(Verdict::Guilty);
    gf.set_verdict(Verdict::Guilty);

    game.skip_to(Night, 2);
    
    assert_eq!(unsafe{GraveReference::new_unchecked(0)}.deref(&game).information, GraveInformation::Obscured);
    assert_contains!(mortician.get_messages(), ChatMessageVariant::PlayerRoleAndAlibi { player: townie.player_ref(), role: Role::Detective, will: "".to_string() });
}

#[test]
fn mortician_obscures_fail_after_death(){
    kit::scenario!(game in Night 1 where
        mortician: Mortician,
        townie: Detective,
        jail: Jailor,
        gf: Godfather
    );

    mortician.send_ability_input_player_list_typical(townie);
    game.skip_to(Nomination, 2);
    
    jail.vote_for_player(Some(mortician));
    gf.vote_for_player(Some(mortician));
    townie.vote_for_player(Some(mortician));

    game.skip_to(Judgement, 2);
    jail.set_verdict(Verdict::Guilty);
    gf.set_verdict(Verdict::Guilty);

    game.skip_to(Night, 2);
    gf.send_ability_input_player_list_typical(townie);
    game.next_phase();
    assert!(matches!(unsafe{GraveReference::new_unchecked(1)}.deref(&game).information, GraveInformation::Normal { role: Role::Detective, .. }));
    assert_not_contains!(mortician.get_messages(), ChatMessageVariant::PlayerRoleAndAlibi { player: townie.player_ref(), role: Role::Detective, will: "".to_string() });
}

#[test]
fn detective_godfather() {
    kit::scenario!(game in Night 1 where
        sher: Detective,
        mafia: Godfather
    );
    sher.send_ability_input_player_list_typical(mafia);
    game.next_phase();
    assert_contains!(sher.get_messages(), ChatMessageVariant::SheriffResult { suspicious: false });
}

#[test]
fn philosopher_basic() {
    kit::scenario!(game in Night 1 where
        philosopher: Philosopher,
        mafia1: Mafioso,
        mafia2: Informant,
        townie1: Detective,
        townie2: Vigilante
    );

    philosopher.send_ability_input_two_player_typical(mafia1, townie1);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(1),
        ChatMessageVariant::SeerResult { enemies: true }
    );

    game.skip_to(Night, 2);
    philosopher.send_ability_input_two_player_typical(mafia1, mafia2);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(2),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 3);
    philosopher.send_ability_input_two_player_typical(townie2, townie1);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(3),
        ChatMessageVariant::SeerResult { enemies: false }
    );
}

#[test]
fn philosopher_neutrals() {
    kit::scenario!(game in Night 1 where
        philosopher: Philosopher,
        mafia1: Mafioso,
        townie1: Vigilante,
        jester: Jester,
        witch: Witch
    );

    game.skip_to(Night, 3);
    philosopher.send_ability_input_two_player_typical(jester, mafia1);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(3),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 4);
    philosopher.send_ability_input_two_player_typical(townie1, jester);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(4),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 6);
    philosopher.send_ability_input_two_player_typical(townie1, witch);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(6),
        ChatMessageVariant::SeerResult { enemies: true }
    );

    game.skip_to(Night, 7);
    philosopher.send_ability_input_two_player_typical(jester, witch);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(7),
        ChatMessageVariant::SeerResult { enemies: false }
    );

    game.skip_to(Night, 8);
    philosopher.send_ability_input_two_player_typical(mafia1, witch);
    
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_night(8),
        ChatMessageVariant::SeerResult { enemies: false }
    );
}

#[test]
fn jester_basic() {
    kit::scenario!(game in Night 1 where
        jester: Jester,
        townie: Detective,
        mafia: Godfather,
        lookout1: Lookout,
        lookout2: Lookout,
        mafia2: Mafioso
    );

    game.skip_to(Nomination, 2);
    mafia.vote_for_player(Some(jester));
    townie.vote_for_player(Some(jester));
    lookout1.vote_for_player(Some(jester));
    lookout2.vote_for_player(Some(jester));


    game.skip_to(Judgement, 2);
    townie.set_verdict(mafia_server::game::verdict::Verdict::Guilty);
    mafia.set_verdict(mafia_server::game::verdict::Verdict::Guilty);
    mafia2.set_verdict(mafia_server::game::verdict::Verdict::Guilty);
    lookout1.set_verdict(mafia_server::game::verdict::Verdict::Innocent);
    lookout2.set_verdict(mafia_server::game::verdict::Verdict::Innocent);

    game.skip_to(Night, 2);
    assert!(!jester.alive());
    lookout1.send_ability_input_player_list_typical(townie);
    lookout2.send_ability_input_player_list_typical(mafia);


    game.skip_to(Obituary, 3);

    assert_eq!(
        PlayerReference::all_players(&game)
            .filter(|p|!p.alive(&game)).count(), 2
    );
    assert_contains!(
        lookout1.get_messages(), 
        ChatMessageVariant::LookoutResult { players: vec![] }
    );
    assert_contains!(
        lookout2.get_messages(), 
        ChatMessageVariant::LookoutResult { players: vec![] }
    );
}

#[test]
fn rabble_rouser_dies(){
    kit::scenario!(game in Night 1 where
        exe: Revolutionary,
        townie: Detective,
        mafioso: Mafioso
    );

    game.skip_to(Nomination, 2);
    mafioso.vote_for_player(Some(townie));
    exe.vote_for_player(Some(townie));

    game.skip_to(Judgement, 2);
    exe.set_verdict(Verdict::Guilty);
    mafioso.set_verdict(Verdict::Guilty);

    game.skip_to(FinalWords, 2);
    assert!(!exe.alive());
    assert!(townie.alive());
    assert!(mafioso.alive());
}

#[test]
fn psychic_auras(){
    for _ in 0..20 {
        kit::scenario!(game in Night 1 where
            psy: Psychic,
            god: Godfather,
            maf: Framer,
            town1: Detective,
            town2: Vigilante
        );

        psy.send_ability_input_player_list_typical(maf);
    
        game.next_phase();
        let messages = psy.get_messages_after_night(1);
        let messages: Vec<_> = 
            messages.into_iter()
            .filter(|msg|match msg {
                ChatMessageVariant::PsychicEvil { first, second } => {
                    ![first.index(), second.index()].contains(&god.index())
                }
                _ => false
            }).collect();

        if messages.len() != 1 {
            panic!("{messages:?}");
        }

        game.skip_to(Night, 2);
        maf.send_ability_input_player_list_typical(town2);
        psy.send_ability_input_player_list_typical(maf);
        game.next_phase();
        let messages = psy.get_messages_after_night(2);
        let messages: Vec<_> = 
            messages.into_iter()
            .filter(|msg|match msg {
                ChatMessageVariant::PsychicGood { player } => {
                    player.index() == town1.index()
                },
                _ => false
            }).collect();

        if messages.len() != 1 {
            panic!("{messages:?}");
        }
    }
}

#[test]
fn tally_clerk_basic(){
    kit::scenario!(game in Nomination 2 where
        fg: TallyClerk,
        townie: Detective,
        mafioso: Mafioso
    );

    fg.vote_for_player(Some(townie));
    mafioso.vote_for_player(Some(townie));

    game.skip_to(Judgement, 2);

    fg.set_verdict(Verdict::Guilty);
    mafioso.set_verdict(Verdict::Guilty);
    
    game.skip_to(Obituary, 3);
    assert_contains!(
        fg.get_messages_after_night(1),
        ChatMessageVariant::TallyClerkResult { evil_count: 1 }
    );
}


#[test]
fn spy_basic_transported() {
    kit::scenario!(game in Night 1 where
        spy: Spy,
        _mafioso: Mafioso,
        blackmailer: Blackmailer,
        esc: Escort,
        transp: Transporter,
        bugged: Detective,
        jester: Jester,
        witch: Witch
    );
    spy.send_ability_input_player_list_typical(jester);
    transp.send_ability_input_two_player_typical(jester, bugged);
    blackmailer.send_ability_input_player_list_typical(jester);
    esc.send_ability_input_player_list_typical(jester);
    witch.send_ability_input_two_player_typical(jester, esc);

    game.next_phase();

    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Silenced });
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Roleblocked });
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Transported });
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyBug { bug: SpyBug::Possessed });

    
    assert_contains!(spy.get_messages(), ChatMessageVariant::SpyMafiaVisit { players: vec![bugged.index()] });
}

#[test]
fn bodyguard_basic() {
    kit::scenario!(game in Night 2 where
        maf: Mafioso,
        bg: Bodyguard,
        townie: Detective
    );

    maf.send_ability_input_player_list_typical(townie);
    bg.send_ability_input_player_list_typical(townie);

    game.skip_to(Obituary, 3);

    // assert!(townie.get_messages().contains(&ChatMessageVariant::YouWereGuarded));

    assert!(townie.alive());
    assert!(!bg.alive());
    assert!(!maf.alive());
}

#[test]
fn doctor_basic() {
    kit::scenario!(game in Night 2 where
        maf: Mafioso,
        doc: Doctor,
        townie: Detective
    );

    maf.send_ability_input_player_list_typical(townie);
    doc.send_ability_input_player_list_typical(townie);

    game.skip_to(Discussion, 3);

    assert!(townie.alive());
    assert!(doc.alive());

    assert_contains!(maf.get_messages(), ChatMessageVariant::SomeoneSurvivedYourAttack);
    assert_contains!(townie.get_messages(), ChatMessageVariant::YouWereGuarded);
    assert_contains!(doc.get_messages(), ChatMessageVariant::YouGuardedSomeone);

    assert!(maf.alive());
}

/// Tests if transporter properly swaps, redirecting actions on their first target to their
/// second. The vigilante will try to kill town1, which should end up killing town2.
#[test]
fn transporter_basic_vigilante_escort() {
    kit::scenario!(game in Night 2 where
        _godfather: Godfather,
        trans: Transporter,
        vigi: Vigilante,
        escort: Escort,
        town1: Detective,
        town2: Detective
    );
    trans.send_ability_input_two_player_typical(town1, town2);
    vigi.send_ability_input_player_list_typical(town1);
    escort.send_ability_input_player_list_typical(town2);

    game.skip_to(Obituary, 3);
    assert!(town1.alive());
    assert!(!town2.alive());

    assert!(town1.received_blocked_message());
    assert!(!town2.received_blocked_message());
    
    game.skip_to(Obituary, 4);
    assert!(!vigi.alive());
}

#[test]
fn transporter_basic_seer_sheriff_framer() {
    kit::scenario!(game in Night 1 where
        trans: Transporter,
        philosopher: Philosopher,
        _mafioso: Mafioso,
        framer: Framer,
        town1: Detective,
        town2: Detective
    );
    assert!(trans.send_ability_input_two_player_typical(town1, town2));
    assert!(framer.send_ability_input_player_list_typical(town1));
    assert!(framer.send_ability_input_player_list(philosopher, 1));
    assert!(philosopher.send_ability_input_two_player_typical(town1, town2));
    assert!(town1.send_ability_input_player_list_typical(town2));
    assert!(town2.send_ability_input_player_list_typical(town1));

    game.skip_to(Obituary, 2);
    assert_contains!(
        philosopher.get_messages_after_night(1),
        ChatMessageVariant::SeerResult { enemies: true }
    );
    assert_contains!(
        town1.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: false }
    );
    assert_contains!(
        town2.get_messages_after_night(1),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );
}

/// Test that the bodyguard protects the person their target was swapped with
#[test]
fn bodyguard_protects_transported_target() {
    kit::scenario!(game in Night 2 where
        trans: Transporter,
        maf: Mafioso,
        bg: Bodyguard,
        t1: Detective,
        t2: Detective
    );
    trans.send_ability_input_two_player_typical(t1, t2);
    maf.send_ability_input_player_list_typical(t1);
    bg.send_ability_input_player_list_typical(t1);
    
    game.skip_to(Obituary, 3);
    assert!(t1.alive());
    assert!(t2.alive());
    assert!(trans.alive());
    assert!(!bg.alive());
    assert!(!maf.alive());

    // assert_not_contains!(t1.get_messages(), ChatMessageVariant::YouWereGuarded);
    // assert_contains!(t2.get_messages(), ChatMessageVariant::YouWereGuarded);
    // assert_contains!(bg.get_messages(), ChatMessageVariant::YouGuardedSomeone);
}

#[test]
fn mayor_reveals_after_they_vote(){
    kit::scenario!(game where
        mayor: Mayor,
        _townie: Detective,
        mafioso: Mafioso
    );

    game.skip_to(Nomination, 2);
    mayor.vote_for_player(Some(mafioso));
    mayor.send_ability_input_unit_typical();
    game.next_phase();
    assert_eq!(game.current_phase().phase(), Testimony);
}


#[test]
fn retributionist_basic(){
    kit::scenario!(game in Night 2 where
        ret: Retributionist,
        sher1: Detective,
        sher2: Detective,
        mafioso: Mafioso
    );

    mafioso.send_ability_input_player_list_typical(sher1);
    game.skip_to(Night, 3);
    mafioso.send_ability_input_player_list_typical(sher2);
    game.skip_to(Night, 4);

    assert!(!sher1.alive());
    assert!(!sher2.alive());

    assert!(ret.send_ability_input_two_player_typical(sher1, mafioso));
    game.next_phase();
    assert_contains!(
        ret.get_messages_after_night(4),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SheriffResult{ suspicious: true }
        )}
    );

    game.skip_to(Night, 5);
    assert!(ret.send_ability_input_two_player_typical(sher1, mafioso));
    game.next_phase();
    assert_contains!(
        ret.get_messages_after_night(5),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SheriffResult{ suspicious: true }
        )}
    );

    game.skip_to(Night, 6);
    ret.send_ability_input_two_player_typical(sher1, mafioso);
    game.next_phase();
    assert_not_contains!(
        ret.get_messages_after_night(6),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SheriffResult{ suspicious: true }
        )}
    );
}

#[test]
fn necromancer_basic(){
    kit::scenario!(game in Night 2 where
        ret: Necromancer,
        sher: Snoop,
        informant: Informant,
        mafioso: Mafioso,
        vigilante: Vigilante
    );
    
    mafioso.send_ability_input_player_list_typical(sher);
    game.skip_to(Night, 3);
    vigilante.send_ability_input_player_list_typical(informant);
    game.skip_to(Night, 4);



    assert!(ret.send_ability_input_two_player_typical(sher, mafioso));
    game.next_phase();
    assert_contains!(
        ret.get_messages_after_night(3),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SnoopResult { townie: false }
        )}
    );
}

#[test]
fn witch_basic(){
    kit::scenario!(game in Night 1 where
        witch: Witch,
        sher: Detective,
        informant: Informant,
        mafioso: Mafioso,
        philosopher: Philosopher
    );

    assert!(witch.send_ability_input_two_player_typical(sher, mafioso));
    game.next_phase();
    assert_contains!(witch.get_messages(), ChatMessageVariant::TargetsMessage{message: Box::new(
        ChatMessageVariant::SheriffResult{ suspicious: true }
    )});
    
    game.skip_to(Night, 2);
    assert!(philosopher.send_ability_input_two_player_typical(sher, informant));
    assert!(witch.send_ability_input_two_player_typical(philosopher, mafioso));
    game.next_phase();
    assert_contains!(
        witch.get_messages_after_night(2),
        ChatMessageVariant::TargetsMessage{message: Box::new(
            ChatMessageVariant::SeerResult { enemies: false }
        )}
    );
}

#[test]
fn cop_basic(){
    kit::scenario!(game in Night 2 where
        crus: Cop,
        protected_player: Jester,
        townie1: Detective,
        townie2: Detective,
        mafioso: Mafioso,
        _maf2: Framer
    );

    crus.send_ability_input_player_list_typical(protected_player);
    townie1.send_ability_input_player_list_typical(protected_player);
    townie2.send_ability_input_player_list_typical(protected_player);
    mafioso.send_ability_input_player_list_typical(protected_player);

    game.skip_to(Night, 3);

    assert!(crus.alive());
    assert!(protected_player.alive());
    assert!(townie1.alive());
    assert!(townie2.alive());
    assert!(!mafioso.alive());

    crus.send_ability_input_player_list_typical(protected_player);
    townie1.send_ability_input_player_list_typical(protected_player);
    townie2.send_ability_input_player_list_typical(protected_player);

    game.next_phase();
    
    assert!(crus.alive());
    assert!(protected_player.alive());
    assert!(townie1.alive() || townie2.alive());
    assert!(!townie1.alive() || !townie2.alive());
}

#[test]
fn cop_does_not_kill_framed_player(){
    kit::scenario!(game in Night 2 where
        crus: Cop,
        protected_player: Jester,
        townie: Detective,
        framer: Framer,
        mafioso: Mafioso
    );

    assert!(crus.send_ability_input_player_list_typical(protected_player));
    assert!(framer.send_ability_input_player_list(townie, 0));
    assert!(framer.send_ability_input_player_list(protected_player, 1));

    game.next_phase();

    assert!(crus.alive());
    assert!(protected_player.alive());
    assert!(framer.alive());
    assert!(mafioso.alive());
    assert!(townie.alive());
}

#[test]
fn ambusher_basic(){
    kit::scenario!(game in Night 2 where
        ambusher: Ambusher,
        protected_player: Jester,
        townie1: Detective,
        townie2: Detective,
        blackmailer: Informant
    );

    
    ambusher.send_ability_input_player_list_typical(protected_player);
    townie1.send_ability_input_player_list_typical(protected_player);
    townie2.send_ability_input_player_list_typical(protected_player);
    blackmailer.send_ability_input_player_list_typical(protected_player);

    game.skip_to(Night, 3);

    assert!(ambusher.alive());
    assert!(protected_player.alive());
    assert!(townie1.alive() || townie2.alive());
    assert!(!townie1.alive() || !townie2.alive());
    assert!(blackmailer.alive());

    let townie1_status = townie1.alive();
    let townie2_status = townie2.alive();

    ambusher.send_ability_input_player_list_typical(protected_player);
    blackmailer.send_ability_input_player_list_typical(protected_player);

    game.next_phase();
    
    assert!(ambusher.alive());
    assert!(protected_player.alive());
    assert!(townie1.alive() || townie2.alive());
    assert!(!townie1.alive() || !townie2.alive());
    assert!(blackmailer.get_messages().contains(&ChatMessageVariant::YouSurvivedAttack)||!blackmailer.alive());
    assert!(townie1.alive() == townie1_status);
    assert!(townie2.alive() == townie2_status);
    
}

#[test]
fn ambusher_does_not_kill_framed_player(){
    kit::scenario!(game in Night 2 where
        ambusher: Ambusher,
        protected_player: Jester,
        townie: Detective,
        framer: Framer,
        mafioso: Mafioso
    );

    assert!(ambusher.send_ability_input_player_list_typical(protected_player));
    assert!(framer.send_ability_input_player_list(townie, 0));
    assert!(framer.send_ability_input_player_list(protected_player, 1));

    game.next_phase();

    assert!(ambusher.alive());
    assert!(protected_player.alive());
    assert!(framer.alive());
    assert!(mafioso.alive());
    assert!(townie.alive());
}

#[test]
fn insider_group_init(){
    kit::scenario!(game in Night 2 where
        syn: Informant,
        townie: Detective
    );
    assert!(InsiderGroupID::Mafia.contains_player(&game, syn.player_ref()));
    assert!(!InsiderGroupID::Mafia.contains_player(&game, townie.player_ref()));
}
#[test]
fn syn_starts_with_gun(){
    kit::scenario!(game in Night 2 where
        syn: Informant,
        _townie: Detective
    );
    assert!(SyndicateGunItem::player_has_gun(&game, syn.player_ref()));
}

#[test]
fn ambusher_attacks_self(){
    kit::scenario!(game in Night 2 where
        ambusher: Ambusher,
        protected_player: Jester,
        townie: Detective
    );

    ambusher.send_ability_input_player_list_typical(protected_player);
    ambusher.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_gun_item_shoot(),
        PlayerListSelection(vec![protected_player.player_ref()])
    ));

    game.next_phase();

    assert_contains!(ambusher.get_messages(), ChatMessageVariant::YouSurvivedAttack);

    assert!(ambusher.alive());
    assert!(!protected_player.alive());
    assert!(townie.alive());
}

#[test]
fn veteran_basic(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        townie: Lookout,
        _godfather: Godfather,
        framer: Framer,
        tracker: Tracker
    );

    assert!(vet.send_ability_input_boolean_typical(true));
    assert!(framer.send_ability_input_player_list_typical(vet));
    assert!(townie.send_ability_input_player_list_typical(vet));
    assert!(tracker.send_ability_input_player_list_typical(vet));

    game.next_phase();

    assert!(vet.alive());
    assert!(!framer.alive());
    assert!(!townie.alive());

    assert!(
        townie.get_messages().contains(&ChatMessageVariant::LookoutResult { players: vec![framer.index(), tracker.index()] }) ||
        townie.get_messages().contains(&ChatMessageVariant::LookoutResult { players: vec![tracker.index(), framer.index()] })
    );

    // assert_contains!(
    //     townie.get_messages(),
    //     ChatMessageVariant::LookoutResult { players: vec![framer.index(), tracker.index()] }
    // );
    assert_contains!(
        tracker.get_messages(),
        ChatMessageVariant::TrackerResult { players: vec![] }
    );
}

#[test]
fn veteran_does_not_kill_framed_player(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        townie: Detective,
        framer: Framer,
        mafioso: Mafioso
    );

    assert!(vet.send_ability_input_boolean_typical(true));
    assert!(framer.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list(vet, 1));

    game.next_phase();

    assert!(vet.alive());
    assert!(framer.alive());
    assert!(mafioso.alive());
    assert!(townie.alive());
}

#[test]
fn rabble_rouser_turns_into_jester(){
    kit::scenario!(game in Night 2 where
        target: Detective,
        mafioso: Mafioso,
        exe: Revolutionary
    );

    assert!(mafioso.send_ability_input_player_list_typical(target));

    game.skip_to(Nomination, 3);

    assert!(!target.alive());
    assert!(exe.alive());
    assert!(mafioso.alive());
    let RoleState::Jester(_) = exe.role_state() else {panic!()};
}
#[test]
fn rabble_rouser_instantly_turns_into_jester(){
    kit::scenario!(_game where
        _godfather: Godfather,
        _serial_killer: SerialKiller,
        exe: Revolutionary
    );
    let RoleState::Jester(_) = exe.role_state() else {panic!()};
}

#[test]
fn can_type_in_jail() {
    kit::scenario!(game in Dusk 1 where
        _godfather: Godfather,
        jailor: Jailor,
        detective: Detective
    );

    jailor.send_ability_input_player_list_typical(detective);
    game.next_phase();

    detective.send_message("Hello!");
    
    assert_contains!(jailor.get_messages(), 
        ChatMessageVariant::Normal { 
            message_sender: MessageSender::Player { player: detective.index() }, 
            text: "Hello!".to_string(),
            block: false
        }
    );
    
    assert_contains!(detective.get_messages(), 
        ChatMessageVariant::Normal { 
            message_sender: MessageSender::Player { player: detective.index() }, 
            text: "Hello!".to_string(),
            block: false
        }
    );
}

#[test]
fn mafioso_cant_kill_mafia() {
    kit::scenario!(game in Night 1 where
        mafioso: Mafioso,
        mortician: Mortician,
        _detective: Detective
    );

    mafioso.send_ability_input_player_list_typical(mortician);

    game.next_phase();

    assert!(mortician.alive());
}

#[test]
fn marksman_basic() {
    kit::scenario!(game in Night 2 where
        mk: Marksman,
        dt: Detective,
        gf: Godfather
    );

    assert!(dt.send_ability_input_player_list_typical(gf));
    mk.send_ability_input(AbilityInput::new(
        ControllerID::role(mk.player_ref(), Role::Marksman, 0),
        PlayerListSelection(vec!(dt.player_ref()))
    ));
    mk.send_ability_input(AbilityInput::new(
        ControllerID::role(mk.player_ref(), Role::Marksman, 1),
        PlayerListSelection(vec!(gf.player_ref()))
    ));

    game.next_phase();

    assert!(!dt.alive());
    assert!(gf.alive());
    assert!(mk.alive());
}

#[test]
fn transporter_cant_transport_dead() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
        _vet: Veteran,
        _necro: Necromancer,
        _seer: Philosopher,
        townie: Detective,
        thomas: Jailor,
        trans: Transporter
    );

    mafioso.send_ability_input_player_list_typical(thomas);

    game.next_phase();

    assert!(!thomas.alive());

    game.skip_to(Night, 3);

    trans.send_ability_input_two_player_typical(townie, thomas);

    game.next_phase();

    assert_not_contains!(thomas.get_messages(), ChatMessageVariant::Transported);
    assert_not_contains!(townie.get_messages(), ChatMessageVariant::Transported);
}

#[test]
fn double_transport() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
 
        townie_a: Detective,
        townie_b: Jailor,

        trans_a: Transporter,
        trans_b: Transporter
    );
    
    assert!(mafioso.send_ability_input_player_list_typical(townie_a));

    assert!(trans_a.send_ability_input_two_player_typical(townie_a, townie_b));
    assert!(trans_b.send_ability_input_two_player_typical(townie_b, townie_a));

    game.next_phase();
    assert!(!townie_a.alive());
    assert!(townie_b.alive());
}


#[test]
fn double_transport_single_player() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
 
        townie_a: Detective,
        townie_b: Jailor,
        townie_c: Vigilante,

        trans_a: Transporter,
        trans_b: Transporter
    );
    
    assert!(mafioso.send_ability_input_player_list_typical(townie_a));

    assert!(trans_a.send_ability_input_two_player_typical(townie_a, townie_b));
    assert!(trans_b.send_ability_input_two_player_typical(townie_a, townie_c));


    game.next_phase();

    assert!(townie_a.alive());
    if trans_a.player_ref().index() < trans_b.player_ref().index(){
        assert!(!townie_b.alive());
        assert!(townie_c.alive());
    }else{
        assert!(townie_b.alive());
        assert!(!townie_c.alive());
    }
}

#[test]
fn double_transport_three_players() {
    for _ in 0..50 {
        kit::scenario!(game in Night 2 where
            mafioso: Mafioso,
     
            townie_a: Detective,
            townie_b: Jailor,
            townie_c: Vigilante,
    
            trans_a: Transporter,
            trans_b: Transporter,
            trans_c: Transporter
        );
        
        assert!(mafioso.send_ability_input_player_list_typical(townie_a));
    
        assert!(trans_a.send_ability_input_two_player_typical(townie_a, townie_b));
        assert!(trans_b.send_ability_input_two_player_typical(townie_a, townie_c));
        assert!(trans_c.send_ability_input_two_player_typical(townie_b, townie_c));
    
    
        game.next_phase();
    
        if [trans_a, trans_b, trans_c].into_iter().is_sorted_by_key(|t|t.player_ref().index()){
            assert!(townie_a.alive());
            assert!(townie_b.alive());
            assert!(!townie_c.alive());
        }else if [trans_c, trans_a, trans_b].into_iter().is_sorted_by_key(|t|t.player_ref().index()) {
            assert!(townie_a.alive());
            assert!(!townie_b.alive());
            assert!(townie_c.alive());
        }else if [trans_b, trans_c, trans_a].into_iter().is_sorted_by_key(|t|t.player_ref().index()) {
            assert!(!townie_a.alive());
            assert!(townie_b.alive());
            assert!(townie_c.alive());
        }
        
        else if [trans_c, trans_b, trans_a].into_iter().is_sorted_by_key(|t|t.player_ref().index()) {
            assert!(townie_a.alive());
            assert!(townie_b.alive());
            assert!(!townie_c.alive());
        }else if [trans_a, trans_c, trans_b].into_iter().is_sorted_by_key(|t|t.player_ref().index()) {
            assert!(!townie_a.alive());
            assert!(townie_b.alive());
            assert!(townie_c.alive());
        }else if [trans_b, trans_a, trans_c].into_iter().is_sorted_by_key(|t|t.player_ref().index()) {
            assert!(townie_a.alive());
            assert!(!townie_b.alive());
            assert!(townie_c.alive());
        }
    }
}


#[test]
fn grave_contains_multiple_killers() {
    kit::scenario!(game in Night 2 where
        mafioso: Mafioso,
        vigilante: Vigilante,
        townie: Detective
    );

    assert!(mafioso.send_ability_input_player_list_typical(townie));
    assert!(vigilante.send_ability_input_player_list_typical(townie));
    game.next_phase();
    assert!(
        *unsafe{GraveReference::new_unchecked(0)}.deref(&game) ==
        Grave{ 
            player: townie.player_ref(),
            died_phase: GravePhase::Night,
            day_number: 2,
            information: GraveInformation::Normal{
                role: Role::Detective,
                death_cause: GraveDeathCause::Killers(vec![GraveKiller::RoleSet(RoleSet::Mafia), GraveKiller::Role(Role::Vigilante)]),
                will: "".to_string(),
                death_notes: vec![],
            }
        } ||
        *unsafe{GraveReference::new_unchecked(0)}.deref(&game) ==
        Grave{ 
            player: townie.player_ref(),
            died_phase: GravePhase::Night,
            day_number: 2,
            information: GraveInformation::Normal{
                role: Role::Detective,
                death_cause: GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Vigilante), GraveKiller::RoleSet(RoleSet::Mafia)]),
                will: "".to_string(),
                death_notes: vec![],
            }
        }
    )
}

#[test]
fn drunk_suspicious_aura() {
    kit::scenario!(game in Night 1 where
        drunk: Drunk,
        detective: Detective,
        _mafioso: Mafioso
    );

    assert!(detective.send_ability_input_player_list_typical(drunk));

    game.next_phase();

    assert_contains!(
        detective.get_messages(),
        ChatMessageVariant::SheriffResult { suspicious: true }
    );
}

#[test]
fn framer_second_visit_erased() {
    kit::scenario!(game in Night 2 where
        framer: Framer,
        lookout: Lookout,
        v: Villager
    );

    framer.send_ability_input_player_list_typical(lookout);
    framer.send_ability_input_player_list(v, 1);
    lookout.send_ability_input_player_list_typical(v);

    assert_not_contains!(
        lookout.get_messages(),
        ChatMessageVariant::LookoutResult { players: vec![framer.index()] }
    )
}

#[test]
fn witch_second_visit_erased() {
    kit::scenario!(game in Night 2 where
        witch: MafiaWitch,
        lookout: Lookout,
        v: Villager
    );

    witch.send_ability_input_two_player_typical(lookout, v);
    lookout.send_ability_input_player_list_typical(v);

    assert_not_contains!(
        lookout.get_messages(),
        ChatMessageVariant::LookoutResult { players: vec![witch.index()] }
    )
}

#[test]
fn drunk_framer() {
    kit::scenario!(game in Night 2 where
        drunk: Drunk,
        lookout: Lookout,
        lookout2: Lookout,
        mafioso: Mafioso,
        townie: Doctor,
        framer: Framer
    );

    assert!(mafioso.send_ability_input_player_list_typical(townie));
    assert!(lookout.send_ability_input_player_list_typical(townie));
    assert!(lookout2.send_ability_input_player_list_typical(mafioso));
    framer.send_ability_input_player_list_typical(drunk);
    framer.send_ability_input_player_list(mafioso, 1);

    game.next_phase();

    let messages2 = lookout2.get_messages();
    if !(
        messages2.contains(&ChatMessageVariant::LookoutResult { players: vec![drunk.index()] })
    ){
        panic!("{messages2:?}");
    }

    let messages = lookout.get_messages();
    if !(
        messages.contains(&ChatMessageVariant::LookoutResult { players: vec![mafioso.index()] })
    ){
        panic!("{messages:?}");
    }
}

#[test]
fn drunk_role_change() {
    kit::scenario!(game in Night 1 where
        drunk: Drunk,
        lo: Lookout,
        apostle: Apostle,
        mafioso: Mafioso
    );

    assert!(apostle.send_ability_input_player_list_typical(drunk));

    game.skip_to(Night, 2);

    assert!(mafioso.send_ability_input_player_list_typical(apostle));
    assert!(lo.send_ability_input_player_list_typical(apostle));

    game.next_phase();

    let messages = lo.get_messages();
    assert!(
        !messages.contains(&ChatMessageVariant::LookoutResult { players: vec![mafioso.index(), drunk.index()] }) &&
        !messages.contains(&ChatMessageVariant::LookoutResult { players: vec![drunk.index(), mafioso.index()] })
    );
    assert!(
        messages.contains(&ChatMessageVariant::LookoutResult { players: vec![mafioso.index()] })
    );
}

#[test]
fn vigilante_cant_select_night_one() {
    kit::scenario!(game in Night 1 where
        townie_b: Doctor,
        _godfather: Godfather,
        vigilante_suicide: Vigilante

    );
    vigilante_suicide.send_ability_input_player_list_typical(townie_b);
    game.next_phase();
    assert!(townie_b.alive());
}

#[test]
fn godfather_backup_kills_esc() {
    kit::scenario!(game in Night 2 where
        godfather: Godfather,
        hypnotist: Hypnotist,
        det: Detective,
        esc: Escort
    );

    godfather.send_ability_input(AbilityInput::new(
        ControllerID::SyndicateChooseBackup,
        PlayerListSelection(vec![hypnotist.player_ref()])
    ));
    hypnotist.send_ability_input(AbilityInput::new(
        ControllerID::SyndicateBackupAttack,
        PlayerListSelection(vec![det.player_ref()])
    ));

    assert!(esc.send_ability_input_player_list_typical(godfather));

    game.next_phase();
    assert!(!det.alive());
    assert!(godfather.alive());
    assert!(hypnotist.alive());
    assert!(esc.alive());
}

#[test]
fn snoop_basic() {
    kit::scenario!(game in Night 1 where
        gf: Godfather,
        det: Detective,
        snoop: Snoop,
        v: Detective
    );

    assert!(snoop.send_ability_input_player_list_typical(det));
    assert!(det.send_ability_input_player_list_typical(snoop));
    game.next_phase();
    assert_contains!(
        snoop.get_messages_after_night(1),
        ChatMessageVariant::SnoopResult { townie: true }
    );

    game.skip_to(Night, 2);

    assert!(snoop.send_ability_input_player_list_typical(gf));
    game.next_phase();
    assert_contains!(
        snoop.get_messages_after_night(2),
        ChatMessageVariant::SnoopResult { townie: false }
    );

    game.skip_to(Night, 3);

    assert!(snoop.send_ability_input_player_list_typical(det));
    assert!(det.send_ability_input_player_list_typical(snoop));
    assert!(v.send_ability_input_player_list_typical(snoop));
    game.next_phase();
    assert_contains!(
        snoop.get_messages_after_night(3),
        ChatMessageVariant::SnoopResult { townie: false }
    );

    game.skip_to(Night, 4);

    assert!(snoop.send_ability_input_player_list_typical(det));
    assert!(gf.send_ability_input_player_list_typical(snoop));
    game.next_phase();
    assert_contains!(
        snoop.get_messages_after_night(4),
        ChatMessageVariant::SnoopResult { townie: false }
    );    
}

#[test]
fn polymath_snoop_basic() {
    kit::scenario!(game in Night 1 where
        gf: Godfather,
        framer: Framer,
        det: Detective,
        polymath: Polymath
    );
    let mut night = 1;
    polymath.send_ability_input_integer_typical(1);
    assert!(polymath.send_ability_input_player_list(det, 1));
    assert!(det.send_ability_input_player_list_typical(polymath));
    game.next_phase();
    assert_contains!(
        polymath.get_messages_after_night(night),
        ChatMessageVariant::PolymathSnoopResult { inno: false }
    );
    night += 1;
    game.skip_to(Night, night);

    polymath.send_ability_input_integer_typical(1);
    assert!(polymath.send_ability_input_player_list(det, 1));
    game.next_phase();
    assert_contains!(
        polymath.get_messages_after_night(night),
        ChatMessageVariant::PolymathSnoopResult { inno: true }
    );

    night += 1;
    game.skip_to(Night, night);

    polymath.send_ability_input_integer_typical(1);
    assert!(polymath.send_ability_input_player_list(gf, 1));
    game.next_phase();
    assert_contains!(
        polymath.get_messages_after_night(night),
        ChatMessageVariant::PolymathSnoopResult { inno: true }
    );

    night += 1;
    game.skip_to(Night, night);

    polymath.send_ability_input_integer_typical(1);
    polymath.send_ability_input_player_list(det, 1);
    det.send_ability_input_player_list_typical(polymath);
    game.next_phase();
    assert_contains!(
        polymath.get_messages_after_night(night),
        ChatMessageVariant::PolymathSnoopResult { inno: false }
    );

    night += 1;
    game.skip_to(Night, night);

    framer.send_ability_input_player_list_typical(det);
    polymath.send_ability_input_integer_typical(1);
    polymath.send_ability_input_player_list(det, 1);
    game.next_phase();
    assert_contains!(
        polymath.get_messages_after_night(night),
        ChatMessageVariant::PolymathSnoopResult { inno: false }
    );

    night += 1;
    game.skip_to(Night, night);

    polymath.send_ability_input_integer_typical(1);
    polymath.send_ability_input_player_list(framer, 1);
    game.next_phase();
    assert_contains!(
        polymath.get_messages_after_night(night),
        ChatMessageVariant::PolymathSnoopResult { inno: false }
    );
}

#[test]
fn polymath_armorsmith_marksman_basic() {
    kit::scenario!(game in Night 2 where
        _godfather: Godfather,
        armor: Polymath,
        mark: Polymath,
        phil: Philosopher
    );
    
    armor.send_ability_input_integer_typical(2);
    armor.send_ability_input_player_list(phil, 2);
    mark.send_ability_input_integer_typical(4);
    mark.send_ability_input_player_list(phil, 4);
    mark.send_ability_input_player_list(armor, 5);
    phil.send_ability_input_two_player_typical(armor, mark);

    game.next_phase();

    assert!(armor.alive());
    assert!(mark.alive());
    assert!(phil.alive());

    assert_contains!(
        mark.get_messages_after_night(1),
        ChatMessageVariant::SomeoneSurvivedYourAttack
    );

    assert_contains!(
        armor.get_messages_after_night(2),
        ChatMessageVariant::YouGuardedSomeone
    );
    
    assert_contains!(
        phil.get_messages_after_night(2),
        ChatMessageVariant::YouWereGuarded
    );
}

#[test]
fn godfather_backup_kills_jail() {
    kit::scenario!(game in Dusk 2 where
        godfather: Godfather,
        hypnotist: Hypnotist,
        det: Detective,
        jail: Jailor
    );

    jail.send_ability_input_player_list_typical(godfather);
    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        PlayerListSelection(vec![hypnotist.player_ref()])
    ));

    game.next_phase();
    hypnotist.send_ability_input_player_list_typical(det);
    hypnotist.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_backup_attack(),
        PlayerListSelection(vec![det.player_ref()])
    ));

    game.next_phase();

    assert!(!det.alive());
    assert!(godfather.alive());
    assert!(hypnotist.alive());
    assert!(jail.alive());
}

#[test]
fn gossip_basic_friends() {
    kit::scenario!(game in Night 1 where
        gossip: Gossip,
        t1: Detective,
        t2: Detective,
        _gf: Godfather
    );

    assert!(gossip.send_ability_input_player_list_typical(t1));
    assert!(t1.send_ability_input_player_list_typical(t2));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: false }
    );
}

#[test]
fn gossip_basic_enemies_inverted() {
    kit::scenario!(game in Night 2 where
        gossip: Gossip,
        t1: Detective,
        _t2: Detective,
        py: Pyrolisk
    );

    assert!(gossip.send_ability_input_player_list_typical(py));
    assert!(py.send_ability_input_player_list_typical(t1));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: false }
    );
}

#[test]
fn gossip_basic_enemies() {
    kit::scenario!(game in Night 2 where
        gossip: Gossip,
        t1: Detective,
        _t2: Detective,
        py: Pyrolisk
    );

    assert!(gossip.send_ability_input_player_list_typical(t1));
    assert!(t1.send_ability_input_player_list_typical(py));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: true }
    );
}

#[test]
fn gossip_framer() {
    kit::scenario!(game in Night 1 where 
        gossip: Gossip,
        framer: Framer,
        t2: Detective,
        townie: Detective,
        _gf: Godfather    
    );

    assert!(gossip.send_ability_input_player_list_typical(townie));
    assert!(townie.send_ability_input_player_list_typical(framer));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: true }
    );

    game.skip_to(Night, 2);

    assert!(gossip.send_ability_input_player_list_typical(townie));
    assert!(townie.send_ability_input_player_list_typical(framer));
    assert!(framer.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list(gossip, 1));

    game.next_phase();

    assert_contains!(
        gossip.get_messages(),
        ChatMessageVariant::GossipResult { enemies: true }
    );

    game.skip_to(Night, 3);

    assert!(gossip.send_ability_input_player_list_typical(t2));
    assert!(t2.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list_typical(townie));
    assert!(framer.send_ability_input_player_list(gossip, 1));

    game.next_phase();

    assert_contains!(
        gossip.get_messages_after_night(3),
        ChatMessageVariant::GossipResult { enemies: true }
    );

    game.skip_to(Night, 4);

    assert!(gossip.send_ability_input_player_list_typical(t2));
    assert!(t2.send_ability_input_player_list_typical(townie));

    game.next_phase();

    assert_contains!(
        gossip.get_messages_after_night(4),
        ChatMessageVariant::GossipResult { enemies: false }
    );
}

#[test]
fn vigilante_one_bullet_with_four_players() {
    kit::scenario!(game in Night 2 where
        vigi: Vigilante,
        t1: Detective,
        t2: Detective,
        gf: Godfather
    );

    assert!(vigi.send_ability_input_player_list_typical(gf));
    game.skip_to(Night, 3);
    assert!(vigi.send_ability_input_player_list_typical(t1));

    game.next_phase();

    assert!(t1.alive());
    assert!(t2.alive());
    assert!(vigi.alive());
    assert!(gf.alive());
}

#[test]
fn seer_cant_see_godfather() {
    kit::scenario!(game in Night 1 where
        philosopher: Philosopher,
        godfather: Godfather,
        mafioso: Mafioso,
        townie: Detective
    );

    assert!(philosopher.send_ability_input_two_player_typical(godfather, mafioso));
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 1 }
        ),
        ChatMessageVariant::SeerResult { enemies: false }
    );
    game.skip_to(Night, 2);

    assert!(philosopher.send_ability_input_two_player_typical(godfather, townie));
    game.next_phase();
    assert_contains!(
        philosopher.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::SeerResult { enemies: false }
    );
}

#[test]
fn bouncer_jailor_double_block() {
    kit::scenario!(game in Dusk 1 where
        b: Bouncer,
        jail: Jailor,
        gf: Godfather,
        det: Detective
    );

    jail.send_ability_input_player_list_typical(gf);

    game.next_phase();

    det.send_ability_input_player_list_typical(gf);
    b.send_ability_input_player_list_typical(gf);

    game.next_phase();

    assert_contains!(
        det.get_messages_after_last_message(ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 1 }),
        ChatMessageVariant::Wardblocked
    );
}

#[test]
fn bouncer_ojo_block() {
    kit::scenario!(game in Night 2 where
        b: Bouncer,
        ojo: Ojo,
        det1: Detective,
        det2: Detective,
        det3: Detective,
        det4: Detective
    );

    ojo.send_ability_input_player_list_typical(det1);
    b.send_ability_input_player_list_typical(det1);

    game.next_phase();

    assert!(det1.alive());
    assert!(det2.alive());
    assert!(det3.alive());
    assert!(det4.alive());

    assert_contains!(
        ojo.get_messages_after_last_message(ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 1 }),
        ChatMessageVariant::Wardblocked
    );
}

#[test]
fn godfather_backup_sets_off_engineer_trap() {
    kit::scenario!(game in Night 2 where
        backup: Informant,
        eng: Engineer,
        gf: Godfather,
        esc: Escort
    );

    gf.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        PlayerListSelection(vec![backup.player_ref()])
    ));
    gf.send_ability_input_player_list_typical(eng);

    backup.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_backup_attack(),
        PlayerListSelection(vec![esc.player_ref()])
    ));
    assert!(esc.send_ability_input_player_list_typical(gf));
    assert!(eng.send_ability_input_player_list_typical(esc));

    game.next_phase();

    assert!(gf.alive());
    assert!(esc.alive());
    assert!(eng.alive());
    assert!(!backup.alive());
    
}

#[test]
fn warden_dismantles_trap() {
    kit::scenario!(game in Dusk 2 where
        evil: Warden,
        eng: Engineer,
        esc: Escort
    );

    evil.send_ability_input_player_list_typical(esc);
    eng.send_ability_input_player_list_typical(esc);

    game.skip_to(Obituary, 3);

    assert!(matches!(
        eng.role_state(),
        RoleState::Engineer(Engineer { trap: Trap::Dismantled })
    ));
    assert!(esc.alive());
    assert!(evil.alive());
    assert!(eng.alive());
}

#[test]
fn godfather_wardblock_still_kills() {
    kit::scenario!(game in Night 2 where
        rev: Bouncer,
        godfather: Godfather,
        jan: Mortician,
        townie_a: Detective,
        townie_b: Detective
    );

    assert!(rev.send_ability_input_player_list_typical(townie_a));
    assert!(godfather.send_ability_input_player_list_typical(townie_a));
    godfather.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_choose_backup(),
        PlayerListSelection(vec![jan.player_ref()])
    ));
    jan.send_ability_input(AbilityInput::new(
        ControllerID::syndicate_backup_attack(),
        PlayerListSelection(vec![townie_b.player_ref()])
    ));

    game.next_phase();
    assert_not_contains!(
        townie_a.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 1}
        ),
        ChatMessageVariant::Wardblocked
    );
    assert_contains!(
        godfather.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 1}
        ),
        ChatMessageVariant::Wardblocked
    );
    assert_not_contains!(
        jan.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange{phase: PhaseState::Night, day_number: 1}
        ),
        ChatMessageVariant::Wardblocked
    );

    assert!(!townie_b.alive());
    assert!(townie_a.alive());
}

#[test]
fn puppeteer_marionettes_philosopher(){
    kit::scenario!(game in Night 2 where
        puppeteer: Puppeteer,
        philo: Philosopher,
        townie: Detective,
        townie2: Detective
    );

    puppeteer.send_ability_input(AbilityInput::new(
        ControllerID::role(puppeteer.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(1)
    ));

    assert!(puppeteer.send_ability_input_player_list_typical(townie));
    assert!(philo.send_ability_input_two_player_typical(townie2, townie));

    game.next_phase();
    assert_contains!(
        philo.get_messages_after_night(1),
        ChatMessageVariant::SeerResult{ enemies: true }
    );

    game.skip_to(Night, 3);

    assert!(philo.send_ability_input_two_player_typical(puppeteer, townie));

    game.next_phase();
    assert_contains!(
        philo.get_messages_after_night(2),
        ChatMessageVariant::SeerResult{ enemies: false }
    );
}

#[test]
fn puppeteer_marionettes_die(){
    kit::scenario!(game in Night 2 where
        puppeteer: Puppeteer,
        townie: Detective,
        townie2: Detective,
        townie3: Detective
    );

    puppeteer.send_ability_input(AbilityInput::new(
        ControllerID::role(puppeteer.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(1)
    ));

    assert!(puppeteer.send_ability_input_player_list_typical(townie));

    game.skip_to(Night, 3);

    puppeteer.send_ability_input(AbilityInput::new(
        ControllerID::role(puppeteer.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(0)
    ));

    assert!(puppeteer.send_ability_input_player_list_typical(townie2));

    game.next_phase();

    assert!(!townie.alive());
    assert!(!townie2.alive());
    assert!(townie3.alive());
    assert!(puppeteer.alive());
}

#[test]
fn puppeteer_marionettes_win(){
    kit::scenario!(game in Night 2 where
        puppeteer: Puppeteer,
        townie: Detective,
        townie2: Detective
    );

    puppeteer.send_ability_input(AbilityInput::new(
        ControllerID::role(puppeteer.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(1)
    ));

    assert!(puppeteer.send_ability_input_player_list_typical(townie));

    game.skip_to(Nomination, 3);

    puppeteer.vote_for_player(Some(townie2));
    townie.vote_for_player(Some(townie2));

    game.skip_to(Judgement, 3);

    puppeteer.set_verdict(Verdict::Guilty);
    townie.set_verdict(Verdict::Guilty);

    game.skip_to(Dusk, 3);

    assert!(puppeteer.alive());
    assert!(townie.alive());
    assert!(!townie2.alive());

    assert!(puppeteer.get_won_game());
    assert!(townie.get_won_game());
    assert!(!townie2.get_won_game());
}

#[test]
fn deputy_shoots_marionette(){
    kit::scenario!(game in Night 2 where
        deputy: Deputy,
        puppeteer: Puppeteer,
        townie: Detective
    );

    puppeteer.send_ability_input(AbilityInput::new(
        ControllerID::role(puppeteer.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(1)
    ));
    assert!(puppeteer.send_ability_input_player_list_typical(townie));

    game.skip_to(Discussion, 3);

    deputy.send_ability_input_player_list_typical(townie);

    assert!(puppeteer.alive());
    assert!(!townie.alive());
    assert!(deputy.alive());
}

#[test]
fn vigilante_shoots_marionette(){
    kit::scenario!(game in Night 2 where
        vigilante: Vigilante,
        puppeteer: Puppeteer,
        townie: Detective
    );

    puppeteer.send_ability_input(AbilityInput::new(
        ControllerID::role(puppeteer.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(1)
    ));
    assert!(puppeteer.send_ability_input_player_list_typical(townie));
    assert!(vigilante.send_ability_input_player_list_typical(townie));

    game.next_phase();

    assert!(puppeteer.alive());
    assert!(!townie.alive());
    assert!(vigilante.alive());
}

#[test]
fn arsonist_ignites_and_aura(){
    kit::scenario!(game in Night 2 where
        arso: Arsonist,
        townie: Detective,
        townie2: Detective,
        gf: Godfather,
        vigi: Vigilante,
        det: Detective
    );

    assert!(townie.send_ability_input_player_list_typical(arso));
    assert!(arso.send_ability_input_player_list_typical(arso));
    assert!(det.send_ability_input_player_list_typical(townie));

    game.next_phase();

    assert!(arso.alive());
    assert!(!townie.alive());
    assert!(det.alive());
    assert!(gf.alive());
    assert!(vigi.alive());

    assert_contains!(
        det.get_messages_after_night(2), 
        ChatMessageVariant::SheriffResult{ suspicious: true }
    );

    game.skip_to(Night, 3);
    
    assert!(arso.send_ability_input_player_list_typical(townie2));
    assert!(det.send_ability_input_player_list_typical(townie2));

    game.next_phase();

    assert_contains!(
        det.get_messages_after_night(3), 
        ChatMessageVariant::SheriffResult{ suspicious: true }
    );

    game.skip_to(Nomination, 4);

    townie2.vote_for_player(Some(arso));
    gf.vote_for_player(Some(arso));
    vigi.vote_for_player(Some(arso));

    game.skip_to(Judgement, 4);

    townie2.set_verdict(Verdict::Guilty);
    gf.set_verdict(Verdict::Guilty);
    vigi.set_verdict(Verdict::Guilty);


    game.skip_to(Night, 4);

    assert!(det.send_ability_input_player_list_typical(townie2));

    game.next_phase();
    
    assert_contains!(
        det.get_messages_after_night(4), 
        ChatMessageVariant::SheriffResult{ suspicious: false }
    );

    
}

#[test]
fn bodyguard_gets_single_target_jailed_message() {
    kit::scenario!(game in Dusk 2 where
        bg: Bodyguard,
        jailor: Jailor,
        _maf: Mafioso,
        townie: Detective
    );

    jailor.send_ability_input_player_list_typical(townie);

    game.next_phase();

    bg.send_ability_input_player_list_typical(townie);

    game.next_phase();

    assert_contains!(
        bg.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { 
                phase: PhaseState::Night, day_number: 2
            }
        ),
        ChatMessageVariant::Wardblocked
        /* They should not get a second Wardblocked message */
    );
}

#[test]
fn martyr_suicide_ends_game() {
    kit::scenario!(game in Night 2 where
        martyr: Martyr,
        player1: Mafioso,
        player2: Detective,
        player3: Mortician,
        player4: Detective
    );

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrRevealed { martyr: martyr.index() }
    );

    martyr.send_ability_input_boolean_typical(true);

    game.next_phase();

    assert!(!martyr.alive());
    assert!(martyr.get_won_game());
    assert!(!player1.alive());
    assert!(!player2.alive());
    assert!(!player3.alive());
    assert!(!player4.alive());

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrWon
    );

    assert!(game.game_is_over());
}

#[test]
fn martyr_roleblocked() {
    kit::scenario!(game in Night 1 where
        martyr: Martyr,
        player1: Mafioso,
        player2: Detective,
        hypnotist: Hypnotist,
        player4: Detective
    );

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrRevealed { martyr: martyr.index() }
    );

    martyr.send_ability_input_player_list_typical(martyr);
    hypnotist.send_ability_input_player_list_typical(martyr);

    game.next_phase();

    assert!(martyr.alive());
    assert!(!martyr.get_won_game());
    assert!(player1.alive());
    assert!(player2.alive());
    assert!(hypnotist.alive());
    assert!(player4.alive());

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrFailed
    );
}

#[test]
fn martyr_healed() {
    kit::scenario!(game in Night 2 where
        martyr: Martyr,
        player1: Mafioso,
        player2: Detective,
        doctor: Doctor,
        player4: Detective
    );

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrRevealed { martyr: martyr.index() }
    );

    martyr.send_ability_input_player_list_typical(martyr);
    doctor.send_ability_input_player_list_typical(martyr);

    game.next_phase();

    assert!(martyr.alive());
    assert!(!martyr.get_won_game());
    assert!(player1.alive());
    assert!(player2.alive());
    assert!(doctor.alive());
    assert!(player4.alive());

    assert_contains!(
        player1.get_messages(),
        ChatMessageVariant::MartyrFailed
    );
}

#[test]
fn deputy_shoots_townie(){
    kit::scenario!(game in Discussion 2 where
        deputy: Deputy,
        player1: Mafioso,
        player2: Detective
    );

    assert!(deputy.send_ability_input_player_list_typical(player2));
    assert!(!deputy.alive());
    assert!(!player2.alive());
    assert!(player1.alive());
    assert!(player1.get_won_game());
    assert!(game.game_is_over());
}

#[test]
fn ojo_transporter(){
    kit::scenario!(game in Night 2 where
        ojo: Ojo,
        transporter: Transporter,
        player1: Philosopher,
        player2: Detective,
        player3: Philosopher,
        gf: Godfather
    );

    ojo.send_ability_input_player_list_typical(player1);

    transporter.send_ability_input_two_player_typical(player1, player2);
    gf.send_ability_input_player_list_typical(ojo);

    game.next_phase(); 

    assert!(player1.alive());
    assert!(!player2.alive());
    assert!(player3.alive());
    assert!(gf.alive());
}

#[test]
fn cult_alternates() {
    for _ in 0..20 {
        kit::scenario!(game in Night 1 where
            apostle: Apostle,
            b: Detective,
            c: Detective,
            d: Detective,
            e: Detective,
            f: Detective,
            g: Detective
        );


        //apostle converts
        assert!(game.cult().next_ability == CultAbility::Convert);
        assert!(apostle.send_ability_input_player_list_typical(b));
        game.next_phase();
        assert!(b.alive());
        assert!(InsiderGroupID::Cult.contains_player(game.deref(), b.player_ref()));
        assert_contains!(game.cult().ordered_cultists, b.player_ref());

        //zealot kills, apostle waits
        game.skip_to(Night, 2);
        assert!(game.cult().next_ability == CultAbility::Kill);
        assert!(game.cult().ordered_cultists.len() == 2);
        assert!(b.send_ability_input_player_list_typical(c));
        game.next_phase();
        assert!(!c.alive());
        assert!(d.alive());
        assert!(!InsiderGroupID::Cult.contains_player(game.deref(), d.player_ref()));
        assert_not_contains!(game.cult().ordered_cultists, d.player_ref());

        //zealot waits, apostle converts
        game.skip_to(Night, 3);
        assert!(game.cult().ordered_cultists.len() == 2);
        assert!(apostle.send_ability_input_player_list_typical(d));
        game.next_phase();
        assert!(e.alive());
        assert!(d.alive());
        assert!(InsiderGroupID::Cult.contains_player(game.deref(), d.player_ref()));
        assert_contains!(game.cult().ordered_cultists, d.player_ref());

        //zealot kills, apostle waits
        game.skip_to(Night, 4);
        assert!(game.cult().ordered_cultists.len() == 3);
        assert!(d.send_ability_input_player_list_typical(g));
        game.next_phase();
        assert!(f.alive());
        assert!(!g.alive());
    }
}

#[test]//writing "cult" here so if you ctrl+f for cult related tests you find this
fn apostle_converting_trapped_player_day_later() {
    for _ in 0..20 {
        kit::scenario!(game in Night 2 where
            apostle: Apostle,
            zealot: Zealot,
            trapped: Detective,
            engineer: Engineer
        );
    
    
        assert!(engineer.send_ability_input_player_list_typical(trapped));
    
        game.skip_to(Night, 3);
    
        //explanation of why they are both targeting here:
        //if zealot.player_index < apostle.player_index then this fails otherwise
        //If the order is backwards, then the get converted in player order
        //this should be fixed
        assert!(apostle.send_ability_input_player_list_typical(trapped));
        assert!(zealot.send_ability_input_player_list_typical(trapped));
    
        game.next_phase();
    
        assert_contains!(engineer.get_messages_after_night(3), ChatMessageVariant::EngineerVisitorsRole { role: Role::Apostle });
        assert_eq!(trapped.role_state().role(), Role::Detective);
    }
}

#[test]//writing "cult" here so if you ctrl+f for cult related tests you find this
fn apostle_converting_trapped_player_same_day(){
    for _ in 0..20 {
        kit::scenario!(game in Night 2 where
            apostle: Apostle,
            _zealot: Zealot,
            trapped: Detective,
            engineer: Engineer
        );


        assert!(engineer.send_ability_input_player_list_typical(trapped));
        assert!(apostle.send_ability_input_player_list_typical(trapped));

        game.next_phase();

        assert!(trapped.role_state().role() != Role::Zealot);
        assert!(trapped.role_state().role() == Role::Detective);
    }
}

#[test]
fn engineer_day_later(){
    kit::scenario!(game in Night 2 where
        maf: Mafioso,
        trapped: Detective,
        engineer: Engineer
    );


    assert!(engineer.send_ability_input_player_list_typical(trapped));

    game.skip_to(Night, 3);

    assert!(maf.send_ability_input_player_list_typical(trapped));

    game.next_phase();

    assert_contains!(engineer.get_messages_after_night(3), ChatMessageVariant::EngineerVisitorsRole { role: Role::Mafioso });
    assert!(!maf.alive());
    assert!(trapped.alive());
}

#[test]
fn yer() {
    kit::scenario!(game in Dusk 1 where
        yer: Yer,
        detective: Detective,
        convertee: Engineer,
        gossip: Gossip,
        _villager: Villager,
        informant: Informant
    );

    /* Night 1 Kill Attempt Test */
    yer.send_ability_input_boolean_typical(true);
    yer.send_ability_input_player_list(detective, 1);
    yer.send_ability_input(AbilityInput::new(
        ControllerID::Role { player: yer.player_ref(), role: Role::Yer, id: 2 }, 
        RoleListSelection(vec!(Role::TallyClerk)) 
    ));

    game.skip_to(PhaseType::Dusk, 2);

    assert!(yer.alive());
    assert!(detective.alive());
    assert!(convertee.alive());
    assert!(gossip.alive());
    assert!(informant.alive());

    assert!(detective.role() == Role::Detective);

    /* Reg Attack Test */
    yer.send_ability_input_boolean_typical(false);
    yer.send_ability_input_player_list(detective, 1);
    yer.send_ability_input(AbilityInput::new(
        ControllerID::Role { player: yer.player_ref(), role: Role::Yer, id: 2 }, 
        RoleListSelection(vec!(Role::Mafioso)) 
    ));
    
    game.skip_to(PhaseType::Dusk, 3);

    assert!(yer.alive());
    assert!(!detective.alive());
    assert!(convertee.alive());
    assert!(gossip.alive());
    assert!(informant.alive());

    assert!(detective.role() == Role::Detective);
    assert!(convertee.role() == Role::Engineer);

    let grave_message = ChatMessageVariant::PlayerDied{grave: Grave{
        player: detective.player_ref(),
        died_phase: GravePhase::Night,
        day_number: 2,
        information: GraveInformation::Normal { 
            role: Role::Detective, 
            will: "".to_string(), 
            death_cause: GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Yer)]), 
            death_notes: Vec::new(),
        },
        
    }};
    

    assert!(yer.get_messages().contains(&grave_message));
    assert!(detective.get_messages().contains(&grave_message));
    assert!(convertee.get_messages().contains(&grave_message));
    assert!(gossip.get_messages().contains(&grave_message));
    assert!(informant.get_messages().contains(&grave_message));

    /* Conversion & Disguise Test */
    yer.send_ability_input_boolean_typical(true);
    yer.send_ability_input_player_list(convertee, 1);
    yer.send_ability_input(AbilityInput::new(
        ControllerID::Role { player: yer.player_ref(), role: Role::Yer, id: 2 }, 
        RoleListSelection(vec!(Role::Zealot)) 
    ));
    
    game.skip_to(PhaseType::Dusk, 4);

    assert!(!yer.alive());
    assert!(!detective.alive());
    assert!(convertee.alive());
    assert!(gossip.alive());
    assert!(informant.alive());

    assert!(detective.role() == Role::Detective);
    assert!(convertee.role() == Role::Yer);

    let grave_message = ChatMessageVariant::PlayerDied{grave: Grave{
        player: yer.player_ref(),
        died_phase: GravePhase::Night,
        day_number: 3,
        information: GraveInformation::Normal { 
            role: Role::Zealot, 
            will: "".to_string(), 
            death_cause: GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Yer)]), 
            death_notes: Vec::new(),
        },
    }};

    assert!(yer.get_messages().contains(&grave_message));
    assert!(detective.get_messages().contains(&grave_message));
    assert!(convertee.get_messages().contains(&grave_message));
    assert!(gossip.get_messages().contains(&grave_message));
    assert!(informant.get_messages().contains(&grave_message));

    /* Idiot Proofing Test */
    convertee.send_ability_input_boolean_typical(true);
    convertee.send_ability_input_player_list(informant, 1);
    convertee.send_ability_input(AbilityInput::new(
        ControllerID::Role { player: yer.player_ref(), role: Role::Yer, id: 2 }, 
        RoleListSelection(vec!(Role::Zealot)) 
    ));
    
    game.skip_to(PhaseType::Dusk, 5);

    println!("messages: {}", kit::_format_messages_debug(convertee.get_messages()));

    assert!(!yer.alive());
    assert!(!detective.alive());
    assert!(!convertee.alive());
    assert!(gossip.alive());
    assert!(informant.alive());

    assert!(detective.role() == Role::Detective);
    assert!(convertee.role() == Role::Yer);
    assert!(informant.role() == Role::Yer);

    let grave_message = ChatMessageVariant::PlayerDied{grave: Grave{
        player: convertee.player_ref(),
        died_phase: GravePhase::Night,
        day_number: 4,
        information: GraveInformation::Normal { 
            role: Role::Engineer, 
            will: "".to_string(), 
            death_cause: GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Yer)]), 
            death_notes: Vec::new(),
        },
    }};

    assert!(yer.get_messages().contains(&grave_message));
    assert!(detective.get_messages().contains(&grave_message));
    assert!(convertee.get_messages().contains(&grave_message));
    assert!(gossip.get_messages().contains(&grave_message));
    assert!(informant.get_messages().contains(&grave_message));
}

#[test]
fn godfather_dies_to_veteran(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        gf: Godfather,
        _maf: Mortician
    );

    assert!(gf.send_ability_input_player_list_typical(vet));
    assert!(vet.send_ability_input_boolean_typical(true));

    game.next_phase();

    assert!(!gf.alive());
    assert!(vet.alive());
}

#[test]
fn witch_leaves_by_winning(){
    kit::scenario!(game in Night 2 where
        t: Veteran,
        gf: Godfather,
        arso: Arsonist,
        min: Witch
    );

    assert!(gf.send_ability_input_player_list_typical(t));

    game.next_phase();

    assert!(gf.alive());
    assert!(!min.alive());
    assert!(arso.alive());
    assert!(!t.alive());
}
#[test]
fn scarecrow_leaves_by_winning(){
    kit::scenario!(game in Night 2 where
        t: Veteran,
        gf: Godfather,
        arso: Arsonist,
        min: Scarecrow
    );

    assert!(gf.send_ability_input_player_list_typical(t));

    game.next_phase();

    assert!(gf.alive());
    assert!(!min.alive());
    assert!(arso.alive());
    assert!(!t.alive());
}
#[test]
fn witch_leaves_by_winning_puppeteer(){
    kit::scenario!(game in Night 2 where
        pup: Puppeteer,
        t: Armorsmith,
        t2: Detective,
        gf: Godfather,
        min: Witch
    );

    pup.send_ability_input(AbilityInput::new(
        ControllerID::role(pup.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(1)
    ));
    assert!(pup.send_ability_input_player_list_typical(t));

    game.skip_to(Night, 3);

    assert!(t.send_ability_input_boolean_typical(true));
    assert!(gf.send_ability_input_player_list_typical(t2));

    game.next_phase();

    assert!(gf.alive());
    assert!(!min.alive());
    assert!(!t.alive());
    assert!(!t2.alive());
    assert!(pup.alive());
}

#[test]
fn armorsmith_doesnt_get_wardblocked_when_warded(){
    kit::scenario!(game in Night 2 where
        gf: Godfather,
        armor: Armorsmith,
        bouncer: Bouncer
    );

    assert!(gf.send_ability_input_player_list_typical(bouncer));
    assert!(bouncer.send_ability_input_player_list_typical(armor));
    assert!(armor.send_ability_input_player_list_typical(bouncer));

    game.next_phase();

    assert!(gf.alive());
    assert!(armor.alive());
    assert!(bouncer.alive());

    assert_not_contains!(
        armor.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::Wardblocked
    );
    
    assert_contains!(
        bouncer.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::YouWereGuarded
    );
    
    assert_contains!(
        armor.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number: 2 }
        ),
        ChatMessageVariant::FragileVestBreak{ player_with_vest: bouncer.player_ref(), defense: DefensePower::Protected }
    );
}

#[test]
fn godfather_dies_to_veteran_after_possessed(){
    kit::scenario!(game in Night 2 where
        vet: Veteran,
        gf: Godfather,
        _maf: Mortician,
        min: Witch
    );

    assert!(min.send_ability_input_two_player_typical(gf, vet));
    assert!(vet.send_ability_input_boolean_typical(true));

    game.next_phase();

    assert!(!gf.alive());
    assert!(vet.alive());
    assert!(min.alive());
}

#[test]
fn fiends_wildcard_defense_upgrade(){
    kit::scenario!(game in Dusk 2 where
        fiend: FiendsWildcard,
        mafia: Godfather
    );
    
    fiend.send_ability_input(AbilityInput::new(
        ControllerID::role(fiend.player_ref(), Role::FiendsWildcard, 0),
        RoleListSelection(vec!(Role::Puppeteer))
    ));

    game.next_phase();

    fiend.send_ability_input(AbilityInput::new(
        ControllerID::role(fiend.player_ref(), Role::Puppeteer, 1),
        IntegerSelection(1)
    ));

    assert_eq!(fiend.role(), Role::Puppeteer);
    assert!(mafia.send_ability_input_player_list_typical(fiend));
    assert!(fiend.send_ability_input_player_list_typical(mafia));

    game.next_phase();

    assert!(fiend.alive());
    assert!(mafia.alive());

    assert!(game.game_is_over());
}

#[test]
fn spiraling_player_infects_visitors() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        townie1: Villager,
        townie2: Snoop
    );
    spiral.send_ability_input_player_list_typical(townie1);


    townie2.send_ability_input_player_list_typical(townie1);

    game.skip_to(Obituary, 3);
    assert!(!townie1.alive());
    assert!(townie2.alive());

    game.skip_to(Obituary, 4);
    assert!(!townie2.alive());
}

#[test]
fn spiral_can_select_when_no_spiraling_players() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        townie1: Villager,
        townie2: Snoop,
        townie3: Villager,
        _townie4: Villager
    );

    spiral.send_ability_input_player_list_typical(townie1);
    townie2.send_ability_input_player_list_typical(townie1);
    //kill 1
    //spiral 2

    game.skip_to(Night, 3);
    assert!(spiral.alive());
    assert!(!townie1.alive());
    assert!(townie2.alive());


    //kill 2
    //nobody is spiraling
    spiral.send_ability_input_player_list_typical(townie2);

    game.skip_to(Night, 4);
    assert!(spiral.alive());
    assert!(!townie2.alive());
    assert!(townie3.alive());

    //nobody is spiraling so we can kill 3
    assert!(spiral.send_ability_input_player_list_typical(townie3));
}

#[test]
fn spiral_does_not_kill_protected_player() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        doctor: Doctor
    );
    spiral.send_ability_input_player_list_typical(doctor);

    doctor.send_ability_input_player_list_typical(doctor);

    game.skip_to(Obituary, 3);
    assert!(doctor.alive());
}

#[test]
fn killed_player_is_not_spiraling() {
    kit::scenario!(game in Night 2 where
        spiral: Spiral,
        townie: Villager
    );
    spiral.send_ability_input_player_list_typical(townie);

    game.skip_to(Obituary, 3);
    assert!(!townie.alive());
}

#[test]
fn santa_cannot_convert_naughty_player() {
    kit::scenario!(game in Night 1 where
        santa: SantaClaus,
        nice: Villager,
        naughty: Villager
    );
    santa.send_ability_input_player_list_typical(nice);

    game.skip_to(Night, 2);

    assert_contains!(
        nice.player_ref().win_condition(&game).win_if_any_conclusions().unwrap(),
        GameConclusion::NiceList
    );

    assert_contains!(santa.get_messages_after_night(2), 
        ChatMessageVariant::NextSantaAbility { ability: mafia_server::game::role::santa_claus::SantaListKind::Naughty }
    );
    santa.send_ability_input_player_list(naughty, 1);

    game.skip_to(Night, 3);

    assert_contains!(
        naughty.player_ref().win_condition(&game).win_if_any_conclusions().unwrap(),
        GameConclusion::NaughtyList
    );

    santa.send_ability_input_player_list_typical(naughty);

    game.skip_to(Obituary, 4);

    assert_contains!(
        naughty.player_ref().win_condition(&game).win_if_any_conclusions().unwrap(),
        GameConclusion::NaughtyList
    );

    assert_not_contains!(
        naughty.player_ref().win_condition(&game).win_if_any_conclusions().unwrap(),
        GameConclusion::NiceList
    );
}

#[test]
fn krampus_obeys_ability_order() {
    kit::scenario!(game in Night 1 where
        krampus: Krampus,
        town1: Villager,
        town2: Villager,
        town3: Villager
    );

    use mafia_server::game::role::krampus::KrampusAbility;

    let expect_ability = |night: u8, ability: KrampusAbility| {
        assert_contains!(krampus.get_messages_after_night(night), ChatMessageVariant::NextKrampusAbility { ability });
    };

    expect_ability(1, KrampusAbility::DoNothing);
    krampus.send_ability_input_player_list_typical(town2);

    game.skip_to(Night, 2);

    expect_ability(2, KrampusAbility::Kill);
    krampus.send_ability_input_player_list_typical(town1);

    assert_not_contains!(town2.get_messages_after_night(3), ChatMessageVariant::YouDied);
    game.skip_to(Obituary, 3);
    assert!(!town1.alive());

    game.skip_to(Night, 3);
    expect_ability(3, KrampusAbility::DoNothing);
    krampus.send_ability_input_player_list_typical(town2);

    game.skip_to(Obituary, 4);
    assert!(town2.alive());

    game.skip_to(Night, 4);
    expect_ability(4, KrampusAbility::Kill);

    game.skip_to(Night, 5);
    expect_ability(5, KrampusAbility::Kill);
    krampus.send_ability_input_player_list_typical(town3);

    game.skip_to(Obituary, 6);
    assert!(!town3.alive());

    game.skip_to(Night, 6);
    expect_ability(6, KrampusAbility::DoNothing);

    game.skip_to(Night, 7);
    expect_ability(7, KrampusAbility::Kill);
}

#[test]
fn only_santa_and_krampus_ends_instantly() {
    kit::scenario!(game in Nomination 2 where
        santa: SantaClaus,
        krampus: Krampus,
        town: Villager
    );

    santa.vote_for_player(town);
    krampus.vote_for_player(town);

    game.skip_to(Judgement, 2);

    santa.set_verdict(Verdict::Guilty);
    krampus.set_verdict(Verdict::Guilty);

    game.skip_to(Dusk, 2);

    assert!(game.game_is_over());
}

#[test]
fn santa_always_gets_their_naughty_selection() {
    for _ in 0..20 {
        kit::scenario!(game in Night 1 where
            santa: SantaClaus,
            nice: Villager,
            naughty: Villager,
            _potential1: Villager,
            _potential2: Villager,
            _potential3: Villager,
            _potential4: Villager,
            _potential5: Villager
        );
        santa.send_ability_input_player_list_typical(nice);
    
        game.skip_to(Night, 2);
    
        santa.send_ability_input_player_list(naughty, 1);
    
        game.skip_to(Obituary, 3);
    
        assert_contains!(
            naughty.player_ref().win_condition(&game).win_if_any_conclusions().unwrap(),
            GameConclusion::NaughtyList
        );
    }
}
#[test]
fn werewolf_kills_visiting_target() {
    kit::scenario!(game in Night 2 where
        werewolf: Werewolf,
        target: Snoop,
        bystander: Snoop
    );
    werewolf.send_ability_input_player_list_typical(target);
    target.send_ability_input_player_list_typical(bystander);

    game.skip_to(Night, 3);
    werewolf.send_ability_input_player_list_typical(target);
    target.send_ability_input_player_list_typical(bystander);
    game.next_phase();
    assert!(!target.alive());
}
#[test]
fn werewolf_kills_nonvisiting_target() {
    kit::scenario!(game in Night 2 where
        werewolf: Werewolf,
        target: Snoop,
        bystander: Snoop
    );
    werewolf.send_ability_input_player_list_typical(target);
    target.send_ability_input_player_list_typical(bystander);

    game.skip_to(Night, 3);
    werewolf.send_ability_input_player_list_typical(target);
    game.next_phase();
    assert!(target.alive());
}
#[test]
fn werewolf_spares_nonvisiting_target() {
    kit::scenario!(game in Night 2 where
        werewolf: Werewolf,
        target: Snoop,
        bystander: Snoop,
        _bystander2: Snoop,
        _bystander3: Snoop
    );
    werewolf.send_ability_input_player_list_typical(target);
    target.send_ability_input_player_list_typical(bystander);

    game.skip_to(Night, 3);
    werewolf.send_ability_input_player_list_typical(bystander);
    game.next_phase();
    assert!(bystander.alive());
}

#[test]
fn werewolf_rampage() {
    kit::scenario!(game in Night 2 where
        werewolf: Werewolf,
        target: Snoop,
        bystander: Snoop
    );
    werewolf.send_ability_input_player_list_typical(target);
    target.send_ability_input_player_list_typical(bystander);

    game.skip_to(Night, 3);
    werewolf.send_ability_input_player_list_typical(target);
    bystander.send_ability_input_player_list_typical(target);
    game.next_phase();
    assert!(!bystander.alive());
}
#[test]
fn werewolf_nonrampage() {
    kit::scenario!(game in Night 2 where
        werewolf: Werewolf,
        target: Snoop,
        bystander: Snoop
    );
    werewolf.send_ability_input_player_list_typical(bystander);
    target.send_ability_input_player_list_typical(bystander);

    game.skip_to(Night, 3);
    werewolf.send_ability_input_player_list_typical(target);
    bystander.send_ability_input_player_list_typical(target);
    game.next_phase();
    assert!(bystander.alive());
}

#[test]
fn enraged_werewolf_kills() {
    kit::scenario!(game in Night 2 where
        werewolf: Werewolf,
        target: Snoop,
        bystander: Snoop
    );
    werewolf.send_ability_input_player_list_typical(target);
    target.send_ability_input_player_list_typical(bystander);

    game.skip_to(Night, 3);
    werewolf.send_ability_input_player_list_typical(target);
    bystander.send_ability_input_player_list_typical(target);
    game.next_phase();
    assert!(!bystander.alive());
}

#[test]
fn recruiter_role_list_is_correct() {
    let (game, _assignments) = mock_game(
        Settings {
            role_list: RoleList(vec![
                RoleOutline {options: vec1::vec1![RoleOutlineOption {
                    roles: RoleOutlineOptionRoles::Role { role: Role::Recruiter },
                    win_condition: RoleOutlineOptionWinCondition::RoleDefault,
                    insider_groups: RoleOutlineOptionInsiderGroups::RoleDefault,
                    player_pool: vec_set![0]
                }]},
                RoleOutline {options: vec1::vec1![RoleOutlineOption {
                    roles: RoleOutlineOptionRoles::Role { role: Role::Goon },
                    win_condition: RoleOutlineOptionWinCondition::RoleDefault,
                    insider_groups: RoleOutlineOptionInsiderGroups::RoleDefault,
                    player_pool: vec_set![1]
                }]},
                RoleOutline {options: vec1::vec1![RoleOutlineOption {
                    roles: RoleOutlineOptionRoles::Role { role: Role::Detective },
                    win_condition: RoleOutlineOptionWinCondition::RoleDefault,
                    insider_groups: RoleOutlineOptionInsiderGroups::RoleDefault,
                    player_pool: vec_set![2]
                }]},
                RoleOutline {options: vec1::vec1![RoleOutlineOption {
                    roles: RoleOutlineOptionRoles::Role { role: Role::Detective },
                    win_condition: RoleOutlineOptionWinCondition::RoleDefault,
                    insider_groups: RoleOutlineOptionInsiderGroups::RoleDefault,
                    player_pool: vec_set![3]
                }]}
            ]),
            phase_times: PhaseTimeSettings::default(),
            enabled_roles: RoleSet::Any.get_roles(),
            enabled_modifiers: vec_set![],
        },
        4
    ).unwrap();

    let [recruiter, goon, detective1, detective2] = [0, 1, 2, 3].map(|i| unsafe { PlayerReference::new_unchecked(i) } );

    assert!(recruiter.role(&game) == Role::Recruiter);
    assert!(goon.role(&game) != Role::Goon);
    assert!(detective1.role(&game) == Role::Detective);
    assert!(detective2.role(&game) == Role::Detective);
}