use crate::vec_map::VecMap;
use crate::game::prelude::*;


#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum TransportPriority {
    Transporter = 0,
    Warper = 1,
    Bodyguard = 2,
    None = 3
}
impl TransportPriority{
    fn from_visit_tag(visit_tag: &VisitTag) -> TransportPriority {
        let VisitTag::Ability{ability: AbilityID::Role { role, .. }, ..} = visit_tag else {return TransportPriority::None};
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
        midnight_variables: &mut OnMidnightFold,
        transport_priority: TransportPriority, 
        player_map: VecMap<PlayerReference, PlayerReference>,
        filter: impl Fn(&Visit) -> bool + Clone + Send + 'static,
        send_message: bool, 
    ) {

        if send_message {
            player_map
                .keys()
                .for_each(|p|
                    p.push_night_message(midnight_variables, ChatMessageVariant::Transported)
                );
        }
        
        Visits::push_ledger_event(midnight_variables, move |visits|{
            visits
                .iter_mut()
                .filter(|v|filter(v))
                .filter(|v|!v.transport_immune)
                .filter(|v|transport_priority.can_transport(&TransportPriority::from_visit_tag(&v.tag)))
                .for_each(|v|
                    if let Some(new_target) = player_map.get(&v.target){
                        v.target = *new_target;
                    }
                );
        });
    }
}