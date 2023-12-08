use axum::{body::Body, http::Method};
use {{crate_name}}_web::controllers::greeting::Greeting;
use pacesetter::test::helpers::response_body_json;
use pacesetter::test::helpers::{request, TestContext};
use pacesetter_procs::test;
use std::collections::HashMap;

mod common;

#[test]
async fn test_hello(context: &TestContext) {
    let response = request(
        &context.app,
        "/greet",
        HashMap::new(),
        Body::empty(),
        Method::GET,
    )
    .await;

    let greeting: Greeting = response_body_json(response).await;
    assert_eq!(greeting.hello, String::from("world"));
}
