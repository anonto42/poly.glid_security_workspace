use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use polyglid_core::store::collaboration_store::DbUser;
use polyglid_core::store::WorkspaceStore;

pub fn initialize_auth_token(store: &WorkspaceStore) -> Result<String, String> {
    if let Some(token) = store.settings().get("api_admin_token").unwrap_or(None) {
        Ok(token)
    } else {
        // Generate secure random API key using Uuid bytes
        let uuid_val = uuid::Uuid::new_v4();
        let token = hex::encode(uuid_val.as_bytes());

        store
            .settings()
            .set("api_admin_token", &token, "Workspace")?;

        println!("============================================================");
        println!("  INITIALIZATION SUCCESS: PolyGlid Server Admin Token Generated!");
        println!("  Bearer Token: {}", token);
        println!("  Save this token to configure requests to /api/v1/ endpoints.");
        println!("============================================================");

        Ok(token)
    }
}

#[derive(Clone)]
pub struct AuthState {
    pub expected_token: String,
    pub collaboration_service: std::sync::Arc<polyglid_core::services::CollaborationService>,
}

pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    if let Some(auth_header) = req.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // 1. Check if token is the bootstrap admin token
                if token == state.expected_token {
                    let admin_user = DbUser {
                        id: "bootstrap-admin".to_string(),
                        username: "admin".to_string(),
                        password_hash: "".to_string(),
                        salt: "".to_string(),
                        role: "Owner".to_string(),
                        created_at: 0,
                        updated_at: 0,
                    };
                    req.extensions_mut().insert(admin_user);
                    return Ok(next.run(req).await);
                }

                // 2. Validate token against database
                if let Ok(Some(user)) = state.collaboration_service.validate_token(token) {
                    req.extensions_mut().insert(user);
                    return Ok(next.run(req).await);
                }
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
