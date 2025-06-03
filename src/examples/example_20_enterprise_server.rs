// Example 20: Enterprise Server Implementation
//
// This example demonstrates how to build a comprehensive enterprise server
// that combines authentication, monitoring, caching, HTTP endpoints, and
// proper error handling in a production-ready application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

// Struct: User
//
// Represents a user in the enterprise system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    username: String,
    email: String,
    role: UserRole,

    created_at: DateTime<Utc>,
    last_active: DateTime<Utc>,
}

// Enum: UserRole
//
// Defines user roles in the enterprise system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Manager,
    Employee,
    Guest,
}

// Struct: Session
//
// Represents an authenticated user session.
#[derive(Debug, Clone)]
pub struct Session {
    id: Uuid,
    user_id: Uuid,
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
}

// Struct: CacheEntry
//
// Represents a cached value with expiration.
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    value: T,
    expires_at: DateTime<Utc>,
}

// Struct: Cache
//
// Simple in-memory cache with TTL support.
pub struct Cache<T: Clone> {
    entries: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
}

impl<T: Clone> Default for Cache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Cache<T> {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get(key) {
            if entry.expires_at > Utc::now() {
                return Some(entry.value.clone());
            } else {
                entries.remove(key);
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: T, ttl_seconds: i64) {
        let entry = CacheEntry {
            value,
            expires_at: Utc::now() + chrono::Duration::seconds(ttl_seconds),
        };

        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
    }

    pub async fn remove(&self, key: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
    }

    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        let now = Utc::now();
        entries.retain(|_, entry| entry.expires_at > now);
    }
}

// Struct: Metrics
//
// Tracks various server metrics.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Metrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    average_response_time_ms: f64,
    active_sessions: u64,
    cache_hits: u64,
    cache_misses: u64,
}

// Struct: ApiRequest
//
// Represents an API request to the server.
#[derive(Debug, Clone)]
pub struct ApiRequest {
    #[allow(dead_code)]
    id: Uuid,
    method: String,
    path: String,
    headers: HashMap<String, String>,
    #[allow(dead_code)]
    body: Option<String>,
    user_id: Option<Uuid>,
    #[allow(dead_code)]
    timestamp: DateTime<Utc>,
}

impl ApiRequest {
    pub fn new(method: String, path: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            method,
            path,
            headers: HashMap::new(),
            body: None,
            user_id: None,
            timestamp: Utc::now(),
        }
    }
}

// Struct: ApiResponse
//
// Represents an API response from the server.
#[derive(Debug, Clone)]
pub struct ApiResponse {
    status_code: u16,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
    #[allow(dead_code)]
    body: String,
    processing_time_ms: u64,
}

impl ApiResponse {
    pub fn success(body: String, processing_time_ms: u64) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Self {
            status_code: 200,
            headers,
            body,
            processing_time_ms,
        }
    }

    pub fn error(status_code: u16, message: String, processing_time_ms: u64) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Self {
            status_code,
            headers,
            body: format!(r#"{{"error": "{}"}}"#, message),
            processing_time_ms,
        }
    }
}

// Struct: EnterpriseServer
//
// Main enterprise server that combines all components.
pub struct EnterpriseServer {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    user_cache: Cache<User>,
    #[allow(dead_code)]
    data_cache: Cache<String>,
    metrics: Arc<RwLock<Metrics>>,
}

impl Default for EnterpriseServer {
    fn default() -> Self {
        Self::new()
    }
}

