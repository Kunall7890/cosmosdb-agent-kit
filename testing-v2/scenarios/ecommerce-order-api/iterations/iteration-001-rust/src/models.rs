use serde::{Deserialize, Serialize};

/// An individual item within an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    #[serde(rename = "productId")]
    pub product_id: String,
    #[serde(rename = "productName")]
    pub product_name: String,
    pub quantity: i64,
    #[serde(rename = "unitPrice")]
    pub unit_price: f64,
}

/// The full order document stored in Cosmos DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Cosmos DB document id
    pub id: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "customerId")]
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub total: f64,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "shippingAddress", skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<String>,
}

/// Request body for creating an order.
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    #[serde(rename = "customerId")]
    pub customer_id: Option<String>,
    pub items: Option<Vec<OrderItemInput>>,
    #[serde(rename = "shippingAddress")]
    pub shipping_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderItemInput {
    #[serde(rename = "productId")]
    pub product_id: Option<String>,
    #[serde(rename = "productName")]
    pub product_name: Option<String>,
    pub quantity: Option<i64>,
    #[serde(rename = "unitPrice")]
    pub unit_price: Option<f64>,
}

/// Request body for updating order status.
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: Option<String>,
}

/// Response for customer order summary.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct CustomerSummary {
    #[serde(rename = "customerId")]
    pub customer_id: String,
    #[serde(rename = "totalOrders")]
    pub total_orders: i64,
    #[serde(rename = "totalSpent")]
    pub total_spent: f64,
    #[serde(rename = "averageOrderValue")]
    pub average_order_value: f64,
}

/// Generic error response.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ErrorResponse {
    pub error: String,
}
