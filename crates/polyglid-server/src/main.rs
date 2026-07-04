use axum::{
    extract::{Path as AxumPath, Query, State, ws::{Message, WebSocket, WebSocketUpgrade}},
    http::{header, HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use std::path::Path;
use std::sync::Arc;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use polyglid_config::AppConfig;
use polyglid_plugin_api::PluginId;
use polyglid_runtime::WasmRuntime;
use polyglid_core::{
    execution::ExecutionManager,
    plugin_manager::PluginManager,
    store::WorkspaceStore,
    services::{PluginService, ExecutionService, TargetService, ReportService, SettingsService},
};

mod auth;
#[cfg(test)]
mod tests;

#[derive(Clone)]
#[allow(dead_code)]
struct ServerState {
    plugin_service: Arc<PluginService<WasmRuntime>>,
    execution_service: Arc<ExecutionService<WasmRuntime>>,
    target_service: Arc<TargetService>,
    report_service: Arc<ReportService>,
    settings_service: Arc<SettingsService>,
    execution_manager: Arc<ExecutionManager<WasmRuntime>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load_from_env().unwrap_or_else(|_| AppConfig::development());
    let runtime = std::sync::Arc::new(WasmRuntime::new());
    let db_path = config.plugin_dir.parent().unwrap_or(&config.plugin_dir).join("polyglid.db");
    
    let store = WorkspaceStore::new(&db_path)?;
    let pm = Arc::new(PluginManager::new(runtime.clone(), &config, store.clone())?);
    let em = Arc::new(ExecutionManager::new(WasmRuntime::new(), Some(store.clone())));
    
    // Sync plugins
    let _ = pm.sync_directory();

    let admin_token = auth::initialize_auth_token(&store)?;

    let server_state = ServerState {
        plugin_service: Arc::new(PluginService::new(pm.clone())),
        execution_service: Arc::new(ExecutionService::new(em.clone(), store.clone())),
        target_service: Arc::new(TargetService::new(store.clone())),
        report_service: Arc::new(ReportService::new(store.clone())),
        settings_service: Arc::new(SettingsService::new(store.clone())),
        execution_manager: em.clone(),
    };

    let auth_state = auth::AuthState {
        expected_token: admin_token,
    };

    let api_routes = Router::new()
        // Plugins
        .route("/plugins", get(get_plugins).post(install_plugin))
        .route("/plugins/:id", delete(uninstall_plugin))
        .route("/plugins/:id/toggle", post(toggle_plugin))
        // Executions
        .route("/executions", get(list_executions).post(run_execution))
        .route("/executions/:id", get(get_execution))
        // Targets
        .route("/targets", get(list_targets).post(add_target))
        .route("/targets/:name", delete(remove_target))
        // Reports
        .route("/reports", get(list_reports))
        .route("/reports/:id", get(get_report))
        .route("/reports/:id/download", get(download_report))
        .layer(middleware::from_fn_with_state(
            auth_state,
            auth::auth_middleware,
        ))
        .with_state(server_state.clone());

    let ws_routes = Router::new()
        .route("/ws/v1/events", get(ws_handler))
        .with_state(server_state);

    let app = Router::new()
        .nest("/api/v1", api_routes)
        .merge(ws_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        );

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("PolyGlid server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Handlers implementation

async fn get_plugins(
    State(state): State<ServerState>,
) -> Result<Json<Vec<polyglid_config::plugin_registry::PluginRegistryEntry>>, (StatusCode, String)> {
    state.plugin_service.list_plugins()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

#[derive(serde::Deserialize)]
struct InstallRequest {
    path: String,
}

async fn install_plugin(
    State(state): State<ServerState>,
    Json(req): Json<InstallRequest>,
) -> Result<Json<polyglid_config::plugin_registry::PluginRegistryEntry>, (StatusCode, String)> {
    state.plugin_service.install_plugin(Path::new(&req.path))
        .map(Json)
        .map_err(|err| (StatusCode::BAD_REQUEST, err))
}

async fn uninstall_plugin(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let pid = PluginId::new(&id).map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    state.plugin_service.uninstall_plugin(&pid)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

#[derive(serde::Deserialize)]
struct ToggleRequest {
    enabled: bool,
}

async fn toggle_plugin(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
    Json(req): Json<ToggleRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let pid = PluginId::new(&id).map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    state.plugin_service.toggle_plugin(&pid, req.enabled)
        .map(|_| StatusCode::OK)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

#[derive(serde::Deserialize)]
struct RunExecutionRequest {
    plugin_id: String,
    target: String,
}

#[derive(serde::Serialize)]
struct RunExecutionResponse {
    job_id: String,
}

async fn run_execution(
    State(state): State<ServerState>,
    Json(req): Json<RunExecutionRequest>,
) -> Result<(StatusCode, Json<RunExecutionResponse>), (StatusCode, String)> {
    let pid = PluginId::new(&req.plugin_id).map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    state.execution_service.run_plugin(&pid, &req.target)
        .map(|job_id| (StatusCode::ACCEPTED, Json(RunExecutionResponse { job_id })))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn list_executions(
    State(state): State<ServerState>,
) -> Result<Json<Vec<polyglid_core::store::execution_store::DbJobRecord>>, (StatusCode, String)> {
    state.execution_service.list_executions()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn get_execution(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<polyglid_core::store::execution_store::DbJobRecord>, (StatusCode, String)> {
    state.execution_service.get_execution(&id)
        .and_then(|opt| opt.ok_or_else(|| "Execution not found".to_string()))
        .map(Json)
        .map_err(|err| (StatusCode::NOT_FOUND, err))
}

async fn list_targets(
    State(state): State<ServerState>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    state.target_service.list_targets()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

#[derive(serde::Deserialize)]
struct AddTargetRequest {
    name: String,
}

async fn add_target(
    State(state): State<ServerState>,
    Json(req): Json<AddTargetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.target_service.add_target(&req.name)
        .map(|_| StatusCode::CREATED)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn remove_target(
    State(state): State<ServerState>,
    AxumPath(name): AxumPath<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.target_service.remove_target(&name)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn list_reports(
    State(state): State<ServerState>,
) -> Result<Json<Vec<polyglid_core::store::report_store::DbReportRecord>>, (StatusCode, String)> {
    state.report_service.list_reports()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn get_report(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<polyglid_core::store::report_store::DbReportRecord>, (StatusCode, String)> {
    state.report_service.get_report(&id)
        .and_then(|opt| opt.ok_or_else(|| "Report not found".to_string()))
        .map(Json)
        .map_err(|err| (StatusCode::NOT_FOUND, err))
}

#[derive(serde::Deserialize)]
struct DownloadParams {
    format: Option<String>,
}

async fn download_report(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
    Query(params): Query<DownloadParams>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let format = params.format.unwrap_or_else(|| {
        if let Some(accept) = headers.get(header::ACCEPT) {
            if let Ok(accept_str) = accept.to_str() {
                if accept_str.contains("html") {
                    return "html".to_string();
                } else if accept_str.contains("markdown") || accept_str.contains("text") {
                    return "markdown".to_string();
                } else if accept_str.contains("sarif") {
                    return "sarif".to_string();
                }
            }
        }
        "json".to_string()
    });

    let content = state.report_service.export_report(&id, &format)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;

    let content_type = match format.as_str() {
        "html" => "text/html",
        "markdown" | "md" => "text/markdown",
        "sarif" | "json" => "application/json",
        _ => "text/plain",
    };

    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"report-{}.{}\"", id, format))
        .body(axum::body::Body::from(content))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(response)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.execution_manager.clone()))
}

async fn handle_socket(mut socket: WebSocket, em: Arc<ExecutionManager<WasmRuntime>>) {
    let mut rx = em.subscribe();
    while let Ok(event) = rx.recv().await {
        let envelope = serde_json::json!({
            "type": format!("{:?}", event).to_lowercase(),
            "payload": event
        });
        let payload = serde_json::to_string(&envelope).unwrap_or_default();
        if socket.send(Message::Text(payload)).await.is_err() {
            break;
        }
    }
}
