use crate::errors::DomainError;
use crate::services::billing::stripe_client::{StripeClient, StripeWebhookEvent};

/// Domain service for processing Stripe webhooks
pub struct StripeWebhookService<C: StripeClient> {
    stripe_client: C,
}

impl<C: StripeClient> StripeWebhookService<C> {
    pub fn new(stripe_client: C) -> Self {
        Self { stripe_client }
    }

    /// Process a Stripe webhook with signature verification
    pub async fn process_webhook(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<(), DomainError> {
        // Verify the webhook signature and get the event
        let event = self
            .stripe_client
            .verify_webhook_signature(payload, signature)
            .await?;

        // Process the event
        self.handle_event(event).await
    }

    /// Handle a verified Stripe event
    async fn handle_event(&self, event: StripeWebhookEvent) -> Result<(), DomainError> {
        match event.event_type.as_str() {
            "customer.subscription.created" => {
                // Handle new subscription
                println!("Processing subscription created: {}", event.id);
            }
            "customer.subscription.updated" => {
                // Handle subscription updates
                println!("Processing subscription updated: {}", event.id);
            }
            "customer.subscription.deleted" => {
                // Handle subscription cancellation
                println!("Processing subscription deleted: {}", event.id);
            }
            _ => {
                // Log unknown event type
                println!("Unknown Stripe event type: {}", event.event_type);
            }
        }

        Ok(())
    }
}

/// Legacy function for backward compatibility
pub async fn process_stripe_webhook(
    event_type: &str,
    _event_data: serde_json::Value,
) -> Result<(), DomainError> {
    // This is a stub for backward compatibility
    println!("Legacy webhook handler called for event: {event_type}");
    Ok(())
}
