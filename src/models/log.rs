use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Log {
    pub level: String,
    pub message: String,
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    #[serde(rename = "spanId")]
    pub span_id: String,
    pub commit: String,
    #[sqlx(skip)]
    pub metadata: LogMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LogMetadata {
    #[serde(rename = "parentResourceId")]
    pub parent_resource_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogQuery {
    pub level: Option<String>,
    pub message: Option<String>,
    pub resource_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub commit: Option<String>,
    pub parent_resource_id: Option<String>,
    pub use_regex: Option<bool>,
} 