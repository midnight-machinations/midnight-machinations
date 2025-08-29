
use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Abstaining;

/*
    There is modifier specific code in the set_verdict() function
*/
impl From<&Abstaining> for ModifierID{
    fn from(_: &Abstaining) -> Self {
        ModifierID::Abstaining
    }
}
impl ModifierStateImpl for Abstaining{}
