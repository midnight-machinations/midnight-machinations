#[cfg(test)]
mod business_tests {
    use crate::game::role::Role;
    use crate::game::role::business::Business;
    use crate::game::role::RoleStateImpl;
    use crate::game::attack_power::DefensePower;

    #[test]
    fn business_has_correct_defense() {
        assert_eq!(crate::game::role::business::DEFENSE, DefensePower::None);
    }

    #[test]
    fn business_has_correct_max_count() {
        assert_eq!(crate::game::role::business::MAXIMUM_COUNT, Some(1));
    }

    #[test]
    fn business_default_state() {
        let business = Business::default();
        assert_eq!(business.bribes_remaining, 3);
        assert_eq!(business.bribed_players.len(), 0);
    }

    #[test]
    fn business_is_mafia() {
        let business = Business::default();
        let revealed_groups = business.default_revealed_groups();
        assert!(revealed_groups.contains(&crate::game::components::insider_group::InsiderGroupID::Mafia));
    }
}
