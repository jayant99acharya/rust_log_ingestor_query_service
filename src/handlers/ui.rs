use actix_web::{web, HttpResponse, Responder};
use crate::db::Database;

pub async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Log Query Interface</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                .container { max-width: 800px; margin: 0 auto; }
                .form-group { margin-bottom: 15px; }
                label { display: block; margin-bottom: 5px; }
                input, select { width: 100%; padding: 8px; margin-bottom: 10px; }
                button { padding: 10px 20px; background-color: #007bff; color: white; border: none; cursor: pointer; }
                .results { margin-top: 20px; }
                .log-entry { border: 1px solid #ddd; padding: 10px; margin-bottom: 10px; }
                .checkbox-group { display: flex; align-items: center; }
                .checkbox-group input[type="checkbox"] { width: auto; margin-right: 10px; }
                .checkbox-group label { margin-bottom: 0; }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Log Query Interface</h1>
                <form id="queryForm">
                    <div class="form-group">
                        <label for="level">Level:</label>
                        <select id="level" name="level">
                            <option value="">Any</option>
                            <option value="error">Error</option>
                            <option value="info">Info</option>
                            <option value="warn">Warn</option>
                            <option value="debug">Debug</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="message">Message:</label>
                        <input type="text" id="message" name="message">
                    </div>
                    <div class="form-group checkbox-group">
                        <input type="checkbox" id="useRegex" name="useRegex">
                        <label for="useRegex">Use Regex for Message Search</label>
                    </div>
                    <div class="form-group">
                        <label for="resourceId">Resource ID:</label>
                        <input type="text" id="resourceId" name="resourceId">
                    </div>
                    <div class="form-group">
                        <label for="startTime">Start Time:</label>
                        <input type="datetime-local" id="startTime" name="startTime">
                    </div>
                    <div class="form-group">
                        <label for="endTime">End Time:</label>
                        <input type="datetime-local" id="endTime" name="endTime">
                    </div>
                    <button type="submit">Search</button>
                </form>
                <div id="results" class="results"></div>
            </div>
            <script>
                document.getElementById('queryForm').addEventListener('submit', async (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.target);
                    const query = {};
                    for (let [key, value] of formData.entries()) {
                        if (value) query[key] = value;
                    }
                    
                    try {
                        const response = await fetch('/api/logs/query', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify(query)
                        });
                        const logs = await response.json();
                        const resultsDiv = document.getElementById('results');
                        resultsDiv.innerHTML = logs.map(log => `
                            <div class="log-entry">
                                <strong>Level:</strong> ${log.level}<br>
                                <strong>Message:</strong> ${log.message}<br>
                                <strong>Resource ID:</strong> ${log.resource_id}<br>
                                <strong>Timestamp:</strong> ${log.timestamp}<br>
                                <strong>Trace ID:</strong> ${log.trace_id}<br>
                                <strong>Span ID:</strong> ${log.span_id}<br>
                                <strong>Commit:</strong> ${log.commit}<br>
                                <strong>Parent Resource ID:</strong> ${log.metadata.parent_resource_id}
                            </div>
                        `).join('');
                    } catch (error) {
                        console.error('Error:', error);
                        document.getElementById('results').innerHTML = '<p style="color: red">Error fetching results</p>';
                    }
                });
            </script>
        </body>
        </html>
    "#)
} 