impl EnterpriseServer {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_cache: Cache::new(),
            data_cache: Cache::new(),
            metrics: Arc::new(RwLock::new(Metrics::default())),
        }
    }

    // Authentication methods
    pub async fn create_user(
        &self,
        username: String,
        email: String,
        role: UserRole,
    ) -> Result<Uuid, String> {
        let user = User {
            id: Uuid::new_v4(),
            username: username.clone(),
            email,
            role,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let user_id = user.id;
        let mut users = self.users.write().await;

        // Check if username already exists
        if users.values().any(|u| u.username == username) {
            return Err("Username already exists".to_string());
        }

        users.insert(user_id, user.clone());

        // Cache the user
        self.user_cache.set(user_id.to_string(), user, 3600).await;

        info!("Created user: {} ({})", username, user_id);
        Ok(user_id)
    }

    pub async fn create_session(&self, user_id: Uuid) -> Result<Uuid, String> {
        // Verify user exists
        if !self.users.read().await.contains_key(&user_id) {
            return Err("User not found".to_string());
        }

        let session = Session {
            id: Uuid::new_v4(),
            user_id,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(8),
            last_accessed: Utc::now(),
        };

        let session_id = session.id;
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.active_sessions += 1;

        info!("Created session: {} for user: {}", session_id, user_id);
        Ok(session_id)
    }

    pub async fn validate_session(&self, session_id: Uuid) -> Option<Uuid> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(&session_id) {
            if session.expires_at > Utc::now() {
                session.last_accessed = Utc::now();
                return Some(session.user_id);
            } else {
                sessions.remove(&session_id);

                // Update metrics
                let mut metrics = self.metrics.write().await;
                metrics.active_sessions = metrics.active_sessions.saturating_sub(1);
            }
        }
        None
    }

    // API endpoints
    pub async fn handle_request(&self, mut request: ApiRequest) -> ApiResponse {
        let start_time = std::time::Instant::now();

        // Extract session token from headers
        if let Some(auth_header) = request.headers.get("Authorization") {
            if let Some(session_id_str) = auth_header.strip_prefix("Bearer ") {
                if let Ok(session_id) = Uuid::parse_str(session_id_str) {
                    request.user_id = self.validate_session(session_id).await;
                }
            }
        }

        let response = match request.path.as_str() {
            "/api/health" => self.handle_health_check().await,
            "/api/users/profile" => self.handle_user_profile(&request).await,
            "/api/data" => self.handle_data_request(&request).await,
            "/api/metrics" => self.handle_metrics_request(&request).await,
            _ => ApiResponse::error(404, "Not Found".to_string(), 0),
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Update metrics
        self.update_metrics(&response, processing_time).await;

        info!(
            "Request {} {} -> {} ({}ms)",
            request.method, request.path, response.status_code, processing_time
        );

        ApiResponse {
            processing_time_ms: processing_time,
            ..response
        }
    }

    async fn handle_health_check(&self) -> ApiResponse {
        let health_data = serde_json::json!({
            "status": "healthy",
            "timestamp": Utc::now().to_rfc3339(),
            "version": "1.0.0"
        });

        ApiResponse::success(health_data.to_string(), 0)
    }

    async fn handle_user_profile(&self, request: &ApiRequest) -> ApiResponse {
        let user_id = match request.user_id {
            Some(id) => id,
            None => return ApiResponse::error(401, "Unauthorized".to_string(), 0),
        };

        // Try cache first
        if let Some(user) = self.user_cache.get(&user_id.to_string()).await {
            let mut metrics = self.metrics.write().await;
            metrics.cache_hits += 1;

            return ApiResponse::success(serde_json::to_string(&user).unwrap(), 0);
        }

        // Cache miss - fetch from database
        let users = self.users.read().await;
        if let Some(user) = users.get(&user_id) {
            let mut metrics = self.metrics.write().await;
            metrics.cache_misses += 1;

            // Cache for future requests
            self.user_cache
                .set(user_id.to_string(), user.clone(), 3600)
                .await;

            ApiResponse::success(serde_json::to_string(user).unwrap(), 0)
        } else {
            ApiResponse::error(404, "User not found".to_string(), 0)
        }
    }

    async fn handle_data_request(&self, request: &ApiRequest) -> ApiResponse {
        if request.user_id.is_none() {
            return ApiResponse::error(401, "Unauthorized".to_string(), 0);
        }

        // Simulate data processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let data = serde_json::json!({
            "data": "Sample enterprise data",
            "timestamp": Utc::now().to_rfc3339(),
            "user_id": request.user_id
        });

        ApiResponse::success(data.to_string(), 0)
    }

    async fn handle_metrics_request(&self, request: &ApiRequest) -> ApiResponse {
        // Check if user is admin
        if let Some(user_id) = request.user_id {
            let users = self.users.read().await;
            if let Some(user) = users.get(&user_id) {
                if user.role != UserRole::Admin {
                    return ApiResponse::error(403, "Forbidden".to_string(), 0);
                }
            } else {
                return ApiResponse::error(404, "User not found".to_string(), 0);
            }
        } else {
            return ApiResponse::error(401, "Unauthorized".to_string(), 0);
        }

        let metrics = self.metrics.read().await;
        ApiResponse::success(serde_json::to_string(&*metrics).unwrap(), 0)
    }

    async fn update_metrics(&self, response: &ApiResponse, processing_time: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        if response.status_code < 400 {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // Update average response time
        let total_time = metrics.average_response_time_ms * (metrics.total_requests - 1) as f64
            + processing_time as f64;
        metrics.average_response_time_ms = total_time / metrics.total_requests as f64;
    }

    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.read().await.clone()
    }

    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let initial_count = sessions.len();

        sessions.retain(|_, session| session.expires_at > now);

        let removed_count = initial_count - sessions.len();
        if removed_count > 0 {
            let mut metrics = self.metrics.write().await;
            metrics.active_sessions = sessions.len() as u64;
            info!("Cleaned up {} expired sessions", removed_count);
        }
    }
}

