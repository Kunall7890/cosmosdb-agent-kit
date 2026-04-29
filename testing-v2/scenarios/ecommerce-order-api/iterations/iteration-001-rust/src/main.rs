mod cosmos;
mod handlers;
mod models;

use axum::{
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let endpoint = std::env::var("COSMOS_ENDPOINT")
        .unwrap_or_else(|_| "https://localhost:8081".to_string());
    let key = std::env::var("COSMOS_KEY").unwrap_or_else(|_| {
        "C2y6yDjf5/R+ob0N8A7Cgv30VRDJIWEHLM+4QDU5DE2nQ9nDuVTqobD4b8mGGyPMbIZnqyMsEcaGQy67XIw/Jw==".to_string()
    });

    let cosmos = Arc::new(cosmos::CosmosClient::new(
        &endpoint,
        &key,
        "ecommerce-order-api",
        "orders",
    ));

    // Initialize database/container in background so health responds immediately
    let cosmos_init = cosmos.clone();
    tokio::spawn(async move {
        if let Err(e) = cosmos_init.init().await {
            eprintln!("Cosmos DB init error: {}", e);
        } else {
            println!("Cosmos DB initialized successfully");
        }
    });

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/orders", post(handlers::create_order).get(handlers::list_orders))
        .route("/api/orders/:orderId", get(handlers::get_order).delete(handlers::delete_order))
        .route("/api/orders/:orderId/status", patch(handlers::update_order_status))
        .route(
            "/api/customers/:customerId/orders",
            get(handlers::get_customer_orders),
        )
        .route(
            "/api/customers/:customerId/orders/summary",
            get(handlers::get_customer_summary),
        )
        .with_state(cosmos);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = format!("0.0.0.0:{}", port);
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}
