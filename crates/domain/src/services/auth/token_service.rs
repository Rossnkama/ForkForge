use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Service for generating and managing API tokens
pub struct TokenService;

impl TokenService {
    /// Generate a new API token using UUID v4
    pub fn generate_api_token() -> String {
        Uuid::new_v4().to_string()
    }

    /// Hash a token for secure storage using SHA256 with a salt
    ///
    /// The salt should be unique per user (e.g., user ID) to prevent
    /// rainbow table attacks even if the database is compromised
    pub fn hash_token(token: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hasher.update(salt.as_bytes());
        let result = hasher.finalize();
        format!("{result:x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_token() {
        let token1 = TokenService::generate_api_token();
        let token2 = TokenService::generate_api_token();

        // Tokens should be unique
        assert_ne!(token1, token2);

        // Should be valid UUID format
        assert!(Uuid::parse_str(&token1).is_ok());
    }

    #[test]
    fn test_hash_token() {
        let token = "test-token";
        let user_id = "550e8400-e29b-41d4-a716-446655440000";

        let hash1 = TokenService::hash_token(token, user_id);
        let hash2 = TokenService::hash_token(token, user_id);

        // Same token + salt should produce same hash
        assert_eq!(hash1, hash2);

        // Hash should be 64 chars (SHA256 hex)
        assert_eq!(hash1.len(), 64);

        // Different tokens should produce different hashes
        let different_hash = TokenService::hash_token("different-token", user_id);
        assert_ne!(hash1, different_hash);

        // Same token with different salt should produce different hash
        let different_salt_hash = TokenService::hash_token(token, "different-user-id");
        assert_ne!(hash1, different_salt_hash);
    }
}
