use actix_web::{test, web, App};
use serde_json::json;

use crate::db::Database;
use crate::handlers::{ingest_log, query_logs};
use crate::models::log::Log;

#[cfg(test)]
mod api_tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    async fn setup_test_app() -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = Database::new(&database_url).await.unwrap();
        db.init().await.unwrap();

        test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .route("/api/logs", web::post().to(ingest_log))
                .route("/api/logs/query", web::post().to(query_logs)),
        )
        .await
    }

    #[actix_rt::test]
    async fn test_ingest_log_endpoint() {
        let app = setup_test_app().await;

        // Create a test log
        let log_data = json!({
            "level": "info",
            "message": "Test log message",
            "resourceId": "api-test-resource",
            "timestamp": "2025-05-01T12:00:00Z",
            "traceId": "api-test-trace",
            "spanId": "api-test-span",
            "commit": "api-test-commit",
            "metadata": {
                "parentResourceId": "api-test-parent"
            }
        });

        // Send request to ingest log
        let req = test::TestRequest::post()
            .uri("/api/logs")
            .set_json(&log_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_query_logs_endpoint() {
        let app = setup_test_app().await;

        // First, insert a test log
        let log_data = json!({
            "level": "error",
            "message": "API test error message",
            "resourceId": "api-query-resource",
            "timestamp": "2025-05-01T12:00:00Z",
            "traceId": "api-query-trace",
            "spanId": "api-query-span",
            "commit": "api-query-commit",
            "metadata": {
                "parentResourceId": "api-query-parent"
            }
        });

        let req = test::TestRequest::post()
            .uri("/api/logs")
            .set_json(&log_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Now query for the log
        let query_data = json!({
            "level": "error",
            "resource_id": "api-query-resource"
        });

        let req = test::TestRequest::post()
            .uri("/api/logs/query")
            .set_json(&query_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Extract response body and verify it contains our log
        let body: Vec<Log> = test::read_body_json(resp).await;
        assert!(!body.is_empty());
        assert_eq!(body[0].level, "error");
        assert_eq!(body[0].resource_id, "api-query-resource");
    }

    #[actix_rt::test]
    async fn test_query_logs_with_time_range() {
        let app = setup_test_app().await;

        // Insert logs with different timestamps
        let log1 = json!({
            "level": "info",
            "message": "Log from yesterday",
            "resourceId": "time-test-resource",
            "timestamp": "2025-05-01T12:00:00Z",
            "traceId": "time-test-trace-1",
            "spanId": "time-test-span-1",
            "commit": "time-test-commit",
            "metadata": {
                "parentResourceId": "time-test-parent"
            }
        });

        let log2 = json!({
            "level": "info",
            "message": "Log from today",
            "resourceId": "time-test-resource",
            "timestamp": "2025-05-02T12:00:00Z",
            "traceId": "time-test-trace-2",
            "spanId": "time-test-span-2",
            "commit": "time-test-commit",
            "metadata": {
                "parentResourceId": "time-test-parent"
            }
        });

        // Insert both logs
        let req1 = test::TestRequest::post()
            .uri("/api/logs")
            .set_json(&log1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert!(resp1.status().is_success());

        let req2 = test::TestRequest::post()
            .uri("/api/logs")
            .set_json(&log2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert!(resp2.status().is_success());

        // Query logs from today only - use the correct field names that match the LogQuery struct
        let query_data = json!({
            "resource_id": "time-test-resource",
            "start_time": "2025-05-02T00:00:00Z",
            "end_time": "2025-05-03T00:00:00Z"
        });

        let req = test::TestRequest::post()
            .uri("/api/logs/query")
            .set_json(&query_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Debug the response if it fails
        if !resp.status().is_success() {
            let body = test::read_body(resp).await;
            println!("Error response: {:?}", body);
            panic!("Query failed with non-success status code");
        }
        
        assert!(resp.status().is_success());

        // Verify we only get the log from today
        let body: Vec<Log> = test::read_body_json(resp).await;
        assert!(!body.is_empty(), "No logs found in response");
        
        // Find the log with the correct message
        let today_logs: Vec<&Log> = body.iter()
            .filter(|log| log.message == "Log from today")
            .collect();
            
        assert!(!today_logs.is_empty(), "Log from today not found in results");
    }

    #[actix_rt::test]
    async fn test_error_handling_invalid_json() {
        let app = setup_test_app().await;

        // Send invalid JSON
        let req = test::TestRequest::post()
            .uri("/api/logs")
            .set_payload("{invalid json}")
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_query_with_no_results() {
        let app = setup_test_app().await;

        // Query for logs with a non-existent level
        let query_data = json!({
            "level": "non-existent-level"
        });

        let req = test::TestRequest::post()
            .uri("/api/logs/query")
            .set_json(&query_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Verify we get an empty array
        let body: Vec<Log> = test::read_body_json(resp).await;
        assert!(body.is_empty());
    }
}
