use crate::db::Database;
use crate::models::log::{Log, LogMetadata, LogQuery};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    async fn setup_test_db() -> Database {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = Database::new(&database_url).await.unwrap();
        db.init().await.unwrap();
        db
    }

    fn create_test_log(level: &str, message: &str, resource_id: &str) -> Log {
        Log {
            level: level.to_string(),
            message: message.to_string(),
            resource_id: resource_id.to_string(),
            timestamp: Utc::now(),
            trace_id: "test-trace-id".to_string(),
            span_id: "test-span-id".to_string(),
            commit: "test-commit".to_string(),
            metadata: LogMetadata {
                parent_resource_id: "test-parent-resource-id".to_string(),
            },
        }
    }

    #[actix_rt::test]
    async fn test_insert_log() {
        let db = setup_test_db().await;
        
        // Create a test log
        let log = create_test_log("info", "Test message", "test-resource");
        
        // Insert the log
        let result = db.insert_log(&log).await;
        assert!(result.is_ok(), "Failed to insert log: {:?}", result);
    }

    #[actix_rt::test]
    async fn test_query_logs_by_level() {
        let db = setup_test_db().await;
        
        // Create and insert test logs with different levels
        let log1 = create_test_log("error", "Error message", "resource-1");
        let log2 = create_test_log("info", "Info message", "resource-2");
        let log3 = create_test_log("warning", "Warning message", "resource-3");
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        db.insert_log(&log3).await.unwrap();
        
        // Query logs by level
        let query = LogQuery {
            level: Some("error".to_string()),
            message: None,
            resource_id: None,
            start_time: None,
            end_time: None,
            trace_id: None,
            span_id: None,
            commit: None,
            parent_resource_id: None,
            use_regex: None,
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert!(!result.is_empty(), "No logs found");
        assert_eq!(result[0].level, "error");
    }

    #[actix_rt::test]
    async fn test_query_logs_by_message_content() {
        let db = setup_test_db().await;
        
        // Create and insert test logs with different messages
        let log1 = create_test_log("info", "Database connection established", "resource-1");
        let log2 = create_test_log("info", "User login successful", "resource-2");
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        
        // Query logs by message content
        let query = LogQuery {
            level: None,
            message: Some("login".to_string()),
            resource_id: None,
            start_time: None,
            end_time: None,
            trace_id: None,
            span_id: None,
            commit: None,
            parent_resource_id: None,
            use_regex: None,
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert!(!result.is_empty(), "No logs found");
        assert!(result[0].message.contains("login"));
    }

    #[actix_rt::test]
    async fn test_query_logs_by_resource_id() {
        let db = setup_test_db().await;
        
        // Create and insert test logs with different resource IDs
        let log1 = create_test_log("info", "Message 1", "server-123");
        let log2 = create_test_log("info", "Message 2", "server-456");
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        
        // Query logs by resource ID
        let query = LogQuery {
            level: None,
            message: None,
            resource_id: Some("server-123".to_string()),
            start_time: None,
            end_time: None,
            trace_id: None,
            span_id: None,
            commit: None,
            parent_resource_id: None,
            use_regex: None,
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert!(!result.is_empty(), "No logs found");
        assert_eq!(result[0].resource_id, "server-123");
    }

    #[actix_rt::test]
    async fn test_query_logs_by_time_range() {
        let db = setup_test_db().await;
        
        // Create logs with different timestamps
        let mut log1 = create_test_log("info", "Old message", "resource-1");
        let mut log2 = create_test_log("info", "New message", "resource-2");
        
        // Set timestamps 1 day apart
        log1.timestamp = DateTime::from_str("2025-05-01T00:00:00Z").unwrap();
        log2.timestamp = DateTime::from_str("2025-05-02T00:00:00Z").unwrap();
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        
        // Query logs within a specific time range
        let start_time = DateTime::from_str("2025-05-01T12:00:00Z").unwrap();
        let end_time = DateTime::from_str("2025-05-02T12:00:00Z").unwrap();
        
        let query = LogQuery {
            level: None,
            message: None,
            resource_id: None,
            start_time: Some(start_time),
            end_time: Some(end_time),
            trace_id: None,
            span_id: None,
            commit: None,
            parent_resource_id: None,
            use_regex: None,
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert!(!result.is_empty(), "No logs found");
        
        // Find the log with the "New message" text
        let new_logs: Vec<&Log> = result.iter()
            .filter(|log| log.message == "New message")
            .collect();
            
        assert!(!new_logs.is_empty(), "New message log not found in results");
    }

    #[actix_rt::test]
    async fn test_query_logs_with_multiple_filters() {
        let db = setup_test_db().await;
        
        // Create logs with different properties
        let log1 = create_test_log("error", "Database error", "db-server-1");
        let log2 = create_test_log("error", "Network error", "net-server-1");
        let log3 = create_test_log("info", "Database connected", "db-server-1");
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        db.insert_log(&log3).await.unwrap();
        
        // Query logs with multiple filters
        let query = LogQuery {
            level: Some("error".to_string()),
            message: Some("Database".to_string()),
            resource_id: Some("db-server-1".to_string()),
            start_time: None,
            end_time: None,
            trace_id: None,
            span_id: None,
            commit: None,
            parent_resource_id: None,
            use_regex: None,
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert!(!result.is_empty(), "No logs found");
        assert_eq!(result[0].level, "error");
        assert!(result[0].message.contains("Database"));
        assert_eq!(result[0].resource_id, "db-server-1");
    }

    #[actix_rt::test]
    async fn test_query_logs_with_regex() {
        let db = setup_test_db().await;
        
        // Use unique identifiers to avoid conflicts with other tests
        let test_id = Uuid::new_v4().to_string();
        
        // Create logs with different messages
        let log1 = create_test_log("info", &format!("User123 logged in {}", test_id), "auth-server");
        let log2 = create_test_log("info", &format!("User456 logged in {}", test_id), "auth-server");
        let log3 = create_test_log("info", &format!("Admin logged in {}", test_id), "auth-server");
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        db.insert_log(&log3).await.unwrap();
        
        // Query logs using regex
        let query = LogQuery {
            level: None,
            message: Some(format!("User\\d+ logged in {}", test_id)),
            resource_id: None,
            start_time: None,
            end_time: None,
            trace_id: None,
            span_id: None,
            commit: None,
            parent_resource_id: None,
            use_regex: Some(true),
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert_eq!(result.len(), 2, "Expected 2 logs matching the regex pattern");
        
        // Verify both logs contain "User" but not "Admin"
        for log in &result {
            assert!(log.message.contains("User"), "Log should contain 'User'");
            assert!(!log.message.contains("Admin"), "Log should not contain 'Admin'");
        }
    }

    #[actix_rt::test]
    async fn test_query_logs_with_trace_and_span_id() {
        let db = setup_test_db().await;
        
        // Use unique identifiers to avoid conflicts with other tests
        let test_id = Uuid::new_v4().to_string();
        
        // Create logs with different trace and span IDs
        let mut log1 = create_test_log("info", "Operation started", "service-1");
        let mut log2 = create_test_log("info", "Operation in progress", "service-2");
        let mut log3 = create_test_log("info", "Operation completed", "service-3");
        
        log1.trace_id = format!("trace-abc-123-{}", test_id);
        log1.span_id = format!("span-1-{}", test_id);
        
        log2.trace_id = format!("trace-abc-123-{}", test_id);
        log2.span_id = format!("span-2-{}", test_id);
        
        log3.trace_id = format!("trace-def-456-{}", test_id);
        log3.span_id = format!("span-3-{}", test_id);
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        db.insert_log(&log3).await.unwrap();
        
        // Query logs by trace ID
        let query = LogQuery {
            level: None,
            message: None,
            resource_id: None,
            start_time: None,
            end_time: None,
            trace_id: Some(format!("trace-abc-123-{}", test_id)),
            span_id: None,
            commit: None,
            parent_resource_id: None,
            use_regex: None,
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert_eq!(result.len(), 2, "Expected 2 logs with the same trace ID");
        
        // Verify both logs have the correct trace ID
        for log in &result {
            assert_eq!(log.trace_id, format!("trace-abc-123-{}", test_id));
        }
    }

    #[actix_rt::test]
    async fn test_query_logs_with_parent_resource_id() {
        let db = setup_test_db().await;
        
        // Create logs with different parent resource IDs
        let mut log1 = create_test_log("info", "Child service 1 message", "child-1");
        let mut log2 = create_test_log("info", "Child service 2 message", "child-2");
        
        log1.metadata.parent_resource_id = "parent-service-abc".to_string();
        log2.metadata.parent_resource_id = "parent-service-xyz".to_string();
        
        db.insert_log(&log1).await.unwrap();
        db.insert_log(&log2).await.unwrap();
        
        // Query logs by parent resource ID
        let query = LogQuery {
            level: None,
            message: None,
            resource_id: None,
            start_time: None,
            end_time: None,
            trace_id: None,
            span_id: None,
            commit: None,
            parent_resource_id: Some("parent-service-abc".to_string()),
            use_regex: None,
        };
        
        let result = db.query_logs(&query).await.unwrap();
        assert!(!result.is_empty(), "No logs found");
        // The metadata.parent_resource_id is populated from the database's parent_resource_id column
        // We need to check the actual value from the database
        assert!(result[0].resource_id == "child-1", "Expected resource_id to be child-1");
    }
}
