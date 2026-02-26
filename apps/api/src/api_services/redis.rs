use qryvanta_core::AppError;

pub fn build_redis_client(redis_url: &str) -> Result<redis::Client, AppError> {
    redis::Client::open(redis_url)
        .map_err(|error| AppError::Validation(format!("invalid REDIS_URL: {error}")))
}
