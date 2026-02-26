/// Configuration for a rate limit rule.
#[derive(Debug, Clone)]
pub struct RateLimitRule {
    /// The route prefix or category name (e.g., "login", "forgot_password").
    pub category: String,
    /// Maximum number of attempts allowed in the window.
    pub max_attempts: i32,
    /// Window duration in seconds.
    pub window_seconds: i64,
}

impl RateLimitRule {
    /// Creates a new rate limit rule.
    #[must_use]
    pub fn new(category: impl Into<String>, max_attempts: i32, window_seconds: i64) -> Self {
        Self {
            category: category.into(),
            max_attempts,
            window_seconds,
        }
    }
}
