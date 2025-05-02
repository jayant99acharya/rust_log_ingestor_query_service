use crate::db::Database;
use crate::models::log::{Log, LogMetadata, LogQuery};
use chrono::Utc;
use std::time::Instant;
use futures::future::join_all;
use std::pin::Pin;
use std::future::Future;

#[cfg(test)]
mod performance_tests {
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

    fn create_test_log(index: usize) -> Log {
        Log {
            level: ["info", "warning", "error", "debug"][index % 4].to_string(),
            message: format!("Test log message {}", index),
            resource_id: format!("server-{}", index % 10),
            timestamp: Utc::now(),
            trace_id: format!("trace-{}", index % 5),
            span_id: format!("span-{}", index % 5),
            commit: format!("commit-{}", index % 3),
            metadata: LogMetadata {
                parent_resource_id: format!("parent-{}", index % 3),
            },
        }
    }

    #[actix_rt::test]
    #[ignore] // Ignore by default as this is a performance test
    async fn test_bulk_insert_performance() {
        let db = setup_test_db().await;
        
        // Number of logs to insert
        const NUM_LOGS: usize = 1000;
        
        // Create logs
        let logs: Vec<Log> = (0..NUM_LOGS).map(create_test_log).collect();
        
        // Measure time to insert logs
        let start = Instant::now();
        
        // Insert logs concurrently in batches
        const BATCH_SIZE: usize = 100;
        let mut futures = Vec::new();
        
        for chunk in logs.chunks(BATCH_SIZE) {
            let db_clone = db.clone();
            let chunk_vec = chunk.to_vec();
            
            let future = async move {
                for log in chunk_vec {
                    db_clone.insert_log(&log).await.unwrap();
                }
            };
            
            futures.push(Box::pin(future) as Pin<Box<dyn Future<Output = ()> + Send>>);
        }
        
        join_all(futures).await;
        
        let duration = start.elapsed();
        println!("Inserted {} logs in {:?}", NUM_LOGS, duration);
        println!("Average time per log: {:?}", duration / NUM_LOGS as u32);
        
        // Assert that the operation completes within a reasonable time
        // This threshold may need adjustment based on the specific environment
        assert!(duration.as_secs() < 60, "Bulk insert took too long: {:?}", duration);
    }

    #[actix_rt::test]
    #[ignore] // Ignore by default as this is a performance test
    async fn test_query_performance() {
        let db = setup_test_db().await;
        
        // First, insert some test data
        const NUM_LOGS: usize = 1000;
        let logs: Vec<Log> = (0..NUM_LOGS).map(create_test_log).collect();
        
        for log in &logs {
            db.insert_log(log).await.unwrap();
        }
        
        // Test query performance for different query types
        let queries = [
            // Query by level
            LogQuery {
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
            },
            // Query by message with regex
            LogQuery {
                level: None,
                message: Some("message \\d+".to_string()),
                resource_id: None,
                start_time: None,
                end_time: None,
                trace_id: None,
                span_id: None,
                commit: None,
                parent_resource_id: None,
                use_regex: Some(true),
            },
            // Query by resource_id
            LogQuery {
                level: None,
                message: None,
                resource_id: Some("server-5".to_string()),
                start_time: None,
                end_time: None,
                trace_id: None,
                span_id: None,
                commit: None,
                parent_resource_id: None,
                use_regex: None,
            },
            // Complex query with multiple filters
            LogQuery {
                level: Some("info".to_string()),
                message: None,
                resource_id: Some("server-1".to_string()),
                start_time: Some(Utc::now() - chrono::Duration::days(1)),
                end_time: Some(Utc::now() + chrono::Duration::days(1)),
                trace_id: None,
                span_id: None,
                commit: None,
                parent_resource_id: None,
                use_regex: None,
            },
        ];
        
        for (i, query) in queries.iter().enumerate() {
            let start = Instant::now();
            let results = db.query_logs(query).await.unwrap();
            let duration = start.elapsed();
            
            println!("Query {} returned {} results in {:?}", i + 1, results.len(), duration);
            
            // Assert that queries complete within a reasonable time
            assert!(duration.as_secs() < 5, "Query {} took too long: {:?}", i + 1, duration);
        }
    }

    #[actix_rt::test]
    #[ignore] // Ignore by default as this is a performance test
    async fn test_concurrent_operations() {
        let db = setup_test_db().await;
        
        // Create a mix of insert and query operations
        const NUM_OPERATIONS: usize = 100;
        
        let start = Instant::now();
        
        let mut futures: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = Vec::new();
        
        for i in 0..NUM_OPERATIONS {
            let db_clone = db.clone();
            
            if i % 2 == 0 {
                // Insert operation
                let log = create_test_log(i);
                let future = async move {
                    db_clone.insert_log(&log).await.unwrap();
                };
                futures.push(Box::pin(future));
            } else {
                // Query operation
                let query = LogQuery {
                    level: Some(["info", "warning", "error"][i % 3].to_string()),
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
                
                let future = async move {
                    db_clone.query_logs(&query).await.unwrap();
                };
                futures.push(Box::pin(future));
            }
        }
        
        join_all(futures).await;
        
        let duration = start.elapsed();
        println!("Completed {} concurrent operations in {:?}", NUM_OPERATIONS, duration);
        println!("Average time per operation: {:?}", duration / NUM_OPERATIONS as u32);
        
        // Assert that the operations complete within a reasonable time
        assert!(duration.as_secs() < 30, "Concurrent operations took too long: {:?}", duration);
    }
}
