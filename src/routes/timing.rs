use std::time::{Duration, Instant};

/// Duration with cache hit/miss status for Server-Timing header
#[derive(Clone, Copy)]
pub struct CacheableDuration {
    pub duration: Duration,
    pub cached: bool,
}

impl CacheableDuration {
    pub fn new(duration: Duration, cached: bool) -> Self {
        Self { duration, cached }
    }

    /// Record timing from an Instant, marking as cache miss
    pub fn miss_since(start: Instant) -> Self {
        Self::new(start.elapsed(), false)
    }
}

/// Collector for Server-Timing metrics
pub struct ServerTiming {
    pub logo: Option<CacheableDuration>,
    pub image: Option<CacheableDuration>,
    pub queue: Option<Duration>,
    pub template: Option<Duration>,
    pub render: Option<Duration>,
}

impl ServerTiming {
    pub fn new() -> Self {
        Self {
            logo: None,
            image: None,
            queue: None,
            template: None,
            render: None,
        }
    }

    /// Format as Server-Timing header value
    pub fn to_header_value(&self) -> String {
        let mut parts = Vec::new();

        if let Some(cd) = self.logo {
            parts.push(format!(
                "logo;dur={:.1};desc=\"{}\"",
                cd.duration.as_secs_f64() * 1000.0,
                if cd.cached { "cache.hit" } else { "cache.miss" }
            ));
        }

        if let Some(cd) = self.image {
            parts.push(format!(
                "image;dur={:.1};desc=\"{}\"",
                cd.duration.as_secs_f64() * 1000.0,
                if cd.cached { "cache.hit" } else { "cache.miss" }
            ));
        }

        if let Some(dur) = self.queue {
            parts.push(format!("queue;dur={:.1}", dur.as_secs_f64() * 1000.0));
        }

        if let Some(dur) = self.template {
            parts.push(format!("template;dur={:.1}", dur.as_secs_f64() * 1000.0));
        }

        if let Some(dur) = self.render {
            parts.push(format!("render;dur={:.1}", dur.as_secs_f64() * 1000.0));
        }

        parts.join(", ")
    }
}

impl Default for ServerTiming {
    fn default() -> Self {
        Self::new()
    }
}
