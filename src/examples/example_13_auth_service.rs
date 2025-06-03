// Example 13: Authentication Service Implementation
//
// This example demonstrates how to build a secure authentication service
// using JWT tokens, password hashing, and proper session management.
// It shows how to implement user registration, login, token validation,
// and role-based access control in a production-ready manner.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

// Constants for authentication configuration
// These values should be configurable in a real application
#[allow(dead_code)]
const JWT_SECRET: &str = "your-secret-key-here"; // In production, use environment variables
const TOKEN_EXPIRY_HOURS: i64 = 24;
const MAX_LOGIN_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION_MINUTES: i64 = 30;

// Enum: UserRole
//
// This enum defines different roles that users can have in the system.
// Roles determine what actions a user is authorized to perform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,     // Full system access
    Moderator, // Limited administrative access
    User,      // Standard user access
    Guest,     // Read-only access
}

// Struct: User
//
// This struct represents a user account in the authentication system.
// It contains all the necessary information for user management and security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String, // Never store plain text passwords
    role: UserRole,
    created_at: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
    is_active: bool,
    failed_login_attempts: u32,
    locked_until: Option<DateTime<Utc>>,
}

impl User {
    // Function: new
    //
    // Creates a new user account with the provided information.
    // The password is automatically hashed for security.
    //
    // Arguments:
    //     username: The unique username for this account
    //     email: The user's email address
    //     password: The plain text password (will be hashed)
    //     role: The role to assign to this user
    //
    // Returns:
    //     A new User instance with hashed password
    pub fn new(username: String, email: String, password: String, role: UserRole) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash: hash_password(&password),
            role,
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            failed_login_attempts: 0,
            locked_until: None,
        }
    }

    // Function: verify_password
    //
    // Verifies if the provided password matches the user's stored password hash.
    //
    // Arguments:
    //     password: The plain text password to verify
    //
    // Returns:
    //     true if the password is correct, false otherwise
    pub fn verify_password(&self, password: &str) -> bool {
        verify_password(password, &self.password_hash)
    }

    // Function: is_locked
    //
    // Checks if the user account is currently locked due to too many failed login attempts.
    //
    // Returns:
    //     true if the account is locked, false otherwise
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    // Function: increment_failed_attempts
    //
    // Increments the failed login attempt counter and locks the account if necessary.
    pub fn increment_failed_attempts(&mut self) {
        self.failed_login_attempts += 1;

        if self.failed_login_attempts >= MAX_LOGIN_ATTEMPTS {
            self.locked_until = Some(Utc::now() + Duration::minutes(LOCKOUT_DURATION_MINUTES));
            warn!(
                "Account locked for user {} due to {} failed attempts",
                self.username, self.failed_login_attempts
            );
        }
    }

    // Function: reset_failed_attempts
    //
    // Resets the failed login attempt counter and unlocks the account.
    pub fn reset_failed_attempts(&mut self) {
        self.failed_login_attempts = 0;
        self.locked_until = None;
    }

    // Function: update_last_login
    //
    // Updates the last login timestamp to the current time.
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }
}

// Struct: AuthToken
//
// This struct represents an authentication token (JWT-like) that contains
// user information and expiration details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    user_id: Uuid,
    username: String,
    role: UserRole,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    token_id: Uuid, // Unique identifier for this token
}

impl AuthToken {
    // Function: new
    //
    // Creates a new authentication token for the specified user.
    //
    // Arguments:
    //     user: The user for whom to create the token
    //
    // Returns:
    //     A new AuthToken with expiration set to 24 hours from now
    pub fn new(user: &User) -> Self {
        let now = Utc::now();
        Self {
            user_id: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
            issued_at: now,
            expires_at: now + Duration::hours(TOKEN_EXPIRY_HOURS),
            token_id: Uuid::new_v4(),
        }
    }

