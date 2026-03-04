use qryvanta_core::AppError;

pub(super) fn validate_backpressure_config(
    runtime_query_max_limit: usize,
    runtime_query_max_in_flight: usize,
    workflow_burst_max_in_flight: usize,
) -> Result<(), AppError> {
    if runtime_query_max_limit == 0 {
        return Err(AppError::Validation(
            "RUNTIME_QUERY_MAX_LIMIT must be greater than zero".to_owned(),
        ));
    }
    if runtime_query_max_in_flight == 0 {
        return Err(AppError::Validation(
            "RUNTIME_QUERY_MAX_IN_FLIGHT must be greater than zero".to_owned(),
        ));
    }
    if workflow_burst_max_in_flight == 0 {
        return Err(AppError::Validation(
            "WORKFLOW_BURST_MAX_IN_FLIGHT must be greater than zero".to_owned(),
        ));
    }

    Ok(())
}
