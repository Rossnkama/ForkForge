CREATE TABLE users (
    id TEXT PRIMARY KEY,           -- uuid v4
    stripe_id TEXT UNIQUE NOT NULL,
    subscription_status TEXT NOT NULL DEFAULT 'active'
    CHECK (subscription_status IN ('active', 'cancelled', 'past_due')),
    subscription_tier TEXT NOT NULL DEFAULT 'pro',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auth_credentials (
    id TEXT PRIMARY KEY,                 -- uuid v4
    user_id TEXT NOT NULL REFERENCES users (id),
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP,                -- NULL = long-lived
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Performance / business rules
CREATE INDEX idx_auth_user ON auth_credentials (user_id);
CREATE UNIQUE INDEX uniq_active_token_per_user
    ON auth_credentials (user_id)
    WHERE expires_at IS NULL;

-- (Optional) Stripe event-dedup
CREATE TABLE stripe_events (
  id TEXT PRIMARY KEY,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
