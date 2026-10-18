use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::preflight::{PreflightRequest, PreflightResponse};

/// Maximum number of items allowed in a single batch preflight request.
pub const MAX_BATCH_ITEMS: usize = 100;

/// A single item in a batch preflight request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchPreflightItem {
    pub source_asset: String,
    pub destination_asset: String,
    pub amount_usd: f64,
}

/// Request body for `POST /api/v1/preflight/batch`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchPreflightRequest {
    pub items: Vec<BatchPreflightItem>,
}

/// Result for a single batch item, keyed by its index in the request.
#[derive(Debug, Clone, Serialize)]
pub struct BatchPreflightResult {
    pub index: usize,
    pub result: Option<PreflightResponse>,
    pub error: Option<String>,
}

/// Response body for `POST /api/v1/preflight/batch`.
#[derive(Debug, Clone, Serialize)]
pub struct BatchPreflightResponse {
    pub results: Vec<BatchPreflightResult>,
}

/// Error returned when a batch request is invalid.
#[derive(Debug, Clone)]
pub struct BatchValidationError {
    pub message: String,
}

impl BatchValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// A snapshot of the corridor table, reused across all items in a batch so
/// that every item is evaluated against a consistent view of the data.
#[derive(Debug, Clone, Default)]
pub struct CorridorSnapshot {
    /// Map of `(source_asset, destination_asset)` to a corridor identifier.
    corridors: HashMap<(String, String), String>,
}

impl CorridorSnapshot {
    pub fn new(corridors: HashMap<(String, String), String>) -> Self {
        Self { corridors }
    }

    pub fn lookup(&self, source_asset: &str, destination_asset: &str) -> Option<&String> {
        self.corridors
            .get(&(source_asset.to_string(), destination_asset.to_string()))
    }
}

/// Validate a batch request at the boundary.
pub fn validate_batch(request: &BatchPreflightRequest) -> Result<(), BatchValidationError> {
    if request.items.is_empty() {
        return Err(BatchValidationError::new(
            "batch request must contain at least one item",
        ));
    }
    if request.items.len() > MAX_BATCH_ITEMS {
        return Err(BatchValidationError::new(format!(
            "batch request exceeds maximum of {} items",
            MAX_BATCH_ITEMS
        )));
    }
    for (index, item) in request.items.iter().enumerate() {
        if item.source_asset.trim().is_empty() {
            return Err(BatchValidationError::new(format!(
                "item {}: source_asset must not be empty",
                index
            )));
        }
        if item.destination_asset.trim().is_empty() {
            return Err(BatchValidationError::new(format!(
                "item {}: destination_asset must not be empty",
                index
            )));
        }
        if !item.amount_usd.is_finite() || item.amount_usd <= 0.0 {
            return Err(BatchValidationError::new(format!(
                "item {}: amount_usd must be a positive number",
                index
            )));
        }
    }
    Ok(())
}

/// Evaluate a batch of preflight items against a single corridor snapshot.
///
/// The `evaluate` closure performs the per-item preflight computation using
/// the shared snapshot, keeping this module free of HTTP concerns.
pub fn run_batch<F>(
    request: &BatchPreflightRequest,
    snapshot: &CorridorSnapshot,
    mut evaluate: F,
) -> BatchPreflightResponse
where
    F: FnMut(&BatchPreflightItem, &CorridorSnapshot) -> Result<PreflightResponse, String>,
{
    let results = request
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| match evaluate(item, snapshot) {
            Ok(result) => BatchPreflightResult {
                index,
                result: Some(result),
                error: None,
            },
            Err(error) => BatchPreflightResult {
                index,
                result: None,
                error: Some(error),
            },
        })
        .collect();

    BatchPreflightResponse { results }
}

/// Convert a batch item into a single preflight request.
impl From<&BatchPreflightItem> for PreflightRequest {
    fn from(item: &BatchPreflightItem) -> Self {
        PreflightRequest {
            source_asset: item.source_asset.clone(),
            destination_asset: item.destination_asset.clone(),
            amount_usd: item.amount_usd,
        }
    }
}
