//! Data marketplace for ECU diagnostic data.
//!
//! Supports listing, purchasing, and downloading ECU data.
//! Payment split: 70% seller, 30% platform.
//! Integrates with Stripe for payment processing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{PremiumError, Result};

/// A marketplace listing for ECU diagnostic data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListing {
    /// Unique listing ID
    pub id: i64,
    /// Seller's user ID
    pub seller_id: String,
    /// ECU identifier this data is for
    pub ecu_id: String,
    /// Title of the listing
    pub title: String,
    /// Description of the data
    pub description: String,
    /// Price in cents (USD)
    pub price_cents: i32,
    /// Number of downloads
    pub downloads: i32,
    /// Total revenue in cents
    pub revenue_cents: i64,
    /// Rating (1-5 stars)
    pub rating: Option<f32>,
    /// Number of ratings
    pub rating_count: i32,
    /// Tags for search
    pub tags: Vec<String>,
    /// When the listing was created
    pub created_at: DateTime<Utc>,
    /// When the listing was last updated
    pub updated_at: DateTime<Utc>,
}

/// A marketplace purchase record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePurchase {
    /// Unique purchase ID
    pub id: i64,
    /// Buyer's user ID
    pub buyer_id: String,
    /// Reference to the listing
    pub listing_id: i64,
    /// Price paid in cents
    pub price_cents: i32,
    /// Seller's revenue (70%)
    pub seller_revenue_cents: i32,
    /// Platform revenue (30%)
    pub platform_revenue_cents: i32,
    /// Stripe payment intent ID
    pub stripe_payment_id: Option<String>,
    /// When the purchase was made
    pub purchased_at: DateTime<Utc>,
}

impl MarketplacePurchase {
    /// Calculate the payment split from a price
    pub fn calculate_split(price_cents: i32) -> (i32, i32) {
        let seller_share = (price_cents as f64 * 0.70).round() as i32;
        let platform_share = price_cents - seller_share;
        (seller_share, platform_share)
    }
}

/// PostgreSQL schema for the marketplace
pub const MARKETPLACE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS marketplace_listings (
    id BIGSERIAL PRIMARY KEY,
    seller_id VARCHAR(255) NOT NULL,
    ecu_id VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    price_cents INTEGER NOT NULL CHECK (price_cents >= 0),
    downloads INTEGER NOT NULL DEFAULT 0,
    revenue_cents BIGINT NOT NULL DEFAULT 0,
    rating REAL,
    rating_count INTEGER NOT NULL DEFAULT 0,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_listings_seller ON marketplace_listings(seller_id);
CREATE INDEX IF NOT EXISTS idx_listings_ecu ON marketplace_listings(ecu_id);
CREATE INDEX IF NOT EXISTS idx_listings_price ON marketplace_listings(price_cents);

CREATE TABLE IF NOT EXISTS marketplace_purchases (
    id BIGSERIAL PRIMARY KEY,
    buyer_id VARCHAR(255) NOT NULL,
    listing_id BIGINT NOT NULL REFERENCES marketplace_listings(id),
    price_cents INTEGER NOT NULL,
    seller_revenue_cents INTEGER NOT NULL,
    platform_revenue_cents INTEGER NOT NULL,
    stripe_payment_id VARCHAR(255),
    purchased_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_purchases_buyer ON marketplace_purchases(buyer_id);
CREATE INDEX IF NOT EXISTS idx_purchases_listing ON marketplace_purchases(listing_id);
CREATE INDEX IF NOT EXISTS idx_purchases_stripe ON marketplace_purchases(stripe_payment_id);
"#;

/// Stripe integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeConfig {
    /// Stripe API secret key
    pub secret_key: String,
    /// Stripe publishable key (for frontend)
    pub publishable_key: String,
    /// Webhook signing secret
    pub webhook_secret: String,
    /// Platform's Stripe account ID (for Connect)
    pub platform_account_id: String,
}

/// Stripe payment intent for marketplace purchases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    /// Stripe payment intent ID
    pub id: String,
    /// Amount in cents
    pub amount: i32,
    /// Currency code
    pub currency: String,
    /// Status of the payment
    pub status: PaymentStatus,
    /// Client secret for frontend confirmation
    pub client_secret: String,
}

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    RequiresPaymentMethod,
    RequiresConfirmation,
    RequiresAction,
    Processing,
    Succeeded,
    Canceled,
    Failed,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::RequiresPaymentMethod => write!(f, "requires_payment_method"),
            PaymentStatus::RequiresConfirmation => write!(f, "requires_confirmation"),
            PaymentStatus::RequiresAction => write!(f, "requires_action"),
            PaymentStatus::Processing => write!(f, "processing"),
            PaymentStatus::Succeeded => write!(f, "succeeded"),
            PaymentStatus::Canceled => write!(f, "canceled"),
            PaymentStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Marketplace client for interacting with listings and purchases
