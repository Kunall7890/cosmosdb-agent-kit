use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::cosmos::SharedCosmos;
use crate::models::*;

// ── Health ───────────────────────────────────────────────────────────────────

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// ── Create Order ─────────────────────────────────────────────────────────────

pub async fn create_order(
    State(cosmos): State<SharedCosmos>,
    Json(body): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate customerId
    let customer_id = match &body.customer_id {
        Some(id) if !id.trim().is_empty() => id.trim().to_string(),
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "customerId is required" })))),
    };

    // Validate items
    let items_input = match &body.items {
        Some(items) if !items.is_empty() => items,
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "items must be a non-empty array" })))),
    };

    let mut items = Vec::new();
    for item in items_input {
        let product_id = item.product_id.as_deref().unwrap_or("").trim();
        let product_name = item.product_name.as_deref().unwrap_or("").trim();
        let quantity = item.quantity.unwrap_or(0);
        let unit_price = item.unit_price.unwrap_or(0.0);

        if quantity < 1 {
            return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "item quantity must be at least 1" }))));
        }
        if unit_price <= 0.0 {
            return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "item unitPrice must be greater than 0" }))));
        }

        items.push(OrderItem {
            product_id: product_id.to_string(),
            product_name: product_name.to_string(),
            quantity,
            unit_price,
        });
    }

    let total: f64 = items.iter().map(|i| i.quantity as f64 * i.unit_price).sum();
    let order_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let order = Order {
        id: order_id.clone(),
        order_id: order_id.clone(),
        customer_id: customer_id.clone(),
        items,
        total,
        status: "pending".to_string(),
        created_at,
        shipping_address: body.shipping_address.clone(),
    };

    cosmos.create_document(&order).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e })))
    })?;

    let response = order_to_json(&order);
    Ok((StatusCode::CREATED, Json(response)))
}

// ── Get Order by ID ──────────────────────────────────────────────────────────

pub async fn get_order(
    State(cosmos): State<SharedCosmos>,
    Path(order_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // We don't know the partition key (customerId) for an orderId lookup,
    // so query across partitions.
    let orders = cosmos
        .query_documents(
            "SELECT * FROM c WHERE c.orderId = @orderId",
            vec![json!({ "name": "@orderId", "value": order_id })],
            None,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))?;

    match orders.into_iter().next() {
        Some(order) => Ok(Json(order_to_json(&order))),
        None => Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Order not found" })))),
    }
}

// ── List Orders (with optional status / date filters) ────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct ListOrdersQuery {
    pub status: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}

pub async fn list_orders(
    State(cosmos): State<SharedCosmos>,
    Query(params): Query<ListOrdersQuery>,
) -> Result<Json<Vec<Value>>, (StatusCode, Json<Value>)> {
    let mut query = "SELECT * FROM c WHERE 1=1".to_string();
    let mut parameters: Vec<Value> = Vec::new();

    if let Some(ref status) = params.status {
        query.push_str(" AND c.status = @status");
        parameters.push(json!({ "name": "@status", "value": status }));
    }
    if let Some(ref start_date) = params.start_date {
        query.push_str(" AND c.createdAt >= @startDate");
        parameters.push(json!({ "name": "@startDate", "value": start_date }));
    }
    if let Some(ref end_date) = params.end_date {
        query.push_str(" AND c.createdAt <= @endDate");
        parameters.push(json!({ "name": "@endDate", "value": end_date }));
    }

    let orders = cosmos
        .query_documents(&query, parameters, None)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))?;

    let results: Vec<Value> = orders.iter().map(order_to_json).collect();
    Ok(Json(results))
}

// ── Get Customer Orders ──────────────────────────────────────────────────────

