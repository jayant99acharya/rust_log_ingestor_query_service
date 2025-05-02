use actix_web::{web, HttpResponse, Responder};
use crate::models::log::{Log, LogQuery};
use crate::db::Database;
use crate::error::Result;

pub mod ui;

pub async fn ingest_log(
    log: web::Json<Log>,
    db: web::Data<Database>,
) -> impl Responder {
    match db.insert_log(&log).await {
        Ok(_) => HttpResponse::Created().json(log.0),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn query_logs(
    query: web::Json<LogQuery>,
    db: web::Data<Database>,
) -> impl Responder {
    match db.query_logs(&query).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
} 