// Meta
#![warn(clippy::allow_attributes, clippy::allow_attributes_without_reason)]
// Arithmetic
#![warn(clippy::arithmetic_side_effects)]
// Casting
#![warn(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_precision_loss, clippy::cast_sign_loss)]
// Collections
#![warn(clippy::clear_with_drain, clippy::cloned_instead_of_copied, clippy::collection_is_never_read, clippy::explicit_into_iter_loop, clippy::flat_map_option)]
// Panicking operations
#![deny(clippy::unwrap_used, clippy::panic, clippy::indexing_slicing)]
// Misc.
#![warn(clippy::expl_impl_clone_on_copy)]
#![allow(clippy::new_without_default, reason = "This lint is stupid")]

pub mod game;
pub mod websocket_connections;
pub mod websocket_listener;
pub mod room;
pub mod lobby;
pub mod packet;
pub mod client_connection;
pub mod vec_map;
pub mod vec_set;
pub mod webrtc_sfu;

pub mod log {
    #[macro_export]
    /// Log a statement to the console. 
    /// When logging using this macro, a timestamp and possibly a marker is added to the message.
    /// 
    /// # Examples
    /// ```
    /// use mafia_server::log;
    /// log!(error "Error location"; "Error message");
    /// log!(error "Game::new"; "Failed to generate role. rolelist wasnt big enough for number of players");
    /// log!(info "Listener"; "{}: {}", "Received message", "message");
    /// ```
    /// 
    /// # Markers
    /// - `fatal`: Prints the word FATAL
    /// - `error`: Prints red and writes "WARN"
    /// - `important`: 
    /// - `info`: 
    /// 
    /// if none are put then it defaults to info
    /// 
    macro_rules! log {
        // Each case in this macro definition is for a different log marker.
        // None
        ($expr:expr) => {
            println!("\x1b[0;90m{}\x1b[0m {}", chrono::Local::now().format("%m.%d %I:%M:%S"), $expr)
        };
        // Fatal error
        (fatal $prefix:expr; $($expr:expr),*) => {
            log!(&format!("\x1b[0;1;91m[{}] FATAL\x1b[0m \x1b[0;1;41m{}\x1b[0m", $prefix, &format!($($expr),*)))
        };
        // Warning error
        (error $prefix:expr; $($expr:expr),*) => {
            log!(&format!("\x1b[0;1;91m[{}] WARN\x1b[0m {}", $prefix, &format!($($expr),*)))
        };
        // Important
        (important $prefix:expr; $($expr:expr),*) => {
            log!(&format!("\x1b[0;1;93m[{}]\x1b[0m {}", $prefix, &format!($($expr),*)))
        };
        // Info
        (info $prefix:expr; $($expr:expr),*) => {
            log!(&format!("\x1b[0;1;32m[{}]\x1b[0m {}", $prefix, &format!($($expr),*)))
        };
        // Default (use info)
        ($prefix:expr; $($expr:expr),*) => {
            log!(info $prefix; $($expr),*)
        };
    }
}
pub mod strings{
    pub trait TidyableString {
        fn trim_whitespace(&self) -> Self;
        fn remove_newline(&self) -> Self;
        fn trim_newline(&self) -> Self;
        fn truncate(&self, max_chars: usize) -> Self;
        fn truncate_lines(&self, max_lines: usize) -> Self;
    }
    impl TidyableString for String {
        /// Removes multiple whitespace in a row.
        fn trim_whitespace(&self) -> String {
            let mut new_str = self.trim().to_owned();
            let mut prev = ' ';
            new_str.retain(|ch| {
                let result = ch != ' ' || prev != ' ';
                prev = ch;
                result
            });
            new_str
        }
        fn remove_newline(&self) -> String {
            let mut new_str = self.trim().to_owned();
            new_str.retain(|ch| {ch != '\n'});
            new_str
        }
        /// Removes more than two newlines in a row
        fn trim_newline(&self) -> String {
            let mut new_str = self.trim().to_owned();
            let mut prev = (' ', ' ');
            new_str.retain(|ch| {
                let result = ch != '\n' || prev.0 != '\n' || prev.1 != '\n';
                (prev.0, prev.1) = (prev.1, ch);
                result
            });
            new_str
        }
        /// Truncates to a given number of unicode characters, rather than rust's [`String::truncate`]
        /// which truncates to a given number of unicode code points.
        fn truncate(&self, max_chars: usize) -> String {
            match self.char_indices().nth(max_chars) {
                None => self.clone(),
                Some((idx, _)) => self[..idx].to_string(),
            }
        }
        /// Truncates to a given number of lines
        fn truncate_lines(&self, max_lines: usize) -> String {
            let out: String = self.lines()
                .take(max_lines)
                .map(|str| str.to_string() + "\n")
                .collect();

            // Remove trailing newline
            out[0..(out.len().saturating_sub(1))].to_string()
        }
    }
}