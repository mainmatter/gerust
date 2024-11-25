---
sidebar_position: 5
---

# The `macros` crate

The `macros` crate contains the implementation of Gerust's `test` and `db_test` macros. Those macros are used on application tests (see [`web` crate docs](./the-web-crate#testing)) instead of the [`tokio` crate](https://crates.io/crates/tokio)'s own `test` macro. In addition to wrapping `tokio`'s `test` macros, Gerust's macros perform some additional tasks:

- They create a new instance of the application and pass that into the test via the test context (see [`web` crate docs](./the-web-crate#testing)).
- The `db_test` macro furthermore creates a new database that's specific for the test (and created using the main test database as a template) which the application is configured to use and which is passed into the test via the test context. That database is automatically deleted once the test completes so that no unused databases are left behind.

A developer working on a Gerust project would typically not have to make changes to anything inside the `macros` crate directly.
