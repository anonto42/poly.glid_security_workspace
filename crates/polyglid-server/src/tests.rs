#[cfg(test)]
mod integration_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;
    use std::sync::Arc;
    use std::fs;
    use polyglid_runtime::WasmRuntime;
    use polyglid_core::{
        execution::ExecutionManager,
        plugin_manager::PluginManager,
        store::WorkspaceStore,
        services::{PluginService, ExecutionService, TargetService, ReportService, SettingsService},
    };
    use crate::{auth, ServerState};

    fn setup_test_app() -> (Router, String) {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("polyglid_server_test.db");
        if db_path.exists() {
            let _ = fs::remove_file(&db_path);
        }

        let config = polyglid_config::AppConfig::development();
        let runtime = std::sync::Arc::new(WasmRuntime::new());
        let store = WorkspaceStore::new(&db_path).unwrap();
        let pm = Arc::new(PluginManager::new(runtime.clone(), &config, store.clone()).unwrap());
        let em = Arc::new(ExecutionManager::new(WasmRuntime::new(), Some(store.clone())));

        let admin_token = auth::initialize_auth_token(&store).unwrap();

        let server_state = ServerState {
            plugin_service: Arc::new(PluginService::new(pm.clone())),
            execution_service: Arc::new(ExecutionService::new(em.clone(), store.clone())),
            target_service: Arc::new(TargetService::new(store.clone())),
            report_service: Arc::new(ReportService::new(store.clone())),
            settings_service: Arc::new(SettingsService::new(store.clone())),
            execution_manager: em.clone(),
        };

        let auth_state = auth::AuthState {
            expected_token: admin_token.clone(),
        };

        let api_routes = Router::new()
            .route("/plugins", axum::routing::get(crate::get_plugins).post(crate::install_plugin))
            .route("/targets", axum::routing::get(crate::list_targets).post(crate::add_target))
            .layer(axum::middleware::from_fn_with_state(
                auth_state,
                auth::auth_middleware,
            ))
            .with_state(server_state);

        let app = Router::new().nest("/api/v1", api_routes);
        (app, admin_token)
    }

    #[tokio::test]
    async fn test_auth_middleware_blocks_unauthorized() {
        let (app, _) = setup_test_app();

        let req = Request::builder()
            .uri("/api/v1/plugins")
            .header("Authorization", "Bearer wrong_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_permits_authorized() {
        let (app, token) = setup_test_app();

        let req = Request::builder()
            .uri("/api/v1/plugins")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_targets_api_flow() {
        let (app, token) = setup_test_app();

        // Add target
        let add_req = Request::builder()
            .method("POST")
            .uri("/api/v1/targets")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name": "test.target.com"}"#))
            .unwrap();

        let add_res = app.clone().oneshot(add_req).await.unwrap();
        assert_eq!(add_res.status(), StatusCode::CREATED);

        // List targets
        let list_req = Request::builder()
            .uri("/api/v1/targets")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let list_res = app.oneshot(list_req).await.unwrap();
        assert_eq!(list_res.status(), StatusCode::OK);
    }
}
