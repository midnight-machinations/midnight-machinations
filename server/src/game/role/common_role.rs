use std::collections::HashSet;



use crate::game::{chat::ChatGroup, components::{call_witness::CallWitness, detained::Detained, puppeteer_marionette::PuppeteerMarionette, silenced::Silenced, win_condition::WinCondition}, controllers::{ControllerID, ControllerSelection}, game_conclusion::GameConclusion, modifiers::ModifierID, phase::{PhaseState, PhaseType}, player::PlayerReference, role_list::RoleSet, visit::{Visit, VisitTag}, Game};

use super::{medium::Medium, reporter::Reporter, warden::Warden, InsiderGroupID, Role, RoleState};


pub fn standard_charges(game: &Game)->u8{
    game.num_players().div_ceil(5)
}


/// This function uses defaults. When using this function, consider if you need to override the defaults.
/// Defaults to VisitTag::Role { role: actor_ref.role(game), id: 0 }
pub(super) fn convert_controller_selection_to_visits(game: &Game, actor_ref: PlayerReference, controller_id: ControllerID, attack: bool) -> Vec<Visit> {
    convert_controller_selection_to_visits_visit_tag(game, actor_ref, controller_id, attack, VisitTag::Role { role: actor_ref.role(game), id: 0 })
}

pub(super) fn convert_controller_selection_to_visits_visit_tag(game: &Game, actor_ref: PlayerReference, controller_id: ControllerID, attack: bool, tag: VisitTag) -> Vec<Visit> {
    
    let Some(selection) = controller_id.get_selection(game) else {return Vec::new()};

    match selection {
        ControllerSelection::Unit(_) => vec![
            Visit{
                visitor: actor_ref,
                target: actor_ref,
                tag,
                attack,
                wardblock_immune: false,
                transport_immune: false,
                investigate_immune: false,
                indirect: false
            }
        ],
        ControllerSelection::TwoPlayerOption(selection) => {
            if let Some((target_1, target_2)) = selection.0 {
                vec![
                    Visit{
                        visitor: actor_ref,
                        target: target_1,
                        tag,
                        attack,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    },
                    Visit{
                        visitor: actor_ref,
                        target: target_2,
                        tag,
                        attack,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    }
                ]
            }else{
                vec![]
            }
        },
        ControllerSelection::PlayerList(selection) => {
            selection.0
                .iter()
                .map(|target_ref|
                    Visit{
                        visitor: actor_ref,
                        target: *target_ref,
                        tag,
                        attack,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    }
                )
                .collect()
        }
        ControllerSelection::RoleList(selection) => {
            selection.0
                .iter()
                .flat_map(|role|
                    PlayerReference::all_players(game)
                        .filter_map(|player|
                            if player.role(game) == *role {
                                Some(
                                    Visit{
                                        visitor: actor_ref,
                                        target: player,
                                        tag,
                                        attack,
                                        wardblock_immune: false,
                                        transport_immune: false,
                                        investigate_immune: false,
                                        indirect: false
                                    }
                                )
                            }else{
                                None
                            }
                        )
                )
                .collect()
        }
        ControllerSelection::TwoRoleOption(selection) => {
            let mut out = Vec::new();
            for player in PlayerReference::all_players(game){
                if Some(player.role(game)) == selection.0 {
                    out.push(
                        Visit{
                            visitor: actor_ref,
                            target: player,
                            tag,
                            attack,
                            wardblock_immune: false,
                            transport_immune: false,
                            investigate_immune: false,
                            indirect: false
                        }
                    );
                }
                if Some(player.role(game)) == selection.1 {
                    out.push(
                        Visit{
                            visitor: actor_ref,
                            target: player,
                            tag,
                            attack,
                            wardblock_immune: false,
                            transport_immune: false,
                            investigate_immune: false,
                            indirect: false
                        }
                    );
                }
            }
            out
        }
        ControllerSelection::TwoRoleOutlineOption(selection) => {
            let mut out = vec![];
            if let Some(chosen_outline) = selection.0{
                let (_, player) = chosen_outline.deref_as_role_and_player_originally_generated(game);
                out.push(
                    Visit{
                        visitor: actor_ref,
                        target: player,
                        tag,
                        attack,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    }
                );
            }
            if let Some(chosen_outline) = selection.1{
                let (_, player) = chosen_outline.deref_as_role_and_player_originally_generated(game);
                out.push(
                    Visit{
                        visitor: actor_ref,
                        target: player,
                        tag,
                        attack,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    }
                );
            }
            out
        },
        _ => Vec::new()
    }
}

