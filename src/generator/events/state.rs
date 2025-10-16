use std::collections::HashMap;

/// State for tracking SVG processing
pub struct State {
    /// Tracks how deep we are inside a skipped/replaced element
    /// 0 means we're not inside any skipped element
    /// >0 means we're inside a skipped element (and possibly nested children)
    pub skip_depth: usize,

    /// Map of element IDs to their replacement text values
    pub text_replacements: HashMap<String, String>,
}

impl State {
    pub fn new(text_replacements: HashMap<String, String>) -> Self {
        Self {
            skip_depth: 0,
            text_replacements,
        }
    }

    /// Check if we're currently inside a skipped element
    pub fn is_skipping(&self) -> bool {
        self.skip_depth > 0
    }

    /// Start skipping an element
    pub fn start_skip(&mut self) {
        self.skip_depth += 1;
    }

    /// Stop skipping (decrement depth)
    pub fn end_skip(&mut self) {
        if self.skip_depth > 0 {
            self.skip_depth -= 1;
        }
    }
}
