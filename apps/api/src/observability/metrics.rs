use std::sync::atomic::{AtomicU64, Ordering};

/// In-memory API observability counters exposed through `/metrics`.
#[derive(Debug, Default)]
pub struct ApiObservabilityMetrics {
    http_requests_total: AtomicU64,
    http_in_flight: AtomicU64,
    http_2xx_total: AtomicU64,
    http_4xx_total: AtomicU64,
    http_5xx_total: AtomicU64,
    http_request_duration_ms_total: AtomicU64,
    http_request_duration_ms_max: AtomicU64,
    http_slow_requests_total: AtomicU64,
    runtime_query_backpressure_rejections_total: AtomicU64,
    workflow_burst_backpressure_rejections_total: AtomicU64,
}

/// Snapshot of API request counters.
#[derive(Debug, Clone, Copy)]
pub struct ApiObservabilitySnapshot {
    pub http_requests_total: u64,
    pub http_in_flight: u64,
    pub http_2xx_total: u64,
    pub http_4xx_total: u64,
    pub http_5xx_total: u64,
    pub http_request_duration_ms_total: u64,
    pub http_request_duration_ms_max: u64,
    pub http_slow_requests_total: u64,
    pub runtime_query_backpressure_rejections_total: u64,
    pub workflow_burst_backpressure_rejections_total: u64,
}

impl ApiObservabilityMetrics {
    /// Marks one request start.
    pub fn on_request_start(&self) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
        self.http_in_flight.fetch_add(1, Ordering::Relaxed);
    }

    /// Marks one request end and records status/duration.
    pub fn on_request_end(&self, status_code: u16, elapsed_ms: u64, slow_threshold_ms: u64) {
        self.http_in_flight
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                Some(value.saturating_sub(1))
            })
            .ok();

        if (200..300).contains(&status_code) {
            self.http_2xx_total.fetch_add(1, Ordering::Relaxed);
        } else if (400..500).contains(&status_code) {
            self.http_4xx_total.fetch_add(1, Ordering::Relaxed);
        } else if status_code >= 500 {
            self.http_5xx_total.fetch_add(1, Ordering::Relaxed);
        }

        self.http_request_duration_ms_total
            .fetch_add(elapsed_ms, Ordering::Relaxed);
        self.http_request_duration_ms_max
            .fetch_max(elapsed_ms, Ordering::Relaxed);

        if elapsed_ms >= slow_threshold_ms {
            self.http_slow_requests_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Records one runtime-query backpressure rejection.
    pub fn on_runtime_query_backpressure_rejection(&self) {
        self.runtime_query_backpressure_rejections_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Records one workflow-burst backpressure rejection.
    pub fn on_workflow_burst_backpressure_rejection(&self) {
        self.workflow_burst_backpressure_rejections_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Returns a consistent counter snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ApiObservabilitySnapshot {
        ApiObservabilitySnapshot {
            http_requests_total: self.http_requests_total.load(Ordering::Relaxed),
            http_in_flight: self.http_in_flight.load(Ordering::Relaxed),
            http_2xx_total: self.http_2xx_total.load(Ordering::Relaxed),
            http_4xx_total: self.http_4xx_total.load(Ordering::Relaxed),
            http_5xx_total: self.http_5xx_total.load(Ordering::Relaxed),
            http_request_duration_ms_total: self
                .http_request_duration_ms_total
                .load(Ordering::Relaxed),
            http_request_duration_ms_max: self.http_request_duration_ms_max.load(Ordering::Relaxed),
            http_slow_requests_total: self.http_slow_requests_total.load(Ordering::Relaxed),
            runtime_query_backpressure_rejections_total: self
                .runtime_query_backpressure_rejections_total
                .load(Ordering::Relaxed),
            workflow_burst_backpressure_rejections_total: self
                .workflow_burst_backpressure_rejections_total
                .load(Ordering::Relaxed),
        }
    }
}
