mod handlers;
pub(crate) mod replacements; // Public within crate for handlers to use
mod state;

// Re-export handlers for backward compatibility
pub use handlers::{handle_default, handle_empty, handle_end, handle_start};

// Re-export state
pub use state::State;
