/// Database operations for authentication
/// 
/// This module contains all database operations for users and sessions

use super::models::{CreateSession, CreateUser, UpdateUserTokens, User, UserSession};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

// ============================================================================
// User Operations
// ============================================================================

/// Find user by provider user ID and auth provider
pub async fn find_user_by_provider(
    db: &PgPool,
    org_id: &str,
    provider_user_id: &str,
    auth_provider: &str,
) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE org_id = $1 
          AND provider_user_id = $2 
          AND auth_provider = $3
          AND is_active = TRUE
        "#,
    )
    .bind(org_id)
    .bind(provider_user_id)
    .bind(auth_provider)
    .fetch_optional(db)
    .await
    .context("Failed to find user by provider")?;
    
    Ok(user)
}

/// Find user by email in organization
pub async fn find_user_by_email(
    db: &PgPool,
    org_id: &str,
    email: &str,
) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE org_id = $1 
          AND email = $2
          AND is_active = TRUE
        "#,
    )
    .bind(org_id)
    .bind(email)
    .fetch_optional(db)
    .await
    .context("Failed to find user by email")?;
    
    Ok(user)
}

/// Find user by user ID
pub async fn find_user_by_id(db: &PgPool, user_id: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE user_id = $1
          AND is_active = TRUE
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .context("Failed to find user by ID")?;
    
    Ok(user)
}

/// Create a new user
pub async fn create_user(db: &PgPool, user: CreateUser) -> Result<User> {
    let now = Utc::now();
    
    let created_user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (
            user_id, email, name, display_name, picture,
            auth_provider, provider_user_id, org_id,
            access_token, refresh_token, id_token, token_expires_at,
            is_active, created_at, last_login_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
            TRUE, $13, $13, $13
        )
        RETURNING *
        "#,
    )
    .bind(&user.user_id)
    .bind(&user.email)
    .bind(&user.name)
    .bind(&user.display_name)
    .bind(&user.picture)
    .bind(&user.auth_provider)
    .bind(&user.provider_user_id)
    .bind(&user.org_id)
    .bind(&user.access_token)
    .bind(&user.refresh_token)
    .bind(&user.id_token)
    .bind(&user.token_expires_at)
    .bind(now)
    .fetch_one(db)
    .await
    .context("Failed to create user")?;
    
    Ok(created_user)
}

/// Update user tokens and last login time
pub async fn update_user_tokens(db: &PgPool, update: UpdateUserTokens) -> Result<User> {
    let now = Utc::now();
    
    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET access_token = $2,
            refresh_token = $3,
            id_token = $4,
            token_expires_at = $5,
            last_login_at = $6,
            updated_at = $6
        WHERE user_id = $1
        RETURNING *
        "#,
    )
    .bind(&update.user_id)
    .bind(&update.access_token)
    .bind(&update.refresh_token)
    .bind(&update.id_token)
    .bind(&update.token_expires_at)
    .bind(now)
    .fetch_one(db)
    .await
    .context("Failed to update user tokens")?;
    
    Ok(updated_user)
}

/// Update user profile information
pub async fn update_user_profile(
    db: &PgPool,
    user_id: &str,
    name: Option<String>,
    display_name: Option<String>,
    picture: Option<String>,
) -> Result<User> {
    let now = Utc::now();
    
    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET name = COALESCE($2, name),
            display_name = COALESCE($3, display_name),
            picture = COALESCE($4, picture),
            updated_at = $5
        WHERE user_id = $1
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(display_name)
    .bind(picture)
    .bind(now)
    .fetch_one(db)
    .await
    .context("Failed to update user profile")?;
    
    Ok(updated_user)
}

// ============================================================================
// Session Operations
// ============================================================================

/// Create a new session
pub async fn create_session(db: &PgPool, session: CreateSession) -> Result<UserSession> {
    let now = Utc::now();
    
    let created_session = sqlx::query_as::<_, UserSession>(
        r#"
        INSERT INTO user_sessions (
            session_id, user_id, org_id,
            ip_address, user_agent,
            is_active, created_at, expires_at, last_activity_at
        ) VALUES (
            $1, $2, $3, $4, $5, TRUE, $6, $7, $6
        )
        RETURNING *
        "#,
    )
    .bind(&session.session_id)
    .bind(&session.user_id)
    .bind(&session.org_id)
    .bind(&session.ip_address)
    .bind(&session.user_agent)
    .bind(now)
    .bind(&session.expires_at)
    .fetch_one(db)
    .await
    .context("Failed to create session")?;
    
    Ok(created_session)
}

