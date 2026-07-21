#[cfg(test)]
mod integration_tests {
    use crate::{auth, ServerState};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use polyglid_core::{
        execution::ExecutionManager,
        plugin_manager::PluginManager,
        services::{
            CollaborationService, ExecutionService, MarketplaceService, PluginService,
            ReportService, SettingsService, TargetService,
        },
        store::WorkspaceStore,
    };
    use polyglid_runtime::WasmRuntime;
    use std::path::Path;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn setup_test_app() -> (Router, String) {
        let config = polyglid_config::AppConfig::development();
        let runtime = std::sync::Arc::new(WasmRuntime::new());
        let store = WorkspaceStore::new(Path::new(":memory:")).unwrap();
        let pm = Arc::new(PluginManager::new(runtime.clone(), &config, store.clone()).unwrap());
        let em = Arc::new(ExecutionManager::new(
            WasmRuntime::new(),
            Some(store.clone()),
        ));

        let admin_token = auth::initialize_auth_token(&store).unwrap();
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
            expected_token: admin_token.clone(),
            collaboration_service: col_service,
        };

        let public_routes = Router::new()
            .route("/auth/register", axum::routing::post(crate::register_user))
            .route("/auth/login", axum::routing::post(crate::login_user));

        let protected_routes = Router::new()
            .route(
                "/plugins",
                axum::routing::get(crate::get_plugins).post(crate::install_plugin),
            )
            .route(
                "/targets",
                axum::routing::get(crate::list_targets).post(crate::add_target),
            )
            .layer(axum::middleware::from_fn_with_state(
                auth_state,
                auth::auth_middleware,
            ));

        let api_routes = Router::new()
            .merge(public_routes)
            .merge(protected_routes)
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

    #[tokio::test]
    async fn test_user_registration_login_and_role_access_control() {
        let (app, _) = setup_test_app();

        // 1. Register first user (automatically becomes Owner)
        let reg1_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"username": "owner_alice", "password": "password123", "role": "Owner"}"#,
            ))
            .unwrap();
        let reg1_res = app.clone().oneshot(reg1_req).await.unwrap();
        assert_eq!(reg1_res.status(), StatusCode::CREATED);

        // 2. Register second user (since count > 0, it becomes Viewer)
        let reg2_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"username": "viewer_bob", "password": "password123", "role": "Viewer"}"#,
            ))
            .unwrap();
        let reg2_res = app.clone().oneshot(reg2_req).await.unwrap();
        assert_eq!(reg2_res.status(), StatusCode::CREATED);

        // 3. Login as Viewer
        let login_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"username": "viewer_bob", "password": "password123"}"#,
            ))
            .unwrap();
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        assert_eq!(login_res.status(), StatusCode::OK);

        // Parse token
        let body_bytes = axum::body::to_bytes(login_res.into_body(), 10000)
            .await
            .unwrap();
        let auth_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let token = auth_res["token"].as_str().unwrap();

        // 4. Try to add target as Viewer (should fail with 403 Forbidden)
        let fail_req = Request::builder()
            .method("POST")
            .uri("/api/v1/targets")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name": "denied.target.com"}"#))
            .unwrap();
        let fail_res = app.oneshot(fail_req).await.unwrap();
        assert_eq!(fail_res.status(), StatusCode::FORBIDDEN);
    }
}
