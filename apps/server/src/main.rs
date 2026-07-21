use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use polyglid_config::AppConfig;
use polyglid_core::{
    execution::ExecutionManager,
    plugin_manager::PluginManager,
    services::{
        CollaborationService, ExecutionService, MarketplaceService, PluginService, ReportService,
        SettingsService, TargetService,
    },
    store::collaboration_store::{DbTeam, DbUser},
    store::marketplace_store::{DbMarketplacePackage, DbMarketplaceRating, DbPublisherProfile},
    store::WorkspaceStore,
};
use polyglid_plugin_api::PluginId;
use polyglid_runtime::WasmRuntime;

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
    marketplace_service: Arc<MarketplaceService>,
    collaboration_service: Arc<CollaborationService>,
    execution_manager: Arc<ExecutionManager<WasmRuntime>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load_from_env().unwrap_or_else(|_| AppConfig::development());
    let runtime = std::sync::Arc::new(WasmRuntime::new());
    let db_path = config
        .plugin_dir
        .parent()
        .unwrap_or(&config.plugin_dir)
        .join("polyglid.db");

    let store = WorkspaceStore::new(&db_path)?;
    let pm = Arc::new(PluginManager::new(runtime.clone(), &config, store.clone())?);
    let em = Arc::new(ExecutionManager::new(
        WasmRuntime::new(),
        Some(store.clone()),
    ));

    // Sync plugins
    let _ = pm.sync_directory();

    let admin_token = auth::initialize_auth_token(&store)?;
    let col_service = Arc::new(CollaborationService::new(store.clone()));

    let server_state = ServerState {
        plugin_service: Arc::new(PluginService::new(pm.clone())),
        execution_service: Arc::new(ExecutionService::new(em.clone(), store.clone())),
        target_service: Arc::new(TargetService::new(store.clone())),
        report_service: Arc::new(ReportService::new(store.clone())),
        settings_service: Arc::new(SettingsService::new(store.clone())),
        marketplace_service: Arc::new(MarketplaceService::new(store.clone())),
        collaboration_service: col_service.clone(),
        execution_manager: em.clone(),
    };

    let auth_state = auth::AuthState {
        expected_token: admin_token,
        collaboration_service: col_service,
    };

    let public_routes = Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user));

    let protected_routes = Router::new()
        // Auth / Users
        .route("/auth/me", get(get_current_user))
        .route("/auth/users", get(list_users))
        // Teams
        .route("/teams", get(list_teams).post(create_team))
        .route(
            "/teams/:id/members",
            get(list_team_members).post(add_team_member),
        )
        .route("/teams/:id/members/:user_id", delete(remove_team_member))
        // Plugins
        .route("/plugins", get(get_plugins).post(install_plugin))
        .route("/plugins/:id", delete(uninstall_plugin))
        .route("/plugins/:id/toggle", post(toggle_plugin))
        .route("/plugins/:id/configure", post(configure_plugin))
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
        // Marketplace
        .route("/marketplace", get(marketplace_list))
        .route("/marketplace/search", get(marketplace_search))
        .route("/marketplace/packages/:id", get(marketplace_get_package))
        .route(
            "/marketplace/packages/:id/ratings",
            get(marketplace_list_ratings).post(marketplace_add_rating),
        )
        .route(
            "/marketplace/packages/:id/install",
            post(marketplace_install),
        )
        .route("/marketplace/publish", post(marketplace_publish))
        .route(
            "/marketplace/publishers",
            get(marketplace_list_publishers).post(marketplace_register_publisher),
        )
        .layer(middleware::from_fn_with_state(
            auth_state,
            auth::auth_middleware,
        ));

    let api_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
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
) -> Result<Json<Vec<polyglid_config::plugin_registry::PluginRegistryEntry>>, (StatusCode, String)>
{
    state
        .plugin_service
        .list_plugins()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

#[derive(serde::Deserialize)]
struct InstallRequest {
    path: String,
}

async fn install_plugin(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    Json(req): Json<InstallRequest>,
) -> Result<Json<polyglid_config::plugin_registry::PluginRegistryEntry>, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can install plugins".to_string(),
        ));
    }
    state
        .plugin_service
        .install_plugin(Path::new(&req.path))
        .map(Json)
        .map_err(|err| (StatusCode::BAD_REQUEST, err))
}

async fn uninstall_plugin(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can uninstall plugins".to_string(),
        ));
    }
    let pid = PluginId::new(&id).map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .plugin_service
        .uninstall_plugin(&pid)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

