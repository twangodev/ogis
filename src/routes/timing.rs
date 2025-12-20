use axum::http::HeaderValue;
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
    ///
    /// Returns a valid HeaderValue. The format only produces ASCII characters
    /// (digits, letters, semicolons, quotes, periods, commas, spaces), so this
    /// conversion is infallible in practice.
    pub fn to_header_value(&self) -> HeaderValue {
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

        let value = parts.join(", ");
        // Safe: format only produces ASCII (digits, letters, semicolons, quotes, dots, commas, spaces)
        HeaderValue::try_from(value).unwrap_or_else(|_| HeaderValue::from_static(""))
    }
}

impl Default for ServerTiming {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cacheable_duration_new() {
        let cd = CacheableDuration::new(Duration::from_millis(100), true);
        assert_eq!(cd.duration, Duration::from_millis(100));
        assert!(cd.cached);

        let cd = CacheableDuration::new(Duration::from_millis(200), false);
        assert_eq!(cd.duration, Duration::from_millis(200));
        assert!(!cd.cached);
    }

    #[test]
    fn test_cacheable_duration_miss_since() {
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(1));
        let cd = CacheableDuration::miss_since(start);
        assert!(!cd.cached);
        assert!(cd.duration >= Duration::from_millis(1));
    }

    #[test]
    fn test_server_timing_empty() {
        let timing = ServerTiming::new();
        assert_eq!(timing.to_header_value().to_str().unwrap(), "");
    }

    #[test]
    fn test_server_timing_logo_cache_hit() {
        let mut timing = ServerTiming::new();
        timing.logo = Some(CacheableDuration::new(Duration::from_millis(5), true));
        assert_eq!(
            timing.to_header_value().to_str().unwrap(),
            "logo;dur=5.0;desc=\"cache.hit\""
        );
    }

    #[test]
    fn test_server_timing_logo_cache_miss() {
        let mut timing = ServerTiming::new();
        timing.logo = Some(CacheableDuration::new(Duration::from_millis(150), false));
        assert_eq!(
            timing.to_header_value().to_str().unwrap(),
            "logo;dur=150.0;desc=\"cache.miss\""
        );
    }

    #[test]
    fn test_server_timing_simple_durations() {
        let mut timing = ServerTiming::new();
        timing.queue = Some(Duration::from_micros(100)); // 0.1ms
        timing.template = Some(Duration::from_millis(10));
        timing.render = Some(Duration::from_millis(50));
        assert_eq!(
            timing.to_header_value().to_str().unwrap(),
            "queue;dur=0.1, template;dur=10.0, render;dur=50.0"
        );
    }

    #[test]
    fn test_server_timing_full() {
        let mut timing = ServerTiming::new();
        timing.logo = Some(CacheableDuration::new(Duration::from_millis(2), true));
        timing.image = Some(CacheableDuration::new(Duration::from_millis(200), false));
        timing.queue = Some(Duration::from_millis(0));
        timing.template = Some(Duration::from_millis(8));
        timing.render = Some(Duration::from_millis(45));
        assert_eq!(
            timing.to_header_value().to_str().unwrap(),
            "logo;dur=2.0;desc=\"cache.hit\", image;dur=200.0;desc=\"cache.miss\", queue;dur=0.0, template;dur=8.0, render;dur=45.0"
        );
    }

    #[test]
    fn test_server_timing_default() {
        let timing = ServerTiming::default();
        assert!(timing.logo.is_none());
        assert!(timing.image.is_none());
        assert!(timing.queue.is_none());
        assert!(timing.template.is_none());
        assert!(timing.render.is_none());
    }
}
