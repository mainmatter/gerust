use axum::{body::Body, http::Method};
use {{crate_name}}_web::controllers::example::Message;
use pacesetter::test::helpers::response_body_json;
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

    let message: Message = response_body_json(response).await;
    assert_eq!(message.hello, String::from("world"));
}
