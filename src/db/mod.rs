use sqlx::postgres::{PgPool, PgPoolOptions};
use crate::models::log::{Log, LogQuery, LogMetadata};
use crate::error::{Result, AppError};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(Database { pool })
    }

    pub async fn init(&self) -> Result<()> {
        // Create the logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS logs (
                id SERIAL PRIMARY KEY,
                level VARCHAR NOT NULL,
                message TEXT NOT NULL,
                resource_id VARCHAR NOT NULL,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                trace_id VARCHAR NOT NULL,
                span_id VARCHAR NOT NULL,
                commit VARCHAR NOT NULL,
                parent_resource_id VARCHAR NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        // Create indexes for frequently queried fields
        let index_queries = [
            "CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level)",
            "CREATE INDEX IF NOT EXISTS idx_logs_resource_id ON logs(resource_id)",
            "CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_logs_trace_id ON logs(trace_id)",
            "CREATE INDEX IF NOT EXISTS idx_logs_span_id ON logs(span_id)",
            "CREATE INDEX IF NOT EXISTS idx_logs_commit ON logs(commit)",
            "CREATE INDEX IF NOT EXISTS idx_logs_parent_resource_id ON logs(parent_resource_id)",
            "CREATE INDEX IF NOT EXISTS idx_logs_message ON logs USING gin(to_tsvector('english', message))"
        ];

        for query in index_queries.iter() {
            sqlx::query(query)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e))?;
        }

        Ok(())
    }

    pub async fn insert_log(&self, log: &Log) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO logs (level, message, resource_id, timestamp, trace_id, span_id, commit, parent_resource_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&log.level)
        .bind(&log.message)
        .bind(&log.resource_id)
        .bind(log.timestamp)
        .bind(&log.trace_id)
        .bind(&log.span_id)
        .bind(&log.commit)
        .bind(&log.metadata.parent_resource_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    pub async fn query_logs(&self, query: &LogQuery) -> Result<Vec<Log>> {
        let mut sql = String::from("SELECT *, parent_resource_id as \"metadata.parent_resource_id\" FROM logs WHERE 1=1");
        let mut params = Vec::new();
        let mut param_count = 1;

        if let Some(level) = &query.level {
            sql.push_str(&format!(" AND level = ${}", param_count));
            params.push(level.clone());
            param_count += 1;
        }

        if let Some(message) = &query.message {
            if query.use_regex.unwrap_or(false) {
                sql.push_str(&format!(" AND message ~ ${}", param_count));
                params.push(message.clone());
            } else {
                sql.push_str(&format!(" AND message ILIKE ${}", param_count));
                params.push(format!("%{}%", message));
            }
            param_count += 1;
        }

        if let Some(resource_id) = &query.resource_id {
            sql.push_str(&format!(" AND resource_id = ${}", param_count));
            params.push(resource_id.clone());
            param_count += 1;
        }

        if let Some(start_time) = &query.start_time {
            sql.push_str(&format!(" AND timestamp >= ${}", param_count));
            param_count += 1;
        }

        if let Some(end_time) = &query.end_time {
            sql.push_str(&format!(" AND timestamp <= ${}", param_count));
            param_count += 1;
        }

        if let Some(trace_id) = &query.trace_id {
            sql.push_str(&format!(" AND trace_id = ${}", param_count));
            params.push(trace_id.clone());
            param_count += 1;
        }

        if let Some(span_id) = &query.span_id {
            sql.push_str(&format!(" AND span_id = ${}", param_count));
            params.push(span_id.clone());
            param_count += 1;
        }

        if let Some(commit) = &query.commit {
            sql.push_str(&format!(" AND commit = ${}", param_count));
            params.push(commit.clone());
            param_count += 1;
        }

        if let Some(parent_resource_id) = &query.parent_resource_id {
            sql.push_str(&format!(" AND parent_resource_id = ${}", param_count));
            params.push(parent_resource_id.clone());
            param_count += 1;
        }

        let mut query_builder = sqlx::query_as::<_, Log>(&sql);

        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        if let Some(start_time) = &query.start_time {
            query_builder = query_builder.bind(start_time);
        }
        
        if let Some(end_time) = &query.end_time {
            query_builder = query_builder.bind(end_time);
        }

        let mut logs = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        // Set metadata for each log
        for log in &mut logs {
            log.metadata = LogMetadata {
                parent_resource_id: log.metadata.parent_resource_id.clone(),
            };
        }

        Ok(logs)
    }
}