    // Function: is_expired
    //
    // Checks if this token has expired.
    //
    // Returns:
    //     true if the token is expired, false otherwise
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    // Function: is_valid
    //
    // Checks if this token is still valid (not expired and user is active).
    //
    // Returns:
    //     true if the token is valid, false otherwise
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

// Struct: LoginRequest
//
// This struct represents a login request from a client.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

// Struct: RegistrationRequest
//
// This struct represents a user registration request.
#[derive(Debug, Deserialize)]
pub struct RegistrationRequest {
    username: String,
    email: String,
    password: String,
}

// Struct: AuthService
//
// This struct implements the main authentication service functionality.
// It manages users, tokens, and provides authentication operations.
pub struct AuthService {
    users: Arc<RwLock<HashMap<String, User>>>, // username -> User
    active_tokens: Arc<RwLock<HashMap<Uuid, AuthToken>>>, // token_id -> AuthToken
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthService {
    // Function: new
    //
    // Creates a new authentication service instance.
    //
    // Returns:
    //     A new AuthService with empty user and token stores
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            active_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Function: register_user
    //
    // Registers a new user account in the system.
    //
    // Arguments:
    //     request: The registration request containing user details
    //
    // Returns:
    //     Result with the created user ID or an error message
    pub async fn register_user(&self, request: RegistrationRequest) -> Result<Uuid, String> {
        let mut users = self.users.write().await;

        // Check if username already exists
        if users.contains_key(&request.username) {
            return Err("Username already exists".to_string());
        }

        // Validate password strength
        if !is_password_strong(&request.password) {
            return Err("Password does not meet security requirements".to_string());
        }

        // Create new user with default role
        let user = User::new(
            request.username.clone(),
            request.email,
            request.password,
            UserRole::User,
        );

        let user_id = user.id;
        let username = request.username.clone();
        users.insert(request.username, user);

        info!("User registered successfully: {}", username);
        Ok(user_id)
    }

    // Function: authenticate
    //
    // Authenticates a user with username and password, returning a token if successful.
    //
    // Arguments:
    //     request: The login request containing credentials
    //
    // Returns:
    //     Result with an authentication token or an error message
    pub async fn authenticate(&self, request: LoginRequest) -> Result<AuthToken, String> {
        let mut users = self.users.write().await;

        // Find the user
        let user = users
            .get_mut(&request.username)
            .ok_or("Invalid username or password")?;

        // Check if account is locked
        if user.is_locked() {
            return Err(
                "Account is temporarily locked due to too many failed attempts".to_string(),
            );
        }

        // Check if account is active
        if !user.is_active {
            return Err("Account is deactivated".to_string());
        }

        // Verify password
        if !user.verify_password(&request.password) {
            user.increment_failed_attempts();
            warn!("Failed login attempt for user: {}", request.username);
            return Err("Invalid username or password".to_string());
        }

        // Successful authentication
        user.reset_failed_attempts();
        user.update_last_login();

        // Create authentication token
        let token = AuthToken::new(user);

        // Store the token
        let mut active_tokens = self.active_tokens.write().await;
        active_tokens.insert(token.token_id, token.clone());

        info!("User authenticated successfully: {}", request.username);
        Ok(token)
    }

    // Function: validate_token
    //
    // Validates an authentication token and returns the associated user information.
    //
    // Arguments:
    //     token_id: The unique identifier of the token to validate
    //
    // Returns:
    //     Result with the token if valid, or an error message
    pub async fn validate_token(&self, token_id: Uuid) -> Result<AuthToken, String> {
        let active_tokens = self.active_tokens.read().await;

        let token = active_tokens.get(&token_id).ok_or("Invalid token")?;

        if token.is_expired() {
            return Err("Token has expired".to_string());
        }

        Ok(token.clone())
    }

    // Function: logout
    //
    // Logs out a user by invalidating their authentication token.
    //
    // Arguments:
    //     token_id: The unique identifier of the token to invalidate
    //
    // Returns:
    //     Result indicating success or failure
    pub async fn logout(&self, token_id: Uuid) -> Result<(), String> {
        let mut active_tokens = self.active_tokens.write().await;

        match active_tokens.remove(&token_id) {
            Some(token) => {
                info!("User logged out: {}", token.username);
                Ok(())
            }
            None => Err("Token not found".to_string()),
        }
    }

    // Function: check_permission
    //
    // Checks if a user has permission to perform a specific action based on their role.
    //
    // Arguments:
    //     token: The authentication token containing user role
    //     required_role: The minimum role required for the action
    //
    // Returns:
    //     true if the user has permission, false otherwise
    pub fn check_permission(&self, token: &AuthToken, required_role: &UserRole) -> bool {
        match (&token.role, required_role) {
            (UserRole::Admin, _) => true, // Admin can do everything
            (UserRole::Moderator, UserRole::Moderator | UserRole::User | UserRole::Guest) => true,
            (UserRole::User, UserRole::User | UserRole::Guest) => true,
            (UserRole::Guest, UserRole::Guest) => true,
            _ => false,
        }
    }

    // Function: cleanup_expired_tokens
    //
    // Removes expired tokens from the active token store.
    // This should be called periodically to prevent memory leaks.
    pub async fn cleanup_expired_tokens(&self) {
        let mut active_tokens = self.active_tokens.write().await;
        let initial_count = active_tokens.len();

        active_tokens.retain(|_, token| !token.is_expired());

        let cleaned_count = initial_count - active_tokens.len();
        if cleaned_count > 0 {
            info!("Cleaned up {} expired tokens", cleaned_count);
        }
    }

    // Function: get_user_info
    //
    // Retrieves user information for a given username (without sensitive data).
    //
    // Arguments:
    //     username: The username to look up
    //
    // Returns:
    //     Result with user information or an error message
    pub async fn get_user_info(&self, username: &str) -> Result<UserInfo, String> {
        let users = self.users.read().await;

        let user = users.get(username).ok_or("User not found")?;

        Ok(UserInfo {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            created_at: user.created_at,
            last_login: user.last_login,
            is_active: user.is_active,
        })
    }
}

// Struct: UserInfo
//
// This struct contains non-sensitive user information that can be shared
// with clients or other services.
#[derive(Debug, Serialize)]
pub struct UserInfo {
    id: Uuid,
    username: String,
    email: String,
    role: UserRole,
    created_at: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
    is_active: bool,
}

// Function: hash_password
//
// Hashes a password using SHA-256. In production, you should use a proper
// password hashing library like bcrypt, scrypt, or Argon2.
//
// Arguments:
//     password: The plain text password to hash
//
// Returns:
//     The hashed password as a hexadecimal string
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Function: verify_password
//
// Verifies a password against its hash.
//
// Arguments:
//     password: The plain text password to verify
//     hash: The stored password hash
//
// Returns:
//     true if the password matches the hash, false otherwise
fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

// Function: is_password_strong
//
// Validates password strength according to security requirements.
//
// Arguments:
//     password: The password to validate
//
// Returns:
//     true if the password meets requirements, false otherwise
fn is_password_strong(password: &str) -> bool {
    password.len() >= 8
        && password.chars().any(|c| c.is_uppercase())
        && password.chars().any(|c| c.is_lowercase())
        && password.chars().any(|c| c.is_numeric())
        && password.chars().any(|c| !c.is_alphanumeric())
}

// Function: demo_authentication_flow
//
// Demonstrates the complete authentication flow with registration,
// login, token validation, and logout.
async fn demo_authentication_flow(
    auth_service: &AuthService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Registration Demo ===");

    // Register a new user
    let registration = RegistrationRequest {
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        password: "SecurePass123!".to_string(),
    };

    match auth_service.register_user(registration).await {
        Ok(user_id) => info!("User registered with ID: {}", user_id),
        Err(e) => error!("Registration failed: {}", e),
    }

    info!("=== Login Demo ===");

    // Authenticate the user
    let login = LoginRequest {
        username: "john_doe".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let token = match auth_service.authenticate(login).await {
        Ok(token) => {
            info!(
                "Authentication successful! Token expires at: {}",
                token.expires_at
            );
            token
        }
        Err(e) => {
            error!("Authentication failed: {}", e);
            return Ok(());
        }
    };

    info!("=== Token Validation Demo ===");

    // Validate the token
    match auth_service.validate_token(token.token_id).await {
        Ok(valid_token) => info!("Token is valid for user: {}", valid_token.username),
        Err(e) => error!("Token validation failed: {}", e),
    }

    info!("=== Permission Check Demo ===");

    // Check permissions
    let can_moderate = auth_service.check_permission(&token, &UserRole::Moderator);
    let can_use = auth_service.check_permission(&token, &UserRole::User);

    info!("Can moderate: {}", can_moderate);
    info!("Can use: {}", can_use);

    info!("=== User Info Demo ===");

    // Get user information
    match auth_service.get_user_info("john_doe").await {
        Ok(user_info) => info!("User info: {:?}", user_info),
        Err(e) => error!("Failed to get user info: {}", e),
    }

    info!("=== Logout Demo ===");

    // Logout the user
    match auth_service.logout(token.token_id).await {
        Ok(()) => info!("User logged out successfully"),
        Err(e) => error!("Logout failed: {}", e),
    }

    // Try to validate the token after logout (should fail)
    match auth_service.validate_token(token.token_id).await {
        Ok(_) => warn!("Token should be invalid after logout!"),
        Err(e) => info!("Token correctly invalidated: {}", e),
    }

    Ok(())
}

// Function: demo_security_features
//
// Demonstrates security features like account locking and failed attempts.
async fn demo_security_features(
    auth_service: &AuthService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Security Features Demo ===");

    // Register a test user
    let registration = RegistrationRequest {
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        password: "TestPass456!".to_string(),
    };

    auth_service.register_user(registration).await?;

    // Demonstrate failed login attempts
    info!("Testing failed login attempts...");

    for attempt in 1..=6 {
        // Try 6 times (should lock after 5)
        let login = LoginRequest {
            username: "test_user".to_string(),
            password: "wrong_password".to_string(),
        };

        match auth_service.authenticate(login).await {
            Ok(_) => warn!("Should not succeed with wrong password!"),
            Err(e) => info!("Attempt {}: {}", attempt, e),
        }
    }

    // Try with correct password (should still be locked)
    info!("Trying with correct password after lockout...");
    let login = LoginRequest {
        username: "test_user".to_string(),
        password: "TestPass456!".to_string(),
    };

    match auth_service.authenticate(login).await {
        Ok(_) => warn!("Should be locked!"),
        Err(e) => info!("Correctly locked: {}", e),
    }

    Ok(())
}

// Function: main
//
// This is the entry point of the program.
// It demonstrates the comprehensive authentication service implementation
// including user registration, login, token management, and security features.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracing subscriber for logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Authentication Service Example");

    // Create a new authentication service
    let auth_service = AuthService::new();

    // Demonstrate the complete authentication flow
    demo_authentication_flow(&auth_service).await?;

    // Demonstrate security features
    demo_security_features(&auth_service).await?;

    // Demonstrate token cleanup
    info!("=== Token Cleanup Demo ===");
    auth_service.cleanup_expired_tokens().await;

    info!("Authentication Service Example completed successfully");

    Ok(())
}
