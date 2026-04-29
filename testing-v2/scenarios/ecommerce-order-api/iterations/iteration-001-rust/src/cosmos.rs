use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;
use std::sync::Arc;

use crate::models::Order;

const API_VERSION: &str = "2018-12-31";

/// Cosmos DB REST API client using Gateway mode (HTTPS) with HMAC-SHA256 auth.
#[derive(Clone)]
pub struct CosmosClient {
    endpoint: String,
    key: String,
    database: String,
    container: String,
    http: Client,
}

impl CosmosClient {
    pub fn new(endpoint: &str, key: &str, database: &str, container: &str) -> Self {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");

        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            key: key.to_string(),
            database: database.to_string(),
            container: container.to_string(),
            http,
        }
    }

    /// Generate the Cosmos DB authorization token via HMAC-SHA256.
    fn auth_token(&self, verb: &str, resource_type: &str, resource_link: &str, date: &str) -> String {
        let payload = format!(
            "{}\n{}\n{}\n{}\n\n",
            verb.to_lowercase(),
            resource_type.to_lowercase(),
            resource_link,
            date.to_lowercase()
        );

        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.key)
            .expect("invalid base64 key");

        let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes).expect("HMAC key error");
        mac.update(payload.as_bytes());
        let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        let token = format!("type=master&ver=1.0&sig={}", signature);
        urlencoding::encode(&token).to_string()
    }

    /// Format date per RFC 7231 for x-ms-date header.
    fn rfc7231_date() -> String {
        chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }

    /// Initialize database and container (create if not exists).
    pub async fn init(&self) -> Result<(), String> {
        // Create database
        let date = Self::rfc7231_date();
        let auth = self.auth_token("post", "dbs", "", &date);

        let body = serde_json::json!({ "id": self.database });
        let url = format!("{}/dbs", self.endpoint);

        let resp = self.http.post(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("create db request failed: {}", e))?;

        let status = resp.status().as_u16();
        if status != 201 && status != 409 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("create db failed ({}): {}", status, text));
        }

        // Create container with /customerId as partition key
        let date = Self::rfc7231_date();
        let resource_link = format!("dbs/{}", self.database);
        let auth = self.auth_token("post", "colls", &resource_link, &date);

        let body = serde_json::json!({
            "id": self.container,
            "partitionKey": {
                "paths": ["/customerId"],
                "kind": "Hash",
                "version": 2
            }
        });
        let url = format!("{}/dbs/{}/colls", self.endpoint, self.database);

        let resp = self.http.post(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("create container request failed: {}", e))?;

        let status = resp.status().as_u16();
        if status != 201 && status != 409 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("create container failed ({}): {}", status, text));
        }

        Ok(())
    }

    /// Create a document in the container.
    pub async fn create_document(&self, order: &Order) -> Result<(), String> {
        let date = Self::rfc7231_date();
        let resource_link = format!("dbs/{}/colls/{}", self.database, self.container);
        let auth = self.auth_token("post", "docs", &resource_link, &date);

        let url = format!("{}/dbs/{}/colls/{}/docs", self.endpoint, self.database, self.container);

        let resp = self.http.post(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("Content-Type", "application/json")
            .header("x-ms-documentdb-partitionkey", format!("[\"{}\"]", order.customer_id))
            .json(order)
            .send()
            .await
            .map_err(|e| format!("create doc failed: {}", e))?;

        let status = resp.status().as_u16();
        if status != 201 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("create doc failed ({}): {}", status, text));
        }

        Ok(())
    }

    /// Point-read a document by id and partition key (customerId).
    #[allow(dead_code)]
    pub async fn read_document(&self, doc_id: &str, partition_key: &str) -> Result<Option<Order>, String> {
        let date = Self::rfc7231_date();
        let resource_link = format!("dbs/{}/colls/{}/docs/{}", self.database, self.container, doc_id);
        let auth = self.auth_token("get", "docs", &resource_link, &date);

        let url = format!("{}/{}", self.endpoint, resource_link);

        let resp = self.http.get(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("x-ms-documentdb-partitionkey", format!("[\"{}\"]", partition_key))
            .send()
            .await
            .map_err(|e| format!("read doc failed: {}", e))?;

        let status = resp.status().as_u16();
        if status == 404 {
            return Ok(None);
        }
        if status != 200 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("read doc failed ({}): {}", status, text));
        }

        let order: Order = resp.json().await.map_err(|e| format!("parse doc: {}", e))?;
        Ok(Some(order))
    }

    /// Replace (update) a document.
    pub async fn replace_document(&self, order: &Order) -> Result<(), String> {
        let date = Self::rfc7231_date();
        let resource_link = format!("dbs/{}/colls/{}/docs/{}", self.database, self.container, order.id);
        let auth = self.auth_token("put", "docs", &resource_link, &date);

        let url = format!("{}/{}", self.endpoint, resource_link);

        let resp = self.http.put(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("Content-Type", "application/json")
            .header("x-ms-documentdb-partitionkey", format!("[\"{}\"]", order.customer_id))
            .json(order)
            .send()
            .await
            .map_err(|e| format!("replace doc failed: {}", e))?;

        let status = resp.status().as_u16();
        if status != 200 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("replace doc failed ({}): {}", status, text));
        }

        Ok(())
    }

    /// Delete a document by id and partition key.
    pub async fn delete_document(&self, doc_id: &str, partition_key: &str) -> Result<bool, String> {
        let date = Self::rfc7231_date();
        let resource_link = format!("dbs/{}/colls/{}/docs/{}", self.database, self.container, doc_id);
        let auth = self.auth_token("delete", "docs", &resource_link, &date);

        let url = format!("{}/{}", self.endpoint, resource_link);

        let resp = self.http.delete(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("x-ms-documentdb-partitionkey", format!("[\"{}\"]", partition_key))
            .send()
            .await
            .map_err(|e| format!("delete doc failed: {}", e))?;

        let status = resp.status().as_u16();
        if status == 404 {
            return Ok(false);
        }
        if status != 204 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("delete doc failed ({}): {}", status, text));
        }

        Ok(true)
    }

    /// Execute a SQL query against the container. Uses cross-partition query
    /// when no partition key is provided.
    pub async fn query_documents(&self, query: &str, parameters: Vec<Value>, partition_key: Option<&str>) -> Result<Vec<Order>, String> {
        let date = Self::rfc7231_date();
        let resource_link = format!("dbs/{}/colls/{}", self.database, self.container);
        let auth = self.auth_token("post", "docs", &resource_link, &date);

        let url = format!("{}/dbs/{}/colls/{}/docs", self.endpoint, self.database, self.container);

        let body = serde_json::json!({
            "query": query,
            "parameters": parameters
        });

        let mut req = self.http.post(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("Content-Type", "application/query+json")
            .header("x-ms-documentdb-isquery", "true");

        if let Some(pk) = partition_key {
            req = req.header("x-ms-documentdb-partitionkey", format!("[\"{}\"]", pk));
        } else {
            req = req.header("x-ms-documentdb-query-enablecrosspartition", "true");
        }

        let resp = req.json(&body)
            .send()
            .await
            .map_err(|e| format!("query failed: {}", e))?;

        let status = resp.status().as_u16();
        if status != 200 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("query failed ({}): {}", status, text));
        }

        let result: Value = resp.json().await.map_err(|e| format!("parse query result: {}", e))?;
        let docs = result.get("Documents")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let orders: Vec<Order> = docs.into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        Ok(orders)
    }

    /// Query that returns a single aggregate value (for summary queries).
    pub async fn query_aggregate(&self, query: &str, parameters: Vec<Value>, partition_key: Option<&str>) -> Result<Value, String> {
        let date = Self::rfc7231_date();
        let resource_link = format!("dbs/{}/colls/{}", self.database, self.container);
        let auth = self.auth_token("post", "docs", &resource_link, &date);

        let url = format!("{}/dbs/{}/colls/{}/docs", self.endpoint, self.database, self.container);

        let body = serde_json::json!({
            "query": query,
            "parameters": parameters
        });

        let mut req = self.http.post(&url)
            .header("Authorization", &auth)
            .header("x-ms-date", &date)
            .header("x-ms-version", API_VERSION)
            .header("Content-Type", "application/query+json")
            .header("x-ms-documentdb-isquery", "true");

        if let Some(pk) = partition_key {
            req = req.header("x-ms-documentdb-partitionkey", format!("[\"{}\"]", pk));
        } else {
            req = req.header("x-ms-documentdb-query-enablecrosspartition", "true");
        }

        let resp = req.json(&body)
            .send()
            .await
            .map_err(|e| format!("aggregate query failed: {}", e))?;

        let status = resp.status().as_u16();
        if status != 200 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("aggregate query failed ({}): {}", status, text));
        }

        let result: Value = resp.json().await.map_err(|e| format!("parse aggregate: {}", e))?;
        Ok(result)
    }
}

pub type SharedCosmos = Arc<CosmosClient>;
