-- migrations/20250721_000004_stripe_improvements.sql
-- 1) Re-create the users table with nullable stripe_id
PRAGMA foreign_keys = OFF;

CREATE TABLE users_new (
  id                TEXT PRIMARY KEY,          -- uuid v4
  stripe_id         TEXT UNIQUE,               -- now NULLABLE
  github_id         INTEGER UNIQUE,            -- GitHub user id
  subscription_status TEXT NOT NULL DEFAULT 'inactive'
    CHECK (subscription_status IN ('inactive','active','cancelled','past_due')),
  subscription_tier TEXT NOT NULL DEFAULT 'free',
  created_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users_new (id, stripe_id, subscription_status,
                       subscription_tier, created_at)
  SELECT id, stripe_id, subscription_status,
         subscription_tier, created_at
  FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;

PRAGMA foreign_keys = ON;
