-- Add github_id to users table
ALTER TABLE users ADD COLUMN github_id TEXT;
CREATE UNIQUE INDEX idx_users_github_id ON users(github_id);

-- Add last_used_at to auth_credentials table
ALTER TABLE auth_credentials ADD COLUMN last_used_at TIMESTAMP;
