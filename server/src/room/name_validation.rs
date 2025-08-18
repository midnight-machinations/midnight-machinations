use crate::strings::TidyableString;
use lazy_static::lazy_static;
use rand::seq::IndexedRandom;

lazy_static!(
    static ref RANDOM_NAMES: Vec<String> = {
        let mut random_names = Vec::new();
        random_names.append(&mut 
            include_str!("../../resources/random_names/default_names.csv").lines()
                .map(str::to_string)
                .collect()
        );
        random_names.append(&mut 
            include_str!("../../resources/random_names/extra_names.csv").lines()
                .map(str::to_string)
                .collect()
        );

        random_names
    };
);

const MAX_NAME_LENGTH: usize = 20;
const MAX_SERVER_NAME_LENGTH: usize = 20;
pub const DEFAULT_SERVER_NAME: &str = "Mafia Room";

/// Sanitizes a player name.
/// If the desired name is invalid or taken, this generates a random acceptable name.
/// Otherwise, this trims and returns the input name.
pub fn sanitize_name(mut desired_name: String, other_names: &[String]) -> String {
    desired_name = desired_name
        .remove_newline()
        .trim_whitespace()
        .truncate(MAX_NAME_LENGTH)
        .truncate_lines(1);

    let name_already_taken = other_names.contains(&desired_name);
    
    if !desired_name.is_empty() && !name_already_taken {
        desired_name
    } else {
        generate_random_name(&other_names.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
    }
}

pub fn sanitize_server_name(desired_name: String) -> String {
    desired_name
        .remove_newline()
        .trim_whitespace()
        .truncate(MAX_SERVER_NAME_LENGTH)
        .truncate_lines(1)
}

pub fn generate_random_name(taken_names: &[&str]) -> String{
    let taken_names_str = taken_names.iter().map(
        |existing_name|
        existing_name.to_string()
        .remove_newline()
        .trim_whitespace()
        .truncate(MAX_NAME_LENGTH)
        .truncate_lines(1)
    ).collect::<Vec<String>>();
    let available_random_names = RANDOM_NAMES.iter().filter(|new_random_name| 
        !taken_names_str.contains(new_random_name)
    ).collect::<Vec<&String>>();

    if let Some(random_name) = available_random_names.choose(&mut rand::rng()) {
        (*random_name).clone()
    } else {
        let mut i: u16 = 0;
        loop {
            i = i.saturating_add(1);
            if !taken_names_str.contains(&i.to_string()) {
                return i.to_string()
            }
        }
    }
}