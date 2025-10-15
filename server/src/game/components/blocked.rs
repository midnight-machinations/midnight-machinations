use crate::game::{
    components::{night_visits::Visits, player_component::PlayerComponent}, event::{on_midnight::{MidnightVariables, OnMidnightPriority}, on_phase_start::OnPhaseStart, on_player_roleblocked::OnPlayerRoleblocked, on_visit_wardblocked::OnVisitWardblocked}, phase::PhaseType, player::PlayerReference, role::Role, visit::VisitTag, Game
};


pub type BlockedComponent = PlayerComponent<Blocked>;
pub struct Blocked{
    blocked: bool
}
impl Blocked{
    fn new()->Self{
        Self { blocked: false }
    }
}
impl BlockedComponent{
    /// # Safety
    /// player_count is correct
    pub unsafe fn new(player_count: u8)->Self{
        unsafe { PlayerComponent::new_component_box(player_count, |_|Blocked::new()) }
    }

    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        if !matches!(event.phase.phase(), PhaseType::Night) {return}
        for player in PlayerReference::all_players(game) {
            game.blocked.get_mut(player).blocked = false;
        }
    }

    pub fn blocked(game: &Game, player: PlayerReference)->bool{
        game.blocked.get(player).blocked
    }

    pub fn set_blocked(game: &mut Game, player: PlayerReference){
        game.blocked.get_mut(player).blocked = true;
    }
}

impl PlayerReference {
    pub fn roleblock(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, send_messages: bool) {
        OnPlayerRoleblocked::new(*self, !send_messages).invoke(game, midnight_variables);
    }

    pub fn ward_night_action(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) -> Vec<PlayerReference> {
        match priority {
            OnMidnightPriority::PreWard => self.pre_ward(game, midnight_variables),
            OnMidnightPriority::Ward => self.ward(game, midnight_variables),
            _ => vec![]
        }
    }

    fn pre_ward(&self, game: &mut Game, midnight_variables: &mut MidnightVariables) -> Vec<PlayerReference> {
        let mut out = Vec::new();
        for visit in Visits::into_iter(midnight_variables) {
            if visit.wardblock_immune {
                continue;
            }
            if !matches!(visit.tag,
                VisitTag::Role { role: Role::Transporter, .. } |
                VisitTag::Role { role: Role::Warper, .. } |
                VisitTag::Role { role: Role::Porter, .. } |
                VisitTag::Role { role: Role::Polymath, id: 3 } |

                VisitTag::Role { role: Role::Witch, .. } |
                VisitTag::Role { role: Role::Retributionist, .. } |
                VisitTag::Role { role: Role::Necromancer, .. }
            ) {
                continue;
            }
            if visit.target != *self {continue;}
            OnVisitWardblocked::new(visit).invoke(game, midnight_variables);
            out.push(visit.visitor);
        }
        out
    }
    fn ward(&self, game: &mut Game, midnight_variables: &mut MidnightVariables) -> Vec<PlayerReference> {
        let mut out = Vec::new();
        for visit in Visits::into_iter(midnight_variables) {
            if visit.wardblock_immune {
                continue;
            }
            if visit.target != *self {continue;}
            OnVisitWardblocked::new(visit).invoke(game, midnight_variables);
            out.push(visit.visitor);
        }
        out
    }
}