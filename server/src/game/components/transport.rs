use crate::game::visit::VisitTag;
use crate::game::{
    chat::ChatMessageVariant,
    event::on_midnight::MidnightVariables,
    player::PlayerReference,
    role::Role,
    visit::Visit,
};
use crate::vec_map::VecMap;

use super::night_visits::NightVisits;


#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub enum TransportPriority {
    Transporter = 0,
    Warper = 1,
    Bodyguard = 2,
    None = 3
}
impl TransportPriority{
    fn from_visit_tag(visit_tag: &VisitTag) -> TransportPriority {
        let VisitTag::Role{role, ..} = visit_tag else {return TransportPriority::None};
        Self::from_role(role)
    }
    fn from_role(role: &Role) -> TransportPriority {
        match role {
            Role::Transporter => TransportPriority::Transporter,
    
            Role::Warper |
            Role::Porter | 
            Role::Polymath => TransportPriority::Warper,
    
            Role::Bodyguard => TransportPriority::Bodyguard,
    
            _ => TransportPriority::None
        }
    }
    fn can_transport(&self, other: &Self)->bool{
        self < other
    }
}
pub struct Transport;
impl Transport{
    pub fn transport(
        midnight_variables: &mut MidnightVariables, transport_priority: TransportPriority, 
        player_map: &VecMap<PlayerReference, PlayerReference>, filter: impl Fn(&Visit) -> bool, send_message: bool, 
    ) -> Vec<Visit> {

        if send_message {
            player_map
                .keys()
                .for_each(|p|
                    p.push_night_message(midnight_variables, ChatMessageVariant::Transported)
                );
        }
        let mut out = vec![];
        
        NightVisits::all_visits_mut(midnight_variables)
            .filter(|v|filter(v))
            .filter(|v|v.transportable)
            .filter(|v|transport_priority.can_transport(&TransportPriority::from_visit_tag(&v.tag)))
            .for_each(|v|
                if let Some(new_target) = player_map.get(&v.target){
                    v.target = *new_target;
                    out.push(*v);
                }
            );
    
        out
    }
}