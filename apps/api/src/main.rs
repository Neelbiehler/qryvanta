//! Qryvanta API composition root.

#![forbid(unsafe_code)]

mod api_config;
mod api_router;
mod api_services;
mod auth;
mod dev_seed;
mod dto;
mod error;
mod handlers;
mod middleware;
mod observability;
mod qrywell_sync;
mod redis_session_store;
mod state;

use qryvanta_core::AppError;
use tracing::info;
use uuid::Uuid;

use crate::api_config::SessionStoreBackend;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    api_config::init_tracing();
    let args = std::env::args().collect::<Vec<_>>();
    let command = args.get(1).map(String::as_str);

    let config = api_config::ApiConfig::load()?;
    if command == Some("print-secret-fingerprints") {
        print_secret_fingerprints(&config)?;
        return Ok(());
    }
    info!(
        physical_isolation_mode = %config.physical_isolation_mode.as_str(),
        physical_isolation_tenant_id = config.physical_isolation_tenant_id.map(|value| value.to_string()),
        physical_isolation_schema_template_configured = config.physical_isolation_schema_template.is_some(),
        physical_isolation_database_url_template_configured = config.physical_isolation_database_url_template.is_some(),
        "physical isolation profile configured"
    );

    let pool = api_services::connect_and_migrate(&config.database_url).await?;
    if config.migrate_only {
        info!("database migrations applied successfully");
        return Ok(());
    }

    if command == Some("seed-dev") {
        dev_seed::run(pool, &config).await?;
        return Ok(());
    }

    if command == Some("portability-export") {
        run_portability_export(&config, pool, &args).await?;
        return Ok(());
    }

    if command == Some("portability-import") {
        run_portability_import(&config, pool, &args).await?;
        return Ok(());
    }

    let app_state = api_services::build_app_state(pool.clone(), &config)?;
    qrywell_sync::spawn_qrywell_sync_worker(app_state.clone());
    let app = match config.session_store_backend {
        SessionStoreBackend::Postgres => {
            let session_layer =
                api_services::build_postgres_session_layer(pool.clone(), config.cookie_secure)
                    .await?;
            api_router::build_router(app_state, &config.frontend_url, session_layer)?
        }
        SessionStoreBackend::Redis => {
            let redis_url = config.redis_url.as_deref().ok_or_else(|| {
                AppError::Validation("REDIS_URL is required when SESSION_STORE=redis".to_owned())
            })?;
            let redis_client = api_services::build_redis_client(redis_url)?;
            let session_layer =
                api_services::build_redis_session_layer(redis_client, config.cookie_secure).await?;
            api_router::build_router(app_state, &config.frontend_url, session_layer)?
        }
    };
    let address = config.socket_address()?;

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| AppError::Internal(format!("failed to bind listener: {error}")))?;

    info!(%address, "qryvanta-api listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .map_err(|error| AppError::Internal(format!("api server error: {error}")))
}

async fn run_portability_export(
    config: &api_config::ApiConfig,
    pool: sqlx::PgPool,
    args: &[String],
) -> Result<(), AppError> {
    let output_path = required_arg_value(args, "--output")?;
    let tenant_id = parse_tenant_id(required_arg_value(args, "--tenant-id")?)?;
    let subject =
        optional_arg_value(args, "--subject").unwrap_or_else(|| "portability-cli".to_owned());
    let display_name =
        optional_arg_value(args, "--display-name").unwrap_or_else(|| subject.clone());
    let metadata_only = has_flag(args, "--metadata-only");
    let runtime_only = has_flag(args, "--runtime-only");

    if metadata_only && runtime_only {
        return Err(AppError::Validation(
            "cannot combine --metadata-only and --runtime-only".to_owned(),
        ));
    }

    let include_metadata = !runtime_only;
    let include_runtime_data = !metadata_only;

    let state = api_services::build_app_state(pool, config)?;
    let actor = qryvanta_core::UserIdentity::new(subject, display_name, None, tenant_id);

    let bundle = state
        .metadata_service
        .export_workspace_bundle(
            &actor,
            qryvanta_application::ExportWorkspaceBundleOptions {
                include_metadata,
                include_runtime_data,
            },
        )
        .await?;

    let encoded = serde_json::to_vec_pretty(&bundle).map_err(|error| {
        AppError::Internal(format!("failed to serialize export bundle: {error}"))
    })?;
    std::fs::write(output_path.as_str(), encoded).map_err(|error| {
        AppError::Internal(format!(
            "failed to write export bundle '{}': {error}",
            output_path
        ))
    })?;

    info!(output_path = %output_path, "workspace portability export completed");
    Ok(())
}

