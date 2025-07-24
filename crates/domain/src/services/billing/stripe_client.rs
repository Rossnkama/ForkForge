//! # Stripe Client Interface
//!
//! This module defines the domain's contract for payment processing operations.
//! Following the Dependency Inversion Principle, the domain defines what it needs
//! from a payment processor without knowing implementation details.
//!
//! ## Architecture
//!
//! The `StripeClient` trait is implemented by the infrastructure layer's `StripeSdk`,
//! allowing the domain to remain independent of specific payment processing libraries
//! or APIs while still defining the operations it requires.

use crate::errors::DomainError;
use crate::models::user::{SubscriptionStatus, SubscriptionTier};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Domain-defined contract for Stripe payment operations
///
/// This trait defines what the domain needs from Stripe without knowing HOW it's implemented.
/// The infrastructure layer provides concrete implementations via `StripeSdk`.
///
/// ## Operations
///
/// - Customer management (creation)
/// - Subscription lifecycle (create, update, cancel, retrieve)
/// - Webhook signature verification
#[async_trait]
pub trait StripeClient: Send + Sync {
    /// Create a new customer in Stripe
    async fn create_customer(
        &self,
        email: &str,
        metadata: Option<CustomerMetadata>,
    ) -> Result<StripeCustomer, DomainError>;

    /// Create a subscription for a customer
    async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
    ) -> Result<StripeSubscription, DomainError>;

    /// Update a subscription
    async fn update_subscription(
        &self,
        subscription_id: &str,
        price_id: &str,
    ) -> Result<StripeSubscription, DomainError>;

    /// Cancel a subscription
    async fn cancel_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<StripeSubscription, DomainError>;

    /// Get subscription details
    async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<StripeSubscription, DomainError>;

    /// Verify webhook signature
    async fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<StripeWebhookEvent, DomainError>;
}

/// Customer metadata for Stripe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerMetadata {
    pub github_id: Option<String>,
    pub user_id: String,
}

/// Stripe customer representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeCustomer {
    pub id: String,
    pub email: String,
    pub created: i64,
}

/// Stripe subscription representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeSubscription {
    pub id: String,
    pub customer: String,
    pub status: String,
    pub current_period_end: i64,
    pub items: Vec<SubscriptionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionItem {
    pub id: String,
    pub price: Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub id: String,
    pub product: String,
    pub unit_amount: Option<i64>,
    pub currency: String,
}

/// Stripe webhook event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookEvent {
    pub id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub created: i64,
}

impl StripeSubscription {
    /// Convert Stripe status to domain subscription status
    pub fn to_domain_status(&self) -> SubscriptionStatus {
        match self.status.as_str() {
            "active" => SubscriptionStatus::Active,
            "past_due" => SubscriptionStatus::PastDue,
            "canceled" | "unpaid" => SubscriptionStatus::Cancelled,
            _ => SubscriptionStatus::Cancelled,
        }
    }

    /// Determine subscription tier from price ID
    pub fn to_domain_tier(&self, price_id: &str) -> SubscriptionTier {
        // This would be configured based on your Stripe product/price IDs
        match price_id {
            "price_entry" => SubscriptionTier::Entry,
            "price_lite" => SubscriptionTier::Lite,
            "price_pro" => SubscriptionTier::Pro,
            _ => SubscriptionTier::Entry,
        }
    }
}
