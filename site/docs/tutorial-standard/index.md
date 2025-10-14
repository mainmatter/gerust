---
sidebar_position: 4
---

# Tutorial: Complete Project

In this tutorial, we will create a complete application with a database, (simplified) authentication and full test coverage. The application is a basic notes management app with a REST+JSON interface.

Create a new project with

```sh
» gerust my-app
```

That creates a basic project structure:

```
├── Cargo.toml
├── README.md
├── cli
│   ├── Cargo.toml
│   ├── README.md
│   ├── blueprints
│   │   ├── controller
│   │   │   ├── crud
│   │   │   │   ├── controller.rs
│   │   │   │   └── test.rs
│   │   │   └── minimal
│   │   │       ├── controller.rs
│   │   │       └── test.rs
│   │   ├── entity
│   │   │   └── file.rs
│   │   ├── entity-test-helper
│   │   │   └── file.rs
│   │   └── middleware
│   │       └── file.rs
│   └── src
│       ├── bin
│       │   ├── db.rs
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
├── db
│   ├── Cargo.toml
│   ├── README.md
│   ├── migrations
│   ├── seeds.sql
│   └── src
│       ├── entities
│       │   └── mod.rs
│       ├── lib.rs
│       └── test_helpers
│           └── mod.rs
├── docker-compose.yml
├── macros
│   ├── Cargo.toml
│   ├── README.md
│   └── src
│       └── lib.rs
└── web
    ├── Cargo.toml
    ├── README.md
    ├── src
    │   ├── controllers
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
            └── main.rs
```

The structure contains the `cli`, `config`, `db`, `macros`, and `web` crates. The `cli` crate contains binaries for creating project files like controllers or middlewares as well as for managing the database. The `config` crate contains code for reading configuration values and managing them in a `Config` struct. The `db` crate contains all functionality related to database access from entity definitions, functions for reading and writing data, as well as migrations. The `macros` crate contains macros that are used for testing. And the `web` crate contains the system's HTTP interface implementation, which essentially is a standard `axum` application. More on the different crates, their purpose and elements, in the [Architecture guide](../architecture).

## Running the Project

Run the generated project with:

```sh
» cargo run
```

That starts up the server at `127.0.0.1:3000`.

## Testing the Project

Run the tests with:

```sh
» cargo test
```

There are no tests yet but we'll add them step-by-step as we progress through the tutorial.

The project comes preconfigured with a CI setup for GitHub Actions that covers format checking with [`rustfmt`](https://github.com/rust-lang/rustfmt) and [`clippy`](https://github.com/rust-lang/rust-clippy) for linting besides running the project's own tests.

## Building the Project

Build the project with

```sh
» cargo build
```

or

```sh
» cargo build --release
```

for a release build.

The binary in `target/release/my-app-web` is your deployment artifact which has no external dependencies except for the database.

## Documenting the Project

Gerust projects come with complete API documentation out of the box. Generate the documentation with

```sh
» cargo doc --workspace --all-features
```

and access it via `target/doc/my_app_web/index.html`. The API documentation is a great way to explore in more detail all of the elements of the codebase that Gerust generated.

---

Let's now build the notes management app step-by-step and explore its various elements along the way.
