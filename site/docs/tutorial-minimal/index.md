---
sidebar_position: 3
---

# Tutorial: Minimal Project

In this tutorial, we will create a minimal application with simple hello-world endpoints, a visits counter, a rate-limiting middleware, and the respective tests.

A minimal project is a project that does not use a database. It's a simple web server along with code for reading in configuration settings and a CLI for creating e.g. controllers and other project files.

Create a new minimal project with

```
gerust my-app --minimal
```

That creates a basic project structure:

```
.
├── Cargo.toml
├── README.md
├── cli
│   ├── Cargo.toml
│   ├── README.md
│   ├── blueprints
│   │   └── …
│   └── src
│       ├── bin
│       │   └── generate.rs
│       ├── lib.rs
│       └── util
│           ├── mod.rs
│           └── ui.rs
├── config
│   ├── Cargo.toml
│   ├── README.md
│   ├── app.toml
│   ├── environments
│   │   ├── development.toml
│   │   ├── production.toml
│   │   └── test.toml
│   └── src
│       └── lib.rs
├── macros
│   └── …
└── web
    ├── Cargo.toml
    ├── README.md
    ├── src
    │   ├── controllers
    │   │   ├── greeting.rs
    │   │   └── mod.rs
    │   ├── error.rs
    │   ├── lib.rs
    │   ├── main.rs
    │   ├── middlewares
    │   │   └── mod.rs
    │   ├── routes.rs
    │   ├── state.rs
    │   └── test_helpers
    │       └── mod.rs
    └── tests
        └── api
            ├── greeting_test.rs
            └── main.rs

```

The structure contains the `cli`, `config`, `macros`, and `web` crates. The `cli` crate contains a binary for creating project files like controllers or middlewares. The `config` crate contains code for reading configuration values and managing them in a `Config` struct. The `macros` crate contains macros that are used for testing. And the `web` crate contains the system HTTP interface implementation, which essentially is a standard `axum` application. More on the different crates, their purpose and elements, in the [Architecture guide](../architecture).

## Running the Project

Run the generated project with:

```
cargo run
```

That starts up the server at `127.0.0.1:3000`. To verify everything works as intended, request the demo endpoint that Gerust created by default:

```
» curl http://localhost:3000/greet
{"hello":"world"}%
```

The implementation of the endpoint is in `web/src/controllers/greeting.rs`:

```rust
use axum::response::Json;
use serde::{Deserialize, Serialize};

/// A greeting to respond with to the requesting client
#[derive(Deserialize, Serialize)]
pub struct Greeting {
    /// Who do we say hello to?
    pub hello: String,
}

/// Responds with a [`Greeting`], encoded as JSON.
#[axum::debug_handler]
pub async fn hello() -> Json<Greeting> {
    Json(Greeting {
        hello: String::from("world"),
    })
}
```

## Testing the Project

Run the tests with:

```
cargo test
```

Along with the `/greet` demo endpoint, Gerust also created the corresponding test:

```rust
use my_app_web::test_helpers::{BodyExt, RouterExt, TestContext};
use googletest::prelude::*;
use my_app_macros::test;
use my_app_web::controllers::greeting::Greeting;

#[test]
async fn test_hello(context: &TestContext) {
    let response = context.app.request("/greet").send().await;

    let greeting: Greeting = response.into_body().into_json().await;
    assert_that!(greeting.hello, eq(&String::from("world")));
}
```

The project comes preconfigured with a CI setup for GitHub Actions that covers format checking with [`rustfmt`](https://github.com/rust-lang/rustfmt) and [`clippy`](https://github.com/rust-lang/rust-clippy) for linting besides running the project's own tests.

## Building the Project

Build the project with

```
cargo build
```

or

```
cargo build --release
```

for a release build.

The binary in `target/release/my-app-web` is your deployment artifact which in the case of a minimal project has no external dependencies since no database or similar is being used.

## Documenting the Project

Gerust projects come with complete API documentation out of the box. Generate the documentation with

```
cargo doc --workspace --all-features
```

and access it via `target/doc/my_app_web/index.html`. The API documentation is a great way to explore in more detail all of the elements of the codebase that Gerust generated.

---

Let's now extend the minimal project step-by-step and explore its various elements along the way.
