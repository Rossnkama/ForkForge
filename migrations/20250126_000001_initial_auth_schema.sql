-- Initial schema for ForkForge authentication system
-- Focus: User accounts and API token authentication

-- Users table: Core user accounts
CREATE TABLE users (
    id TEXT PRIMARY KEY,                    -- UUID v4
    email TEXT UNIQUE NOT NULL,             -- Primary identifier for login
    github_id INTEGER UNIQUE,               -- GitHub OAuth integration
    github_username TEXT UNIQUE,            -- GitHub username for display
    stripe_customer_id TEXT UNIQUE,         -- Stripe integration (nullable for free users)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Auth tokens: API authentication tokens
CREATE TABLE auth_tokens (
    id TEXT PRIMARY KEY,                    -- UUID v4
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,        -- SHA256 hash of the actual token
    name TEXT,                              -- Optional friendly name (e.g., "CLI on MacBook")
    last_used_at TIMESTAMP,                 -- Track token usage
    expires_at TIMESTAMP,                   -- NULL = never expires
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Performance indexes
CREATE INDEX idx_auth_tokens_user_id ON auth_tokens(user_id);
CREATE INDEX idx_auth_tokens_hash ON auth_tokens(token_hash);
CREATE INDEX idx_users_email ON users(email);

-- Future tables to add as features are built:
-- fork_sessions: When implementing Solana forking
-- snapshots: When implementing time-travel features
-- usage_metrics: When implementing billing/quotas
-- billing_events: When implementing paid subscriptions