pub async fn get_customer_orders(
    State(cosmos): State<SharedCosmos>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<Value>>, (StatusCode, Json<Value>)> {
    // Partition-scoped query — efficient single-partition read
    let orders = cosmos
        .query_documents(
            "SELECT * FROM c WHERE c.customerId = @customerId",
            vec![json!({ "name": "@customerId", "value": customer_id })],
            Some(&customer_id),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))?;

    let results: Vec<Value> = orders.iter().map(order_to_json).collect();
    Ok(Json(results))
}

// ── Customer Order Summary ───────────────────────────────────────────────────

pub async fn get_customer_summary(
    State(cosmos): State<SharedCosmos>,
    Path(customer_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Single aggregate query scoped to the customer partition
    let result = cosmos
        .query_aggregate(
            "SELECT COUNT(1) AS totalOrders, SUM(c.total) AS totalSpent FROM c WHERE c.customerId = @cid",
            vec![json!({ "name": "@cid", "value": customer_id })],
            Some(&customer_id),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))?;

    let docs = result.get("Documents").and_then(|v| v.as_array());
    let (total_orders, total_spent) = if let Some(arr) = docs {
        if let Some(doc) = arr.first() {
            let orders = doc.get("totalOrders").and_then(|v| v.as_i64()).unwrap_or(0);
            let spent = doc.get("totalSpent").and_then(|v| v.as_f64()).unwrap_or(0.0);
            (orders, spent)
        } else {
            (0, 0.0)
        }
    } else {
        (0, 0.0)
    };

    let avg = if total_orders > 0 {
        total_spent / total_orders as f64
    } else {
        0.0
    };

    Ok(Json(json!({
        "customerId": customer_id,
        "totalOrders": total_orders,
        "totalSpent": total_spent,
        "averageOrderValue": avg
    })))
}

// ── Update Order Status ──────────────────────────────────────────────────────

pub async fn update_order_status(
    State(cosmos): State<SharedCosmos>,
    Path(order_id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let new_status = match &body.status {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "status is required" })))),
    };

    // Look up the order (cross-partition since we only have orderId)
    let orders = cosmos
        .query_documents(
            "SELECT * FROM c WHERE c.orderId = @orderId",
            vec![json!({ "name": "@orderId", "value": order_id })],
            None,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))?;

    let mut order = match orders.into_iter().next() {
        Some(o) => o,
        None => return Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Order not found" })))),
    };

    // Validate transition
    let valid = matches!(
        (order.status.as_str(), new_status.as_str()),
        ("pending", "shipped") | ("pending", "cancelled") | ("shipped", "delivered")
    );
    if !valid {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": format!("Invalid status transition from {} to {}", order.status, new_status) })),
        ));
    }

    order.status = new_status;
    cosmos.replace_document(&order).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e })))
    })?;

    Ok(Json(order_to_json(&order)))
}

// ── Delete Order ─────────────────────────────────────────────────────────────

pub async fn delete_order(
    State(cosmos): State<SharedCosmos>,
    Path(order_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    // Look up order first
    let orders = cosmos
        .query_documents(
            "SELECT * FROM c WHERE c.orderId = @orderId",
            vec![json!({ "name": "@orderId", "value": order_id })],
            None,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))?;

    let order = match orders.into_iter().next() {
        Some(o) => o,
        None => return Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Order not found" })))),
    };

    if order.status != "pending" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "Only orders with status 'pending' can be deleted" })),
        ));
    }

    cosmos
        .delete_document(&order.id, &order.customer_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Helper ───────────────────────────────────────────────────────────────────

fn order_to_json(order: &Order) -> Value {
    let mut v = json!({
        "orderId": order.order_id,
        "customerId": order.customer_id,
        "items": order.items.iter().map(|i| json!({
            "productId": i.product_id,
            "productName": i.product_name,
            "quantity": i.quantity,
            "unitPrice": i.unit_price
        })).collect::<Vec<_>>(),
        "total": order.total,
        "status": order.status,
        "createdAt": order.created_at
    });
    if let Some(ref addr) = order.shipping_address {
        v.as_object_mut().unwrap().insert("shippingAddress".to_string(), json!(addr));
    }
    v
}
