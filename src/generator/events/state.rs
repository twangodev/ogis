/// Context data available for replacements
#[derive(Clone)]
pub struct ReplacementContext<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub logo: &'a str,
    pub subtitle: &'a str,
    pub logo_image_base64: Option<&'a str>,
}

impl<'a> ReplacementContext<'a> {
    pub fn new(
        title: &'a str,
        description: &'a str,
        logo: &'a str,
        subtitle: &'a str,
        logo_image_base64: Option<&'a str>,
    ) -> Self {
        Self {
            title,
            description,
            logo,
            subtitle,
            logo_image_base64,
        }
    }
}

/// State for tracking SVG processing
pub struct State<'a> {
    /// Tracks how deep we are inside a skipped/replaced element
    /// 0 means we're not inside any skipped element
    /// >0 means we're inside a skipped element (and possibly nested children)
    pub skip_depth: usize,

    /// Context data for replacements (title, description, etc.)
    pub context: ReplacementContext<'a>,
}

impl<'a> State<'a> {
    pub fn new(context: ReplacementContext<'a>) -> Self {
        Self {
            skip_depth: 0,
            context,
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
