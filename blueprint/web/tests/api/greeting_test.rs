use crate::common::{request, response_body_json, TestContext};
use axum::{body::Body, http::Method};
use {{crate_name}}_web::controllers::greeting::Greeting;
use {{crate_name}}_macros::test;
use std::collections::HashMap;

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