#[derive(serde::Deserialize)]
struct ToggleRequest {
    enabled: bool,
}

async fn toggle_plugin(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
    Json(req): Json<ToggleRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" && user.role != "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners or Editors can toggle plugins".to_string(),
        ));
    }
    let pid = PluginId::new(&id).map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .plugin_service
        .toggle_plugin(&pid, req.enabled)
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
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    Json(req): Json<RunExecutionRequest>,
) -> Result<(StatusCode, Json<RunExecutionResponse>), (StatusCode, String)> {
    if user.role != "Owner" && user.role != "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners or Editors can run scans".to_string(),
        ));
    }
    let pid =
        PluginId::new(&req.plugin_id).map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .execution_service
        .run_plugin(&pid, &req.target)
        .map(|job_id| (StatusCode::ACCEPTED, Json(RunExecutionResponse { job_id })))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn list_executions(
    State(state): State<ServerState>,
) -> Result<Json<Vec<polyglid_core::store::execution_store::DbJobRecord>>, (StatusCode, String)> {
    state
        .execution_service
        .list_executions()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn get_execution(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<polyglid_core::store::execution_store::DbJobRecord>, (StatusCode, String)> {
    state
        .execution_service
        .get_execution(&id)
        .and_then(|opt| opt.ok_or_else(|| "Execution not found".to_string()))
        .map(Json)
        .map_err(|err| (StatusCode::NOT_FOUND, err))
}

async fn list_targets(
    State(state): State<ServerState>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    state
        .target_service
        .list_targets()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

#[derive(serde::Deserialize)]
struct AddTargetRequest {
    name: String,
}

async fn add_target(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    Json(req): Json<AddTargetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" && user.role != "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners or Editors can add targets".to_string(),
        ));
    }
    state
        .target_service
        .add_target(&req.name)
        .map(|_| StatusCode::CREATED)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn remove_target(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath(name): AxumPath<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can remove targets".to_string(),
        ));
    }
    state
        .target_service
        .remove_target(&name)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn list_reports(
    State(state): State<ServerState>,
) -> Result<Json<Vec<polyglid_core::store::report_store::DbReportRecord>>, (StatusCode, String)> {
    state
        .report_service
        .list_reports()
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn get_report(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<polyglid_core::store::report_store::DbReportRecord>, (StatusCode, String)> {
    state
        .report_service
        .get_report(&id)
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

    let content = state
        .report_service
        .export_report(&id, &format)
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
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"report-{}.{}\"", id, format),
        )
        .body(axum::body::Body::from(content))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(response)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> impl IntoResponse {
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

async fn configure_plugin(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
    Json(req): Json<std::collections::HashMap<String, String>>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" && user.role != "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners or Editors can configure plugins".to_string(),
        ));
    }
    let pid = PluginId::new(&id).map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    for (k, v) in req {
        let setting_key = format!("plugin:{}:{}", pid.as_str(), k);
        state
            .settings_service
            .set_setting(&setting_key, &v)
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    }
    Ok(StatusCode::OK)
}
// ─────────────────────────────────────────────────────────────────────────────
// Marketplace handlers
// ─────────────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct MarketplaceSearchQuery {
    q: Option<String>,
    category: Option<String>,
}

