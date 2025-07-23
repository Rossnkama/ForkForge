# Traits in Production: When & Why

## The HttpClient Trait Pattern

```rust
// PROBLEM: Domain layer coupled to infrastructure
pub struct GitHubAuthService {
    http_client: reqwest::Client,  // ❌ Domain knows about reqwest!
}

// SOLUTION: Abstract with trait
pub trait HttpClient: Send + Sync {
    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError>;
}

pub struct GitHubAuthService<C: HttpClient> {
    http_client: C,  // ✅ Domain only knows the contract
}
```

## Rule of Thumb: Use Traits When

1. **Crossing architectural boundaries** - Domain ↔ Infrastructure
2. **External dependencies** - Database, HTTP, filesystem
3. **Testing needs mocks** - Can't/shouldn't hit real services
4. **Multiple implementations likely** - Different providers (AWS/GCP)

## Real Example from Our Codebase

```rust
// crates/domain/src/services/auth/github.rs
pub trait HttpClient { /* ... */ }  // Domain defines what it needs

// crates/api/src/http_client.rs
impl HttpClient for ReqwestHttpClient {  // Infrastructure implements it
    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError> {
        // All the messy reqwest details hidden here
    }
}

// In tests:
struct MockHttpClient { responses: HashMap<String, String> }
impl HttpClient for MockHttpClient { /* return canned responses */ }
```

## Don't Use Traits When

- Internal implementation details
- Single implementation forever (e.g., business logic)
- Would just add complexity without benefit

**Think**: "Will I need to swap this out?" If yes → trait. If no → direct implementation.

