//! Rate limiting ports and application service.
//!
//! Implements a sliding-window rate limiter backed by the `auth_rate_limits`
//! database table. Follows OWASP Credential Stuffing Prevention cheat sheet
//! recommendations for per-IP and per-endpoint throttling.

mod config;
mod ports;
mod service;

pub use config::RateLimitRule;
pub use ports::{AttemptInfo, RateLimitRepository};
pub use service::RateLimitService;
