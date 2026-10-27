use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use template_rust::api::app_router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint_service_ext() {
    let app = app_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body bytes");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], "0.1.0");
    assert!(json["timestamp"].is_string());
}

#[tokio::test]
async fn test_health_endpoint_axum_serve() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let app = app_router();
    let server_handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("Failed to connect to live server");

    let request = format!("GET /health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("Failed to write request");

    let mut response_buf = Vec::new();
    stream
        .read_to_end(&mut response_buf)
        .await
        .expect("Failed to read response");

    let response_str = String::from_utf8_lossy(&response_buf);
    assert!(response_str.contains("HTTP/1.1 200 OK"));
    assert!(response_str.contains(r#""status":"ok""#));
    assert!(response_str.contains(r#""version":"0.1.0""#));

    server_handle.abort();
}
