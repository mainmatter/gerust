use axum::body::Bytes;
use axum::response::Response;
use axum::{body::Body, http::Method};
use pacesetter::test::helpers::{request, teardown, TestContext};
use pacesetter_procs::test;
use std::collections::HashMap;

mod common;

#[test]
async fn test_hello(context: &TestContext) {
    let response = request(
        &context.app,
        "/example",
        HashMap::new(),
        Body::empty(),
        Method::GET,
    )
    .await;

    let body = response_body(response).await;
    assert_eq!(&body[..], b"<h1>Hello, World!</h1>");
}

async fn response_body(response: Response<Body>) -> Bytes {
    // We don't care about the size limit in tests.
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body")
}
