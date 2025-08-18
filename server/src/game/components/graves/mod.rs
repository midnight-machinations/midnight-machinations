use crate::game::{components::graves::{grave::Grave, grave_reference::GraveReference}, event::on_grave_added::OnGraveAdded, Game};

pub mod grave;
pub mod grave_reference;

#[derive(Default)]
pub struct Graves{
    graves: Vec<Grave>,
}
impl Graves{
    pub fn add_grave(game: &mut Game, grave: Grave) {
        if let Ok(grave_index) = game.graves.graves.len().try_into() {
            game.graves.graves.push(grave.clone());

            if let Some(grave_ref) = GraveReference::new(game, grave_index) {
                OnGraveAdded::new(grave_ref).invoke(game);
            }
        }
    }
}