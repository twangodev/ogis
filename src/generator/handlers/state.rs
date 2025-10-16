/// State for tracking SVG processing
#[derive(Default)]
pub struct State {
    /// Tracks how deep we are inside a skipped/replaced element
    /// 0 means we're not inside any skipped element
    /// >0 means we're inside a skipped element (and possibly nested children)
    pub skip_depth: usize,
}

impl State {
    pub fn new() -> Self {
        Self { skip_depth: 0 }
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