pub(super) fn convert_controller_selection_to_visits_possession(game: &Game, actor_ref: PlayerReference, controller_id: ControllerID) -> Vec<Visit> {
    let Some(selection) = controller_id.get_selection(game) else {return Vec::new()};

    if let ControllerSelection::TwoPlayerOption(selection) = selection {
        if let Some((target_1, target_2)) = selection.0 {
            vec![
                Visit::new_role(actor_ref, target_1, false, actor_ref.role(game), 0 ),
                Visit{
                    visitor: actor_ref,
                    target: target_2,
                    tag: VisitTag::Role { role: actor_ref.role(game), id: 1 },
                    attack: false,
                    wardblock_immune: true,
                    transport_immune: true,
                    investigate_immune: true,
                    indirect: true
                }
            ]
        }else{
            vec![]
        }
    }else{
        vec![]
    }
}





pub(super) fn get_current_send_chat_groups(game: &Game, actor_ref: PlayerReference, mut night_chat_groups: Vec<ChatGroup>) -> HashSet<ChatGroup> {
    if game.current_phase().phase() == PhaseType::Recess {
        return vec![ChatGroup::All].into_iter().collect()
    }
    if 
        !actor_ref.alive(game) && 
        !game.modifier_settings().is_enabled(ModifierID::DeadCanChat)
    {
        if PuppeteerMarionette::marionettes_and_puppeteer(game).contains(&actor_ref){
            return vec![ChatGroup::Dead, ChatGroup::Puppeteer].into_iter().collect();
        }
        return vec![ChatGroup::Dead].into_iter().collect();
    }
    if Silenced::silenced(game, actor_ref) {
        return HashSet::new();
    }

    match game.current_phase() {
        PhaseState::Briefing => HashSet::new(),
        PhaseState::Obituary { .. } => {
            let mut out = HashSet::new();

            //evil chat groups
            for group in InsiderGroupID::all_groups_with_player(game, actor_ref) {
                out.insert(group.get_insider_chat_group());
            }

            //medium
            if PlayerReference::all_players(game)
                .any(|med|{
                    match med.role_state(game) {
                        RoleState::Medium(Medium{ haunted_target: Some(seanced_target), .. }) => {
                            actor_ref == *seanced_target
                        },
                        _ => false
                    }
                })
            {
                out.insert(ChatGroup::Dead);
            }

            out
        },
        PhaseState::Discussion
        | PhaseState::Adjournment { .. }
        | PhaseState::Nomination {..}
        | PhaseState::Judgement {..}
        | PhaseState::FinalWords {..}
        | PhaseState::Dusk 
        | PhaseState::Recess => vec![ChatGroup::All].into_iter().collect(),
        &PhaseState::Testimony { .. } => {
            let mut out = HashSet::new();
            if CallWitness::witness_called(game).contains(&actor_ref) {
                out.insert(ChatGroup::All);
            }
            out
        },
        PhaseState::Night => {
            let mut out = vec![];
            //medium seance
            if PlayerReference::all_players(game)
                .any(|med|{
                    match med.role_state(game) {
                        RoleState::Medium(Medium{ haunted_target: Some(seanced_target), .. }) => {
                            actor_ref == *seanced_target
                        },
                        _ => false
                    }
                })
            {
                out.push(ChatGroup::Dead);
            }
            //reporter interview
            if 
                PlayerReference::all_players(game).any(|p|
                    match p.role_state(game) {
                        RoleState::Reporter(Reporter{interviewed_target: Some(interviewed_target_ref), ..}) => {
                            *interviewed_target_ref == actor_ref
                        },
                        _ => false
                    }
                )
            {
                out.push(ChatGroup::Interview);
            }
            if
                PlayerReference::all_players(game).any(|p|
                    match p.role_state(game) {
                        RoleState::Warden(Warden{players_in_prison}) => {
                            players_in_prison.contains(&actor_ref)
                        },
                        _ => false
                    }
                )
            {
                out.push(ChatGroup::Warden);
            }


            let mut jail_or_night_chats = 
            if Detained::is_detained(game, actor_ref) && PlayerReference::all_players(game).any(|detainer|
                match detainer.role_state(game) {
                    RoleState::Jailor(jailor) => {
                        jailor.jailed_target_ref == Some(actor_ref)
                    },
                    _ => false
                }
            ) {
                vec![ChatGroup::Jail]
            }else if Detained::is_detained(game, actor_ref) && PlayerReference::all_players(game).any(|detainer|
                match detainer.role_state(game) {
                    RoleState::Kidnapper(kidnapper) => {
                        kidnapper.jailed_target_ref == Some(actor_ref)
                    },
                    _ => false
                }
            ) {
                vec![ChatGroup::Kidnapped]
            }else{
                for group in InsiderGroupID::all_groups_with_player(game, actor_ref) {
                    night_chat_groups.push(group.get_insider_chat_group());
                }

                night_chat_groups
            };


            out.append(&mut jail_or_night_chats);
            out.into_iter().collect()
        },
    }
}
pub(super) fn get_current_receive_chat_groups(game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
    let mut out = Vec::new();

    out.push(ChatGroup::All);

    if !actor_ref.alive(game){
        out.push(ChatGroup::Dead);
    }

    for group in InsiderGroupID::all_groups_with_player(game, actor_ref) {
        out.push(group.get_insider_chat_group());
    }

    if Detained::is_detained(game, actor_ref) {
        if PlayerReference::all_players(game).any(|detainer|
            match detainer.role_state(game) {
                RoleState::Jailor(jailor) => {
                    jailor.jailed_target_ref == Some(actor_ref)
                },
                _ => false
            }
        ) {
            out.push(ChatGroup::Jail);
        }
        if PlayerReference::all_players(game).any(|detainer|
            match detainer.role_state(game) {
                RoleState::Kidnapper(kidnapper) => {
                    kidnapper.jailed_target_ref == Some(actor_ref)
                },
                _ => false
            }
        ) {
            out.push(ChatGroup::Kidnapped);
        }
    }
    
    if 
        game.current_phase().phase() == PhaseType::Night && 
        PlayerReference::all_players(game)
            .any(|med|{
                match med.role_state(game) {
                    RoleState::Medium(Medium{ haunted_target: Some(seanced_target), .. }) => {
                        actor_ref == *seanced_target
                    },
                    _ => false
                }
            })
    {
        out.push(ChatGroup::Dead);
    }
    if 
        game.current_phase().phase() == PhaseType::Night && 
        PlayerReference::all_players(game).any(|p|
            match p.role_state(game) {
                RoleState::Reporter(Reporter{interviewed_target: Some(interviewed_target_ref), ..}) => {
                    *interviewed_target_ref == actor_ref
                },
                _ => false
            }
        )
    {
        out.push(ChatGroup::Interview);
    }
    if 
        game.current_phase().phase() == PhaseType::Night && 
        PlayerReference::all_players(game).any(|detainer|
            match detainer.role_state(game) {
                RoleState::Warden(warden) => {
                    warden.players_in_prison.contains(&actor_ref)
                },
                _ => false
            }
        )
    {
        out.push(ChatGroup::Warden);
    }

    out.into_iter().collect()
}

