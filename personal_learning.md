# Production Rust Patterns

<!--toc:start-->
- [Production Rust Patterns](#production-rust-patterns)
  - [Traits in Production: When & Why](#traits-in-production-when-why)
    - [Rule of Thumb: Use Traits When](#rule-of-thumb-use-traits-when)
    - [Don't Use Traits When](#dont-use-traits-when)
  - [The HttpClient Trait Pattern](#the-httpclient-trait-pattern)
    - [Real Example from Our Codebase](#real-example-from-our-codebase)
  - [The Adapter Pattern](#the-adapter-pattern)
  - [The Repository Pattern](#the-repository-pattern)
    - [Why It's Brilliant](#why-its-brilliant)
    - [Real-World Example: Solana Fork State](#real-world-example-solana-fork-state)
    - [The Pattern's DNA](#the-patterns-dna)
    - [When to Use](#when-to-use)
<!--toc:end-->

## Traits in Production: When & Why

### Rule of Thumb: Use Traits When

1. **Crossing architectural boundaries** - Domain ↔ Infrastructure
2. **External dependencies** - Database, HTTP, filesystem
3. **Testing needs mocks** - Can't/shouldn't hit real services
4. **Multiple implementations likely** - Different providers (AWS/GCP)

### Don't Use Traits When

- Internal implementation details
- Single implementation forever (e.g., business logic)
- Would just add complexity without benefit

**Think**: "Will I need to swap this out?" If yes → trait. If no → direct implementation.

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

### Real Example from Our Codebase

```rust
// crates/domain/src/services/auth/github.rs
pub trait HttpClient { /* ... */ }  // Domain defines what it needs

// crates/api/src/http_client.rs
impl HttpClient for ReqwestAdapter {  // Infrastructure implements it
    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError> {
        // All the messy reqwest details hidden here
    }
}

// In tests:
struct MockHttpClient { responses: HashMap<String, String> }
impl HttpClient for MockHttpClient { /* return canned responses */ }
```

## The Adapter Pattern

```rust
// Step 1: Create concrete implementation
let http_client = reqwest::Client::builder().build()?;

// Step 2: Wrap in adapter (implements domain trait)
let reqwest_adapter = ReqwestAdapter::new(http_client);

// Step 3: Inject into domain service
let service = GitHubAuthService::new(client_id, reqwest_adapter);
```

**Why "Adapter"?** Clear it's YOUR code wrapping external dependency, not the library itself

## The Repository Pattern

Think of it like plumbing - you don't care if water comes from a well, city pipes, or tank. You just turn the tap. Repository pattern is that tap for data.

```rust
// The trait defines WHAT you can do, not HOW
trait UserRepository {
    async fn find_by_id(&self, id: u64) -> Result<User>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn list_active(&self) -> Result<Vec<User>>;
}

// Different implementations define HOW
struct PostgresUserRepo { 
    pool: PgPool 
}

struct RedisUserRepo { 
    client: RedisClient,
    ttl: Duration,
}

struct InMemoryUserRepo { 
    users: Arc<RwLock<HashMap<u64, User>>> 
}

// Your business logic doesn't care which implementation
async fn promote_user(repo: &impl UserRepository, user_id: u64) -> Result<()> {
    let mut user = repo.find_by_id(user_id).await?;
    user.role = Role::Admin;
    user.promoted_at = Some(Utc::now());
    repo.save(&user).await?;
}
```

### Why It's Brilliant

1. **Testing**: Use `InMemoryUserRepo` in tests - blazing fast, no database needed
2. **Caching**: Layer repositories like Russian dolls:

   ```rust
   struct CachedUserRepo<R: UserRepository> {
       cache: RedisUserRepo,
       source: R,
   }
   
   impl<R: UserRepository> UserRepository for CachedUserRepo<R> {
       async fn find_by_id(&self, id: u64) -> Result<User> {
           // Check cache first
           if let Ok(Some(user)) = self.cache.find_by_id(id).await {
               return Ok(user);
           }
           // Fall back to source
           let user = self.source.find_by_id(id).await?;
           self.cache.save(&user).await?;
           Ok(user)
       }
   }
   ```

3. **Migration**: Switch data stores without touching business logic
4. **Multi-tenancy**: Different repos for different customers

### Real-World Example: Solana Fork State

```rust
trait AccountRepository {
    async fn get_account(&self, pubkey: &Pubkey) -> Result<Account>;
    async fn clone_from_mainnet(&self, pubkey: &Pubkey) -> Result<Account>;
}

struct LiveRpcRepo { rpc_client: RpcClient }
struct SnapshotRepo { snapshot_path: PathBuf }
struct HybridRepo { 
    local_cache: HashMap<Pubkey, Account>,
    fallback: Box<dyn AccountRepository>,
}

// ForkForge can seamlessly switch between:
// - Live mainnet data (LiveRpcRepo)
// - Time-travel snapshots (SnapshotRepo)  
// - Local fork with mainnet fallback (HybridRepo)
```

### The Pattern's DNA

Repository isn't about databases - it's about **data access abstraction**. Any source:

- Files on disk
- Remote APIs
- In-memory structures
- Blockchain state
- Message queues

### When to Use

✅ **Use Repository when:**

- Multiple data sources possible
- Need to mock for testing
- Want to add caching/metrics/logging transparently
- Data access is complex enough to warrant abstraction

❌ **Skip it when:**

- Only ever one data source
- Simple CRUD that frameworks handle
- Would just be a thin wrapper adding no value

**Feynman Test**: Can you explain your data access to a new dev without mentioning the storage tech? Then you've got a good repository abstraction.