// Function: demo_enterprise_server
//
// Demonstrates the enterprise server functionality.
async fn demo_enterprise_server() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Creating Enterprise Server ===");
    let server = EnterpriseServer::new();

    // Create users
    let admin_id = server
        .create_user(
            "admin".to_string(),
            "admin@company.com".to_string(),
            UserRole::Admin,
        )
        .await?;

    let employee_id = server
        .create_user(
            "john_doe".to_string(),
            "john@company.com".to_string(),
            UserRole::Employee,
        )
        .await?;

    // Create sessions
    let admin_session = server.create_session(admin_id).await?;
    let employee_session = server.create_session(employee_id).await?;

    info!("=== Processing API Requests ===");

    // Test various endpoints
    let requests = vec![
        ApiRequest::new("GET".to_string(), "/api/health".to_string()),
        {
            let mut req = ApiRequest::new("GET".to_string(), "/api/users/profile".to_string());
            req.headers.insert(
                "Authorization".to_string(),
                format!("Bearer {}", employee_session),
            );
            req
        },
        {
            let mut req = ApiRequest::new("GET".to_string(), "/api/data".to_string());
            req.headers.insert(
                "Authorization".to_string(),
                format!("Bearer {}", employee_session),
            );
            req
        },
        {
            let mut req = ApiRequest::new("GET".to_string(), "/api/metrics".to_string());
            req.headers.insert(
                "Authorization".to_string(),
                format!("Bearer {}", admin_session),
            );
            req
        },
        ApiRequest::new("GET".to_string(), "/api/nonexistent".to_string()),
    ];

    for request in requests {
        let response = server.handle_request(request.clone()).await;
        info!(
            "Response {}: {} ({}ms)",
            request.path, response.status_code, response.processing_time_ms
        );
    }

    // Cleanup and show final metrics
    server.user_cache.cleanup_expired().await;
    server.cleanup_expired_sessions().await;

    let final_metrics = server.get_metrics().await;
    info!("=== Final Server Metrics ===");
    info!("Total requests: {}", final_metrics.total_requests);
    info!(
        "Success rate: {:.1}%",
        (final_metrics.successful_requests as f64 / final_metrics.total_requests as f64) * 100.0
    );
    info!(
        "Average response time: {:.2}ms",
        final_metrics.average_response_time_ms
    );
    info!("Active sessions: {}", final_metrics.active_sessions);
    info!(
        "Cache hit rate: {:.1}%",
        (final_metrics.cache_hits as f64
            / (final_metrics.cache_hits + final_metrics.cache_misses) as f64)
            * 100.0
    );

    Ok(())
}

// Function: main
//
// Entry point demonstrating the enterprise server implementation.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Enterprise Server Example");
    demo_enterprise_server().await?;
    info!("Enterprise Server Example completed successfully");

    Ok(())
}