///Only works for roles that win based on end game condition
pub(super) fn default_win_condition(role: Role) -> WinCondition {
    if RoleSet::Mafia.get_roles().contains(&role) {
        WinCondition::GameConclusionReached{win_if_any: vec![GameConclusion::Mafia].into_iter().collect()}

    }else if RoleSet::Cult.get_roles().contains(&role) {
        WinCondition::GameConclusionReached{win_if_any: vec![GameConclusion::Cult].into_iter().collect()}

    }else if RoleSet::Town.get_roles().contains(&role) {
        WinCondition::GameConclusionReached{win_if_any: vec![GameConclusion::Town].into_iter().collect()}

    }else if RoleSet::Fiends.get_roles().contains(&role) {
        WinCondition::GameConclusionReached{win_if_any: vec![GameConclusion::Fiends].into_iter().collect()}

    }else if RoleSet::Minions.get_roles().contains(&role) {
        // Minions aren't added to generic groups, so this check is sufficient.
        WinCondition::GameConclusionReached{win_if_any: GameConclusion::all_static().into_iter().filter(|end_game_condition|
            !matches!(end_game_condition, 
                GameConclusion::Town | GameConclusion::Draw |
                GameConclusion::NiceList | GameConclusion::NaughtyList
            )
        ).collect()}

    }else{
        WinCondition::RoleStateWon
    }
}