/// Find active session by session ID
pub async fn find_session_by_id(
    db: &PgPool,
    session_id: &str,
) -> Result<Option<UserSession>> {
    let session = sqlx::query_as::<_, UserSession>(
        r#"
        SELECT * FROM user_sessions
        WHERE session_id = $1
          AND is_active = TRUE
          AND expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .fetch_optional(db)
    .await
    .context("Failed to find session")?;
    
    Ok(session)
}

/// Update session activity timestamp
pub async fn update_session_activity(
    db: &PgPool,
    session_id: &str,
) -> Result<UserSession> {
    let now = Utc::now();
    
    let updated_session = sqlx::query_as::<_, UserSession>(
        r#"
        UPDATE user_sessions
        SET last_activity_at = $2
        WHERE session_id = $1
        RETURNING *
        "#,
    )
    .bind(session_id)
    .bind(now)
    .fetch_one(db)
    .await
    .context("Failed to update session activity")?;
    
    Ok(updated_session)
}

/// Extend session expiration (sliding expiration)
pub async fn extend_session_expiration(
    db: &PgPool,
    session_id: &str,
    new_expires_at: DateTime<Utc>,
) -> Result<UserSession> {
    let now = Utc::now();
    
    let updated_session = sqlx::query_as::<_, UserSession>(
        r#"
        UPDATE user_sessions
        SET expires_at = $2,
            last_activity_at = $3
        WHERE session_id = $1
        RETURNING *
        "#,
    )
    .bind(session_id)
    .bind(new_expires_at)
    .bind(now)
    .fetch_one(db)
    .await
    .context("Failed to extend session expiration")?;
    
    Ok(updated_session)
}

/// Invalidate session (logout)
pub async fn invalidate_session(db: &PgPool, session_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE user_sessions
        SET is_active = FALSE
        WHERE session_id = $1
        "#,
    )
    .bind(session_id)
    .execute(db)
    .await
    .context("Failed to invalidate session")?;
    
    Ok(())
}

/// Invalidate all sessions for a user
pub async fn invalidate_all_user_sessions(db: &PgPool, user_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE user_sessions
        SET is_active = FALSE
        WHERE user_id = $1 AND is_active = TRUE
        "#,
    )
    .bind(user_id)
    .execute(db)
    .await
    .context("Failed to invalidate all user sessions")?;
    
    Ok(())
}

/// Clean up expired sessions (run periodically)
pub async fn cleanup_expired_sessions(db: &PgPool) -> Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM user_sessions
        WHERE expires_at < NOW() - INTERVAL '7 days'
        "#,
    )
    .execute(db)
    .await
    .context("Failed to cleanup expired sessions")?;
    
    Ok(result.rows_affected())
}

/// Get all active sessions for a user
pub async fn get_user_sessions(db: &PgPool, user_id: &str) -> Result<Vec<UserSession>> {
    let sessions = sqlx::query_as::<_, UserSession>(
        r#"
        SELECT * FROM user_sessions
        WHERE user_id = $1
          AND is_active = TRUE
          AND expires_at > NOW()
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .context("Failed to get user sessions")?;
    
    Ok(sessions)
}

// ============================================================================
// Session Extension Logic
// ============================================================================

/// Check if session should be extended based on threshold
pub fn should_extend_session(
    session: &UserSession,
    threshold: f64,
) -> bool {
    let now = Utc::now();
    let total_duration = session.expires_at - session.created_at;
    let elapsed = now - session.created_at;
    
    let elapsed_ratio = elapsed.num_seconds() as f64 / total_duration.num_seconds() as f64;
    
    elapsed_ratio >= threshold
}

/// Calculate new expiration time for session extension
pub fn calculate_new_expiration(max_age_seconds: i64) -> DateTime<Utc> {
    Utc::now() + Duration::seconds(max_age_seconds)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate unique user ID
pub fn generate_user_id() -> String {
    use oauth2::CsrfToken;
    format!("usr_{}", CsrfToken::new_random().secret())
}

/// Generate unique session ID
pub fn generate_session_id() -> String {
    use oauth2::CsrfToken;
    format!("ses_{}", CsrfToken::new_random().secret())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_should_extend_session() {
        let now = Utc::now();
        
        // Session created 1 hour ago, expires in 1 hour (50% elapsed)
        let session = UserSession {
            session_id: "test".to_string(),
            user_id: "user1".to_string(),
            org_id: "org1".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test".to_string(),
            is_active: true,
            created_at: now - Duration::hours(1),
            expires_at: now + Duration::hours(1),
            last_activity_at: now,
        };
        
        // Should extend with threshold 0.5 (50%)
        assert!(should_extend_session(&session, 0.5));
        
        // Should not extend with threshold 0.6 (60%)
        assert!(!should_extend_session(&session, 0.6));
    }
    
    #[test]
    fn test_calculate_new_expiration() {
        let now = Utc::now();
        let new_expiration = calculate_new_expiration(3600);
        
        let diff = (new_expiration - now).num_seconds();
        assert!(diff >= 3599 && diff <= 3601);
    }
    
    #[test]
    fn test_generate_ids() {
        let user_id1 = generate_user_id();
        let user_id2 = generate_user_id();
        assert_ne!(user_id1, user_id2);
        assert!(user_id1.starts_with("usr_"));
        
        let session_id1 = generate_session_id();
        let session_id2 = generate_session_id();
        assert_ne!(session_id1, session_id2);
        assert!(session_id1.starts_with("ses_"));
    }
}