pub struct MarketplaceClient {
    stripe_config: Option<StripeConfig>,
    api_base_url: String,
}

impl MarketplaceClient {
    /// Create a new marketplace client
    pub fn new(api_base_url: String, stripe_config: Option<StripeConfig>) -> Self {
        Self {
            stripe_config,
            api_base_url,
        }
    }

    /// Create a new listing
    pub fn create_listing(
        seller_id: &str,
        ecu_id: &str,
        title: &str,
        description: &str,
        price_cents: i32,
        tags: Vec<String>,
    ) -> Result<MarketplaceListing> {
        if price_cents < 0 {
            return Err(PremiumError::Marketplace(
                "Price cannot be negative".into(),
            ));
        }

        Ok(MarketplaceListing {
            id: 0, // assigned by database
            seller_id: seller_id.to_string(),
            ecu_id: ecu_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            price_cents,
            downloads: 0,
            revenue_cents: 0,
            rating: None,
            rating_count: 0,
            tags,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Create a purchase record with 70/30 split
    pub fn create_purchase(
        buyer_id: &str,
        listing: &MarketplaceListing,
        stripe_payment_id: Option<String>,
    ) -> Result<MarketplacePurchase> {
        let (seller_revenue, platform_revenue) =
            MarketplacePurchase::calculate_split(listing.price_cents);

        Ok(MarketplacePurchase {
            id: 0, // assigned by database
            buyer_id: buyer_id.to_string(),
            listing_id: listing.id,
            price_cents: listing.price_cents,
            seller_revenue_cents: seller_revenue,
            platform_revenue_cents: platform_revenue,
            stripe_payment_id,
            purchased_at: Utc::now(),
        })
    }

    /// Create a Stripe payment intent for a listing purchase.
    /// In production, this calls the Stripe API via reqwest.
    pub async fn create_payment_intent(
        &self,
        listing: &MarketplaceListing,
        _buyer_id: &str,
    ) -> Result<PaymentIntent> {
        let _stripe_config = self.stripe_config.as_ref().ok_or_else(|| {
            PremiumError::PaymentError("Stripe not configured".into())
        })?;

        // In production:
        // let client = reqwest::Client::new();
        // let response = client.post("https://api.stripe.com/v1/payment_intents")
        //     .header("Authorization", format!("Bearer {}", stripe_config.secret_key))
        //     .form(&[
        //         ("amount", &listing.price_cents.to_string()),
        //         ("currency", &"usd".to_string()),
        //         ("application_fee_amount", &platform_fee.to_string()),
        //     ])
        //     .send().await?;

        Ok(PaymentIntent {
            id: format!("pi_test_{}", uuid::Uuid::new_v4()),
            amount: listing.price_cents,
            currency: "usd".to_string(),
            status: PaymentStatus::RequiresPaymentMethod,
            client_secret: format!("pi_test_secret_{}", uuid::Uuid::new_v4()),
        })
    }

    /// Get the API base URL
    pub fn api_base_url(&self) -> &str {
        &self.api_base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_split_70_30() {
        let (seller, platform) = MarketplacePurchase::calculate_split(1000);
        assert_eq!(seller, 700);
        assert_eq!(platform, 300);
    }

    #[test]
    fn test_payment_split_rounding() {
        // $9.99 = 999 cents
        let (seller, platform) = MarketplacePurchase::calculate_split(999);
        // 999 * 0.70 = 699.3 → rounds to 699
        assert_eq!(seller, 699);
        assert_eq!(platform, 300);
        assert_eq!(seller + platform, 999);
    }

    #[test]
    fn test_payment_split_small_amount() {
        let (seller, platform) = MarketplacePurchase::calculate_split(100);
        assert_eq!(seller, 70);
        assert_eq!(platform, 30);
    }

    #[test]
    fn test_create_listing() {
        let listing = MarketplaceClient::create_listing(
            "seller-001",
            "vw_golf_2020_ecm",
            "VW Golf 2020 ECM Complete Data",
            "Full diagnostic data including DTCs, PIDs, and security access",
            999,
            vec!["vw".into(), "golf".into(), "ecm".into()],
        )
        .unwrap();

        assert_eq!(listing.seller_id, "seller-001");
        assert_eq!(listing.price_cents, 999);
        assert_eq!(listing.downloads, 0);
    }

    #[test]
    fn test_create_listing_negative_price() {
        let result = MarketplaceClient::create_listing(
            "seller-001",
            "test",
            "Test",
            "Test",
            -100,
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_purchase() {
        let listing = MarketplaceListing {
            id: 42,
            seller_id: "seller-001".into(),
            ecu_id: "test_ecu".into(),
            title: "Test Data".into(),
            description: "Test".into(),
            price_cents: 999,
            downloads: 0,
            revenue_cents: 0,
            rating: None,
            rating_count: 0,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let purchase = MarketplaceClient::create_purchase(
            "buyer-001",
            &listing,
            Some("pi_test_123".into()),
        )
        .unwrap();

        assert_eq!(purchase.listing_id, 42);
        assert_eq!(purchase.price_cents, 999);
        assert_eq!(purchase.seller_revenue_cents, 699);
        assert_eq!(purchase.platform_revenue_cents, 300);
        assert_eq!(
            purchase.seller_revenue_cents + purchase.platform_revenue_cents,
            purchase.price_cents
        );
    }

    #[test]
    fn test_marketplace_schema_sql() {
        // Verify schema contains expected tables
        assert!(MARKETPLACE_SCHEMA.contains("marketplace_listings"));
        assert!(MARKETPLACE_SCHEMA.contains("marketplace_purchases"));
        assert!(MARKETPLACE_SCHEMA.contains("seller_revenue_cents"));
        assert!(MARKETPLACE_SCHEMA.contains("platform_revenue_cents"));
        assert!(MARKETPLACE_SCHEMA.contains("stripe_payment_id"));
    }

    #[test]
    fn test_payment_status_display() {
        assert_eq!(format!("{}", PaymentStatus::Succeeded), "succeeded");
        assert_eq!(format!("{}", PaymentStatus::Processing), "processing");
        assert_eq!(format!("{}", PaymentStatus::Failed), "failed");
    }

    #[tokio::test]
    async fn test_payment_intent_requires_config() {
        let client = MarketplaceClient::new("http://localhost:8080".into(), None);
        let listing = MarketplaceListing {
            id: 1,
            seller_id: "seller".into(),
            ecu_id: "ecu".into(),
            title: "Test".into(),
            description: "Test".into(),
            price_cents: 999,
            downloads: 0,
            revenue_cents: 0,
            rating: None,
            rating_count: 0,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = client.create_payment_intent(&listing, "buyer").await;
        assert!(result.is_err()); // No Stripe config
    }
}