async fn run_portability_import(
    config: &api_config::ApiConfig,
    pool: sqlx::PgPool,
    args: &[String],
) -> Result<(), AppError> {
    let input_path = required_arg_value(args, "--input")?;
    let tenant_id = parse_tenant_id(required_arg_value(args, "--tenant-id")?)?;
    let subject =
        optional_arg_value(args, "--subject").unwrap_or_else(|| "portability-cli".to_owned());
    let display_name =
        optional_arg_value(args, "--display-name").unwrap_or_else(|| subject.clone());
    let dry_run = has_flag(args, "--dry-run");
    let skip_metadata = has_flag(args, "--skip-metadata");
    let skip_runtime = has_flag(args, "--skip-runtime");
    let remap_record_ids = has_flag(args, "--remap-record-ids");

    if skip_metadata && skip_runtime {
        return Err(AppError::Validation(
            "cannot combine --skip-metadata and --skip-runtime".to_owned(),
        ));
    }

    let raw_bundle = std::fs::read_to_string(input_path.as_str()).map_err(|error| {
        AppError::Internal(format!(
            "failed to read import bundle '{}': {error}",
            input_path
        ))
    })?;
    let bundle: qryvanta_application::WorkspacePortableBundle = serde_json::from_str(&raw_bundle)
        .map_err(|error| {
        AppError::Validation(format!(
            "invalid import bundle JSON in '{}': {error}",
            input_path
        ))
    })?;

    let state = api_services::build_app_state(pool, config)?;
    let actor = qryvanta_core::UserIdentity::new(subject, display_name, None, tenant_id);

    let summary = state
        .metadata_service
        .import_workspace_bundle(
            &actor,
            bundle,
            qryvanta_application::ImportWorkspaceBundleOptions {
                dry_run,
                import_metadata: !skip_metadata,
                import_runtime_data: !skip_runtime,
                remap_record_ids,
            },
        )
        .await?;

    let summary_json = serde_json::to_string_pretty(&summary).map_err(|error| {
        AppError::Internal(format!("failed to serialize import summary: {error}"))
    })?;
    println!("{summary_json}");

    info!("workspace portability import command completed");
    Ok(())
}

fn required_arg_value(args: &[String], flag: &str) -> Result<String, AppError> {
    optional_arg_value(args, flag)
        .ok_or_else(|| AppError::Validation(format!("missing required argument {flag}")))
}

fn optional_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|argument| argument == flag)
        .and_then(|index| args.get(index + 1))
        .map(ToOwned::to_owned)
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|argument| argument == flag)
}

fn parse_tenant_id(value: String) -> Result<qryvanta_core::TenantId, AppError> {
    let uuid = Uuid::parse_str(value.as_str())
        .map_err(|error| AppError::Validation(format!("invalid tenant id '{}': {error}", value)))?;
    Ok(qryvanta_core::TenantId::from_uuid(uuid))
}

fn print_secret_fingerprints(config: &api_config::ApiConfig) -> Result<(), AppError> {
    let deployment_environment = std::env::var("DEPLOYMENT_ENVIRONMENT")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Validation(
                "DEPLOYMENT_ENVIRONMENT is required for print-secret-fingerprints".to_owned(),
            )
        })?;
    let fingerprints = config.secret_fingerprint_records(deployment_environment.as_str());
    let output = serde_json::to_string_pretty(&fingerprints).map_err(|error| {
        AppError::Internal(format!("failed to serialize fingerprints: {error}"))
    })?;
    println!("{output}");
    Ok(())
}
