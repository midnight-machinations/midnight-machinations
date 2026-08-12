use crate::game::prelude::*;
use super::Role;


pub fn standard_charges(game: &Game)->u8{
    game.num_players().div_ceil(5)
}
pub fn on_player_roleblocked(id: &AbilityID, midnight_variables: &mut OnMidnightFold, player: PlayerReference){
    if id.get_player_from_role_id().is_some_and(|actor|actor == player) {
        remove_all_visits_associated_with_ability(id, midnight_variables);
    }
}
pub fn on_visit_wardblocked(id: &AbilityID, midnight_variables: &mut OnMidnightFold, visit: Visit){
    if
        let VisitTag::Ability { ability: visit_ability, .. } = visit.tag &&
        visit_ability == *id
    {
        remove_all_visits_associated_with_ability(id, midnight_variables);
    }
}

pub fn remove_all_visits_associated_with_ability(id: &AbilityID, midnight_variables: &mut OnMidnightFold){
    Visits::retain(midnight_variables, |v|
        if let VisitTag::Ability { ability, .. } = v.tag {
            ability != *id
        }else{
            true
        }
    );
}


/// This function uses defaults. When using this function, consider if you need to override the defaults.
/// Defaults to VisitTag::Role { role: actor_ref.role(game), id: 0 }
pub(super) fn convert_controller_selection_to_visits(game: &Game, actor_ref: PlayerReference, controller_id: ControllerID, attack: bool) -> Vec<Visit> {
    let ControllerID::Role { role, .. } = controller_id else {unreachable!("Only call this function for role controllers")};
    convert_controller_selection_to_visits_visit_tag(
        game,
        actor_ref,
        controller_id,
        attack,
        VisitTag::Ability { ability: AbilityID::Role { role, player: actor_ref }, id: 0 }
    )
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

pub(super) fn convert_controller_selection_to_visits_possession(
    game: &Game,
    actor_ref: PlayerReference,
    
    controller_id: ControllerID,

    tag_possess: VisitTag,
    tag_possess_into: VisitTag,
) -> Vec<Visit> {
    let Some(ControllerSelection::TwoPlayerOption(
        TwoPlayerOptionSelection(Some((target_1, target_2)))
    )) = controller_id.get_selection(game) else {return Vec::new()};

    vec![
        Visit{
            visitor: actor_ref,
            target: *target_1,
            tag: tag_possess,
            attack: false,
            wardblock_immune: false,
            transport_immune: false,
            investigate_immune: false,
            indirect: false
        },
        Visit{
            visitor: actor_ref,
            target: *target_2,
            tag: tag_possess_into,
            attack: false,
            wardblock_immune: true,
            transport_immune: true,
            investigate_immune: true,
            indirect: true
        }
    ]
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
        WinCondition::GameConclusionReached{win_if_any: GameConclusion::all().into_iter().filter(|end_game_condition|
            !matches!(end_game_condition, 
                GameConclusion::Town | GameConclusion::Draw |
                GameConclusion::NiceList | GameConclusion::NaughtyList
            )
        ).collect()}

    }else{
        WinCondition::RoleStateWon
    }
}