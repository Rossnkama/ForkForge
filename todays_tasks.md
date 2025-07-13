### **Day 1 — Auth & Billing Foundation**

*(4 h target, every box unchecked → ready-to-ship when all are ticked)*

---

#### 0 – 0 : 20  🗒 Write the Flow Doc

* [ ] **`docs/user-flow.md`** — one page, max 20 min.

  * [x] Sequence diagram: *CLI → API → DB* for authenticated call.
  * [x] Sequence diagram: *Stripe → Webhook → DB → Key issue*.
  * [x] Bullet list of API-key lifecycle (create ➜ rotate ➜ revoke).
  * [ ] Note: rate-limiting & scopes = *future work*.

---

#### 0 : 20 – 1 : 20  🗄 Bootstrap Database (SQLite + SQLx)

* [ ] `sqlx migrate add V01__init_schema.sql` with **one file**:

  ```sql
  CREATE TABLE users (
    id TEXT PRIMARY KEY,           -- uuid v4
    stripe_id TEXT UNIQUE NOT NULL,
    subscription_status TEXT NOT NULL DEFAULT 'active',
    subscription_tier   TEXT NOT NULL DEFAULT 'pro',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );

  CREATE TABLE auth_credentials (
    id TEXT PRIMARY KEY,           -- uuid v4
    user_id TEXT NOT NULL REFERENCES users(id),
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP,          -- NULL = long-lived
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );
  ```

* [ ] `cargo install sqlx-cli` (if not yet).

* [ ] `sqlx migrate run` — verify both tables exist.

---

#### 1 : 20 – 2 : 50  🔑 API-Key Issuance + Auth Middleware

**Endpoint (`POST /auth/api-keys`)**

* [ ] Route handler: validate **Stripe-backed** user (`stripe_id` via JSON for now).
* [ ] `rand::thread_rng().fill_bytes(&mut [0u8;32])` → hex → raw token.
* [ ] `hash_token(raw)` (SHA-256) → insert into `auth_credentials`.
* [ ] Respond `{ "key": "<raw-token>", "key_id": "<uuid>" }`.

**Middleware (`auth_mw`)**

* [ ] Extract `Authorization: Bearer <token>`.
* [ ] `hash_token` → lookup in `auth_credentials` (and `expires_at` check).
* [ ] On hit: inject `AuthContext { user_id, tier }` into `req.extensions`.
* [ ] On miss: `401 {"error":"invalid_or_expired_token"}`.

⚡ **Smoke Tests**

* [ ] `curl -X POST /auth/api-keys …` returns token.
* [ ] Protected POST `/sessions` with/without token → 200 / 401.

---

#### 2 : 50 – 4 : 00  💸 Stripe Webhook MVP

* [ ] Endpoint `POST /billing/webhook` (public).
* [ ] Verify signature with `STRIPE_WEBHOOK_SECRET`.
* [ ] Handle `checkout.session.completed` **only**:

  * [ ] Upsert `users` (new `stripe_id` ⇒ new `id` UUID).
  * [ ] Issue API key via helper & store.
* [ ] Log issued token to console (temporary) for manual copy.
* [ ] Ack all events with **200 OK**.

⚡ **Test-drive**

* [ ] `stripe trigger checkout.session.completed` (Stripe CLI) → see token in logs.
* [ ] Use that token to hit `/sessions` → 200.

---

### 🎯 **Done-Checklist for Day 1**

* [ ] `cargo test` green; `cargo fmt` run.
* [ ] `docs/user-flow.md` committed.
* [ ] Database migrations version-controlled.
* [ ] API issues, verifies, and enforces keys end-to-end.
* [ ] Stripe webhook proves it can mint a key.

Stick to this pared-down rail; any spare minutes go to nicer logging (`tracing_subscriber`) or TODO comments for tomorrow. Good luck — ship it!
