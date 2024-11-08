---
sidebar_position: 2
---

# Architecture

**Gerust is built with API servers in mind.** We expect it to be used in projects that provide an HTTP interface (likely JSON, e.g. REST, although Gerust makes not assumption on the exact data format used) over a network. The API server might provide access to data in a database but Gerust can also be used for projects with a database, e.g. as a proxy in front of other services. We specifically **did not create Gerust for website projects** that render HTML directly, or handle e.g. form submissions originating from browsers directly.

Gerust's goal is to handle all aspects of backend projects that are not specific to the concrete use case the project is built for. Dealing with accidental complexity like figuring out where to put what kind of file, how to set up tracing, how to isolate tests from each other, etc. shouldn't be what developers spend their time on. At the same time, Gerust aims to remain flexibly regarding aspects that are very well essential to projects – it makes no assumptions on the exact data format of the API or how entities from the database map to resources that are exposed via the API.

Besides finding that balance between strictness on non-essential and flexibility on essential aspects, Gerust aims for maintainability. Separating different concerns clearly, choosing the right dependencies and enabling simple and efficient workflows, all contribute to codebases that developers will be able to work on efficiently for the long-term.

## Main Choices

Gerust makes a number of fundamental choices that you, a developer focused on delivering business value, don't need to spend time on:

- **Gerust uses [`axum`](https://crates.io/crates/axum) as the underlying web framework** for its `web` crate. `axum` is a capable framework that provides all relevant functionality. It supports middlewares via the [`tower`](https://crates.io/crates/tower) and [`tower-http`](https://crates.io/crates/tower-http) ecosystems. It's been relatively stable for some time and it being part of the [`tokio`](https://crates.io/crates/tokio) project is a positive signal for long-term maintenance of the crate.
- **Instead of using an ORM, Gerust uses [`sqlx`](https://crates.io/crates/sqlx) for database access.** Deciding between using an ORM or not is often a tradeoff between the convenience that ORMs bring on the one hand and additional accidental complexity on the other hand. At this point, with the libraries that are available in the ecosystem, we feel that not using an ORM and accepting a little more code and repetition in the data layer, still results in a simpler codebase and architecture overall and better long-term maintainability.
- **Gerust by default assumes a PostgreSQL database.** PostgreSQL is a proven, stable, and fast choice and widely used as a standard choice for backend systems in various ecosystems. Yet, support for other relational database, or other kinds of datastores altogether, might be added in the future.
- Gerust comes with a **tracing set up out-of-the-box.** Tracing is the kind of topic that often gets neglected but shouldn't since being able to peek into your production system and understand what's going on, is critical for any serious system.
- **Gerust projects are separated into various crates** (see below) using [Cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). That structure helps with shortening compile times and clearly separates concerns which e.g. would allow for different people or teams to work on different crates without interfering with each other. Within each crate, Gerust defines a fixed file structure so nobody needs to invest time into figuring out where to put things.
- Contributing to any codebase should be as simple and require as little setup time as possible. Gerust projects are set up in a way such that **a working Rust toolchain is all that's needed to build, test, and manage the project** – no services are expected to be present in the development environment and no additional crates with additional CLI tools need to be installed. The database can be run in the development environment but Gerust also creates a Docker configuration out-of-the-box.

## Project Structure

Depending on the kind of project, there a four or five crates in a Gerust workspace:

```
.
├── cli    // CLI tools for e.g. running DB migrations or generating project files
├── config // Defines the `Config` struct and handles building the configuration from environment-specific TOML files and environment variables
├── db     // Encapsulates database access, migrations, as well as entity definitions and related code (this crate only exists if the project uses a database)
├── macros // Contains macros, e.g. for application tests
└── web    // The web interface as well as tests for it
```
