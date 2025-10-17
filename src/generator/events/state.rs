use std::collections::HashMap;

pub struct ImageReplacement {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

/// State for tracking SVG processing
pub struct State {
    /// Tracks how deep we are inside a skipped/replaced element
    /// 0 means we're not inside any skipped element
    /// >0 means we're inside a skipped element (and possibly nested children)
    pub skip_depth: usize,

    /// The ID of the image group currently being replaced
    /// When set, we're waiting for a <rect> child to define image bounds
    pub replacement_id: Option<String>,

    /// Map of element IDs to their replacement text values
    pub text_replacements: HashMap<String, String>,

    /// Map of element IDs to their replacement images
    /// None means remove the element entirely, Some means replace with image
    pub image_replacements: HashMap<String, Option<ImageReplacement>>,
}

impl State {
    pub fn new(
        text_replacements: HashMap<String, String>,
        image_replacements: HashMap<String, Option<ImageReplacement>>,
    ) -> Self {
        Self {
            skip_depth: 0,
            text_replacements,
            image_replacements,
            replacement_id: None,
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
