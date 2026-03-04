use std::fmt::Write as _;

use qryvanta_application::WorkflowQueueStats;

use super::ApiObservabilitySnapshot;

/// Renders Prometheus text format for API + workflow queue metrics.
#[must_use]
pub fn render_metrics_prometheus(
    snapshot: ApiObservabilitySnapshot,
    queue_stats: Option<WorkflowQueueStats>,
    slow_request_threshold_ms: u64,
    slow_query_threshold_ms: u64,
) -> String {
    let error_total = snapshot.http_4xx_total + snapshot.http_5xx_total;
    let error_ratio = if snapshot.http_requests_total == 0 {
        0.0
    } else {
        error_total as f64 / snapshot.http_requests_total as f64
    };
    let avg_duration_ms = if snapshot.http_requests_total == 0 {
        0.0
    } else {
        snapshot.http_request_duration_ms_total as f64 / snapshot.http_requests_total as f64
    };

    let mut output = String::new();
    let _ = writeln!(output, "# TYPE qryvanta_http_requests_total counter");
    let _ = writeln!(
        output,
        "qryvanta_http_requests_total {}",
        snapshot.http_requests_total
    );
    let _ = writeln!(output, "# TYPE qryvanta_http_in_flight gauge");
    let _ = writeln!(
        output,
        "qryvanta_http_in_flight {}",
        snapshot.http_in_flight
    );
    let _ = writeln!(output, "# TYPE qryvanta_http_2xx_total counter");
    let _ = writeln!(
        output,
        "qryvanta_http_2xx_total {}",
        snapshot.http_2xx_total
    );
    let _ = writeln!(output, "# TYPE qryvanta_http_4xx_total counter");
    let _ = writeln!(
        output,
        "qryvanta_http_4xx_total {}",
        snapshot.http_4xx_total
    );
    let _ = writeln!(output, "# TYPE qryvanta_http_5xx_total counter");
    let _ = writeln!(
        output,
        "qryvanta_http_5xx_total {}",
        snapshot.http_5xx_total
    );
    let _ = writeln!(output, "# TYPE qryvanta_http_error_ratio gauge");
    let _ = writeln!(output, "qryvanta_http_error_ratio {:.6}", error_ratio);
    let _ = writeln!(output, "# TYPE qryvanta_http_request_duration_ms_avg gauge");
    let _ = writeln!(
        output,
        "qryvanta_http_request_duration_ms_avg {:.3}",
        avg_duration_ms
    );
    let _ = writeln!(output, "# TYPE qryvanta_http_request_duration_ms_max gauge");
    let _ = writeln!(
        output,
        "qryvanta_http_request_duration_ms_max {}",
        snapshot.http_request_duration_ms_max
    );
    let _ = writeln!(output, "# TYPE qryvanta_http_slow_requests_total counter");
    let _ = writeln!(
        output,
        "qryvanta_http_slow_requests_total {}",
        snapshot.http_slow_requests_total
    );
    let _ = writeln!(
        output,
        "# TYPE qryvanta_runtime_query_backpressure_rejections_total counter"
    );
    let _ = writeln!(
        output,
        "qryvanta_runtime_query_backpressure_rejections_total {}",
        snapshot.runtime_query_backpressure_rejections_total
    );
    let _ = writeln!(
        output,
        "# TYPE qryvanta_workflow_burst_backpressure_rejections_total counter"
    );
    let _ = writeln!(
        output,
        "qryvanta_workflow_burst_backpressure_rejections_total {}",
        snapshot.workflow_burst_backpressure_rejections_total
    );
    let _ = writeln!(
        output,
        "# TYPE qryvanta_http_slow_request_threshold_ms gauge"
    );
    let _ = writeln!(
        output,
        "qryvanta_http_slow_request_threshold_ms {}",
        slow_request_threshold_ms
    );
    let _ = writeln!(
        output,
        "# TYPE qryvanta_runtime_slow_query_threshold_ms gauge"
    );
    let _ = writeln!(
        output,
        "qryvanta_runtime_slow_query_threshold_ms {}",
        slow_query_threshold_ms
    );

    let _ = writeln!(
        output,
        "# TYPE qryvanta_workflow_queue_stats_available gauge"
    );
    let _ = writeln!(
        output,
        "qryvanta_workflow_queue_stats_available {}",
        if queue_stats.is_some() { 1 } else { 0 }
    );

    if let Some(stats) = queue_stats {
        let _ = writeln!(output, "# TYPE qryvanta_workflow_pending_jobs gauge");
        let _ = writeln!(
            output,
            "qryvanta_workflow_pending_jobs {}",
            stats.pending_jobs
        );
        let _ = writeln!(output, "# TYPE qryvanta_workflow_leased_jobs gauge");
        let _ = writeln!(
            output,
            "qryvanta_workflow_leased_jobs {}",
            stats.leased_jobs
        );
        let _ = writeln!(output, "# TYPE qryvanta_workflow_completed_jobs gauge");
        let _ = writeln!(
            output,
            "qryvanta_workflow_completed_jobs {}",
            stats.completed_jobs
        );
        let _ = writeln!(output, "# TYPE qryvanta_workflow_failed_jobs gauge");
        let _ = writeln!(
            output,
            "qryvanta_workflow_failed_jobs {}",
            stats.failed_jobs
        );
        let _ = writeln!(output, "# TYPE qryvanta_workflow_expired_leases gauge");
        let _ = writeln!(
            output,
            "qryvanta_workflow_expired_leases {}",
            stats.expired_leases
        );
        let _ = writeln!(output, "# TYPE qryvanta_workflow_active_workers gauge");
        let _ = writeln!(
            output,
            "qryvanta_workflow_active_workers {}",
            stats.active_workers
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use super::render_metrics_prometheus;
    use crate::observability::ApiObservabilitySnapshot;

    #[test]
    fn prometheus_render_includes_backpressure_counters() {
        let output = render_metrics_prometheus(
            ApiObservabilitySnapshot {
                http_requests_total: 10,
                http_in_flight: 0,
                http_2xx_total: 8,
                http_4xx_total: 2,
                http_5xx_total: 0,
                http_request_duration_ms_total: 100,
                http_request_duration_ms_max: 30,
                http_slow_requests_total: 1,
                runtime_query_backpressure_rejections_total: 4,
                workflow_burst_backpressure_rejections_total: 2,
            },
            None,
            1000,
            250,
        );

        assert!(output.contains("qryvanta_runtime_query_backpressure_rejections_total 4"));
        assert!(output.contains("qryvanta_workflow_burst_backpressure_rejections_total 2"));
    }
}
