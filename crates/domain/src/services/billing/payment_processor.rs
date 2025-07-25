//! Generic payment and subscription management
//!
//! This module consolidates all billing-related domain contracts including
//! payment processing and subscription management, without coupling to any
//! specific payment provider (Stripe, PayPal, etc).

use crate::errors::DomainError;
use crate::models::user::{SubscriptionStatus, SubscriptionTier};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain-agnostic customer identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomerId(pub String);

/// Domain-agnostic subscription identifier  
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub String);

/// Payment method identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentMethodId(pub String);

/// Domain contract for payment processing operations
///
/// Infrastructure implementations handle provider-specific details
/// (Stripe API, PayPal SDK, etc) while the domain remains agnostic.
#[async_trait]
pub trait PaymentProcessor: Send + Sync {
    /// Create a customer in the payment system
    async fn create_customer(
        &self,
        email: &str,
        external_id: &str, // Our system's user ID
    ) -> Result<CustomerId, DomainError>;

    /// Create a subscription for a customer
    async fn create_subscription(
        &self,
        customer_id: &CustomerId,
        tier: SubscriptionTier,
    ) -> Result<SubscriptionId, DomainError>;

    /// Update subscription to a new tier
    async fn update_subscription(
        &self,
        subscription_id: &SubscriptionId,
        new_tier: SubscriptionTier,
    ) -> Result<(), DomainError>;

    /// Cancel a subscription
    async fn cancel_subscription(
        &self,
        subscription_id: &SubscriptionId,
    ) -> Result<(), DomainError>;

    /// Check if a webhook signature is valid
    async fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<bool, DomainError>;
}

/// Webhook event handler for payment events
///
/// Infrastructure parses provider-specific webhooks and calls
/// appropriate domain services (SubscriptionService, etc)
#[async_trait]
pub trait PaymentWebhookHandler: Send + Sync {
    /// Process a webhook payload
    /// Returns Ok(true) if processed, Ok(false) if unrecognized
    async fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<bool, DomainError>;
}

// ===== Subscription Management =====

/// Domain-defined contract for subscription management
///
/// This trait defines pure business operations for managing user subscriptions.
/// Infrastructure layer handles provider-specific details (Stripe, etc.) and
/// calls these methods to execute business logic.
#[async_trait]
pub trait SubscriptionService: Send + Sync {
    /// Activate a new subscription for a user
    async fn activate_subscription(
        &self,
        user_id: Uuid,
        tier: SubscriptionTier,
        provider_subscription_id: String,
    ) -> Result<(), DomainError>;

    /// Update an existing subscription to a new tier
    async fn update_subscription(
        &self,
        user_id: Uuid,
        new_tier: SubscriptionTier,
    ) -> Result<(), DomainError>;

    /// Cancel a user's subscription
    async fn cancel_subscription(&self, user_id: Uuid) -> Result<(), DomainError>;

    /// Record a failed payment attempt
    async fn record_payment_failure(
        &self,
        user_id: Uuid,
        amount_cents: i64,
    ) -> Result<(), DomainError>;

    /// Get current subscription status for a user
    async fn get_subscription_status(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(SubscriptionTier, SubscriptionStatus)>, DomainError>;
}

/// Domain-defined contract for subscription persistence
#[async_trait]
pub trait SubscriptionRepository: Send + Sync {
    /// Create or update a subscription
    async fn upsert_subscription(
        &self,
        user_id: Uuid,
        tier: SubscriptionTier,
        status: SubscriptionStatus,
        provider_subscription_id: String,
    ) -> Result<(), DomainError>;

    /// Update subscription tier
    async fn update_tier(
        &self,
        user_id: Uuid,
        new_tier: SubscriptionTier,
    ) -> Result<(), DomainError>;

    /// Update subscription status
    async fn update_status(
        &self,
        user_id: Uuid,
        status: SubscriptionStatus,
    ) -> Result<(), DomainError>;

    /// Get subscription info
    async fn get_subscription(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(SubscriptionTier, SubscriptionStatus, String)>, DomainError>;

    /// Record payment failure
    async fn record_payment_failure(
        &self,
        user_id: Uuid,
        amount_cents: i64,
    ) -> Result<(), DomainError>;
}

/// Domain service implementation for subscription operations
///
/// This service orchestrates subscription operations using injected repositories
/// and other domain services.
pub struct SubscriptionServiceImpl<R: SubscriptionRepository> {
    repository: R,
}

impl<R: SubscriptionRepository> SubscriptionServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: SubscriptionRepository> SubscriptionService for SubscriptionServiceImpl<R> {
    async fn activate_subscription(
        &self,
        user_id: Uuid,
        tier: SubscriptionTier,
        provider_subscription_id: String,
    ) -> Result<(), DomainError> {
        self.repository
            .upsert_subscription(
                user_id,
                tier,
                SubscriptionStatus::Active,
                provider_subscription_id,
            )
            .await
    }

    async fn update_subscription(
        &self,
        user_id: Uuid,
        new_tier: SubscriptionTier,
    ) -> Result<(), DomainError> {
        self.repository.update_tier(user_id, new_tier).await
    }

    async fn cancel_subscription(&self, user_id: Uuid) -> Result<(), DomainError> {
        self.repository
            .update_status(user_id, SubscriptionStatus::Cancelled)
            .await
    }

    async fn record_payment_failure(
        &self,
        user_id: Uuid,
        amount_cents: i64,
    ) -> Result<(), DomainError> {
        // Record the failure
        self.repository
            .record_payment_failure(user_id, amount_cents)
            .await?;

        // Update subscription status to past due
        self.repository
            .update_status(user_id, SubscriptionStatus::PastDue)
            .await
    }

    async fn get_subscription_status(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(SubscriptionTier, SubscriptionStatus)>, DomainError> {
        let result = self.repository.get_subscription(user_id).await?;
        Ok(result.map(|(tier, status, _)| (tier, status)))
    }
}
