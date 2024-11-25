---
sidebar_position: 3
---

# The `config` crate

The `config` crate contains the `Config` struct that holds all configuration values at runtime as well as code for parsing the configuration based on a hierarchy of TOML files and environment variables. The `Config` struct contains fields for the server and database configuration (if the application uses a database) and can be extended freely:

```rust
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig, // The database configuration only exists for projects that use a database
    // add your config settings here…
}
```

The values for the server and database configuration are read from the `APP_SERVER__IP`, `APP_SERVER__PORT`, and `APP_DATABASE__URL` environment variables. Any application-specific settings are read from `app.toml` as well as environment-specific file, e.g. `production.toml` such that settings in the environment-specific files override those in `app.toml`. In development and test environments, of course Gerust supports loading `.env` and `.env.test` dotenv files as well. Gerust uses the [`figment` crate](https://crates.io/crates/figment) for managing config settings and overlaying settings from different sources.

## Environment

Besides parsing and making available the application's configuration, the `config` crate also determines the environment the application is running in. Gerust supports 3 environments: development, test, and production – the development environment is the default, and tests are automatically run with the test environment. The environment can also manually be set by passing an argument to the commands made available via the [`cli` crate](./the-cli-crate) or via the `APP_ENVIRONMENT` environment variable.
