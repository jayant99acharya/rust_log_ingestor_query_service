# Log Ingestor and Query Service

A high-performance log ingestion and query service built with Rust, featuring efficient log storage and retrieval capabilities.

## Features

- Fast log ingestion via HTTP API
- Efficient full-text search across logs
- Filtering by multiple fields:
  - Log level
  - Message content
  - Resource ID
  - Timestamp
  - Trace ID
  - Span ID
  - Commit hash
  - Parent Resource ID
- PostgreSQL database for reliable storage
- RESTful API endpoints

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- Docker (optional, for running PostgreSQL)

## Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd log_ingestor_query_service
```

2. Create a `.env` file in the project root:
```bash
DATABASE_URL=postgres://username:password@localhost:5432/logs_db
```

3. Set up the database:
```bash
createdb logs_db
```

4. Build and run the project:
```bash
cargo build --release
cargo run
```

The server will start on `http://localhost:3000`.

## API Endpoints

### Ingest Logs
- **POST** `/api/logs`
- Request body:
```json
{
    "level": "error",
    "message": "Failed to connect to DB",
    "resourceId": "server-1234",
    "timestamp": "2023-09-15T08:00:00Z",
    "traceId": "abc-xyz-123",
    "spanId": "span-456",
    "commit": "5e5342f",
    "metadata": {
        "parentResourceId": "server-0987"
    }
}
```

### Query Logs
- **POST** `/api/logs/query`
- Request body:
```json
{
    "level": "error",
    "message": "Failed to connect",
    "resourceId": "server-1234",
    "startTime": "2023-09-10T00:00:00Z",
    "endTime": "2023-09-15T23:59:59Z"
}
```

## Performance Considerations

- The system uses connection pooling for efficient database access
- Indexes are created on frequently queried fields
- Asynchronous processing for better throughput
- Efficient query building with parameterized queries

## Future Improvements

- [ ] Add authentication and authorization
- [ ] Implement rate limiting
- [ ] Add metrics and monitoring
- [ ] Support for bulk log ingestion
- [ ] Add caching layer
- [ ] Implement log rotation and retention policies

## License

MIT 