async fn marketplace_list(
    State(state): State<ServerState>,
) -> Result<Json<Vec<DbMarketplacePackage>>, (StatusCode, String)> {
    state
        .marketplace_service
        .list_featured()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn marketplace_search(
    State(state): State<ServerState>,
    Query(params): Query<MarketplaceSearchQuery>,
) -> Result<Json<Vec<DbMarketplacePackage>>, (StatusCode, String)> {
    let query = params.q.as_deref().unwrap_or("");
    let category = params.category.as_deref();
    state
        .marketplace_service
        .search(query, category)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn marketplace_get_package(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<DbMarketplacePackage>, (StatusCode, String)> {
    state
        .marketplace_service
        .get_package(&id)
        .and_then(|opt| opt.ok_or_else(|| "Package not found".to_string()))
        .map(Json)
        .map_err(|e| (StatusCode::NOT_FOUND, e))
}

async fn marketplace_publish(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    Json(pkg): Json<DbMarketplacePackage>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can publish marketplace packages".to_string(),
        ));
    }
    state
        .marketplace_service
        .publish(&pkg)
        .map(|_| StatusCode::CREATED)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

#[derive(serde::Deserialize)]
struct MarketplaceInstallRequest {
    plugin_id: Option<String>,
}

async fn marketplace_install(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
    Json(req): Json<MarketplaceInstallRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can install marketplace packages".to_string(),
        ));
    }
    // Get the package's download_url so the caller can install via PluginService
    let _url = state
        .marketplace_service
        .get_package_download_url(&id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    // Record the install tracking
    state
        .marketplace_service
        .record_package_install(&id, req.plugin_id)
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn marketplace_list_ratings(
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<Vec<DbMarketplaceRating>>, (StatusCode, String)> {
    state
        .marketplace_service
        .list_ratings(&id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn marketplace_add_rating(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath(id): AxumPath<String>,
    Json(mut rating): Json<DbMarketplaceRating>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" && user.role != "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners or Editors can add ratings".to_string(),
        ));
    }
    rating.package_id = id;
    if rating.id.is_empty() {
        rating.id = format!("r-{}", uuid_hex());
    }
    state
        .marketplace_service
        .add_rating(&rating)
        .map(|_| StatusCode::CREATED)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn marketplace_list_publishers(
    State(state): State<ServerState>,
) -> Result<Json<Vec<DbPublisherProfile>>, (StatusCode, String)> {
    state
        .marketplace_service
        .list_publishers()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn marketplace_register_publisher(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    Json(mut profile): Json<DbPublisherProfile>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can register publishers".to_string(),
        ));
    }
    if profile.id.is_empty() {
        profile.id = format!("pub-{}", uuid_hex());
    }
    state
        .marketplace_service
        .register_publisher(&profile)
        .map(|_| StatusCode::CREATED)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

fn uuid_hex() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::Instant::now().hash(&mut h);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
        .hash(&mut h);
    format!("{:016x}", h.finish())
}

// ─────────────────────────────────────────────────────────────────────────────
// Collaboration & Authentication Handlers
// ─────────────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    role: String,
}

#[derive(serde::Serialize)]
struct AuthResponse {
    token: String,
    user: DbUser,
}

async fn register_user(
    State(state): State<ServerState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<DbUser>), (StatusCode, String)> {
    let count = state
        .collaboration_service
        .count_users()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let final_role = if count == 0 {
        "Owner".to_string()
    } else if req.role == "Owner" || req.role == "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owner can register Owner or Editor accounts".to_string(),
        ));
    } else {
        "Viewer".to_string()
    };

    let user = state
        .collaboration_service
        .register_user(&req.username, &req.password, &final_role)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login_user(
    State(state): State<ServerState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let (user, token) = state
        .collaboration_service
        .login_user(&req.username, &req.password)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;
    Ok(Json(AuthResponse { token, user }))
}

async fn get_current_user(axum::Extension(user): axum::Extension<DbUser>) -> Json<DbUser> {
    Json(user)
}

async fn list_users(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
) -> Result<Json<Vec<DbUser>>, (StatusCode, String)> {
    if user.role != "Owner" && user.role != "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners or Editors can list users".to_string(),
        ));
    }
    state
        .collaboration_service
        .list_users()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// Teams

#[derive(serde::Deserialize)]
struct CreateTeamRequest {
    name: String,
}

async fn create_team(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<DbTeam>), (StatusCode, String)> {
    if user.role != "Owner" && user.role != "Editor" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners or Editors can create teams".to_string(),
        ));
    }
    let team = state
        .collaboration_service
        .create_team(&req.name)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok((StatusCode::CREATED, Json(team)))
}

async fn list_teams(
    State(state): State<ServerState>,
) -> Result<Json<Vec<DbTeam>>, (StatusCode, String)> {
    state
        .collaboration_service
        .list_teams()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(serde::Deserialize)]
struct AddMemberRequest {
    user_id: String,
    role: String,
}

async fn add_team_member(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath(team_id): AxumPath<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can manage team memberships".to_string(),
        ));
    }
    state
        .collaboration_service
        .add_team_member(&team_id, &req.user_id, &req.role)
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn remove_team_member(
    axum::Extension(user): axum::Extension<DbUser>,
    State(state): State<ServerState>,
    AxumPath((team_id, user_id)): AxumPath<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    if user.role != "Owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Owners can manage team memberships".to_string(),
        ));
    }
    state
        .collaboration_service
        .remove_team_member(&team_id, &user_id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn list_team_members(
    State(state): State<ServerState>,
    AxumPath(team_id): AxumPath<String>,
) -> Result<Json<Vec<(DbUser, String)>>, (StatusCode, String)> {
    state
        .collaboration_service
        .list_team_members(&team_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
