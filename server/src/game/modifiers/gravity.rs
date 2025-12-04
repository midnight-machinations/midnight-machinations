use serde::{Deserialize, Serialize};

use crate::game::{
    attack_power::AttackPower,
    chat::ChatMessageVariant,
    components::{
        attack::night_attack::NightAttack,
        graves::grave::GraveKiller,
        night_visits::Visits,
    },
    event::on_midnight::{OnMidnightFold, OnMidnightPriority},
    player::PlayerReference,
    Game,
};

use super::{ModifierID, ModifierSettings, ModifierState, ModifierStateImpl};

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum GravityLevel {
    #[default]
    ZeroGravity,
    AntiGravity,
}

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Gravity {
    pub level: GravityLevel,
}

impl From<&Gravity> for ModifierID {
    fn from(_: &Gravity) -> Self {
        ModifierID::Gravity
    }
}

impl Gravity {
    fn get_level(settings: &ModifierSettings) -> Option<GravityLevel> {
        if let Some(ModifierState::Gravity(gravity)) = settings.get_modifier(ModifierID::Gravity) {
            Some(gravity.level)
        } else {
            None
        }
    }

    pub fn is_zero_gravity(game: &Game) -> bool {
        Self::get_level(game.modifier_settings()) == Some(GravityLevel::ZeroGravity)
    }

    pub fn is_anti_gravity(game: &Game) -> bool {
        Self::get_level(game.modifier_settings()) == Some(GravityLevel::AntiGravity)
    }
}

impl ModifierStateImpl for Gravity {
    fn on_midnight(self, game: &mut Game, fold: &mut OnMidnightFold, priority: OnMidnightPriority) {
        // Anti-gravity: Kill players who go outside (direct visits)
        if self.level != GravityLevel::AntiGravity {
            return;
        }
        if priority != OnMidnightPriority::Kill {
            return;
        }

        // Find all unique players who made direct visits (i.e., went outside)
        let direct_visitors: Vec<PlayerReference> = Visits::into_iter(fold)
            .filter(|v| !v.indirect)
            .map(|v| v.visitor)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for visitor in direct_visitors {
            if visitor.night_died(fold) {
                continue;
            }

            visitor.push_night_message(fold, ChatMessageVariant::GravityFloatedAway);
            
            // Gravity is physics - it bypasses all protection (ProtectionPiercing)
            // because you can't defend against floating away into space
            NightAttack::new()
                .power(AttackPower::ProtectionPiercing)
                .hide_messages()
                .grave_killer(GraveKiller::Suicide)
                .attack(game, fold, visitor);
        }
    }
}
