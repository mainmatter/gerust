# Pacesetter

Pacesetter answers all the questions you shouldn't waste time on when building backends with Rust:

* How to split a project into crates
* What folder structure to use and what kind of file goes where
* How to handle database migrations
* How to seed the database for tests and clean up afterwards
* How to set up tracing and handle errors
* and many more

Pacesetter projects are based on axum and use sqlx and PostgreSQL for data storage (if data storage is used at all).

> [!NOTE]
> This project has been created by [Mainmatter](https://mainmatter.com/rust-consulting/).  
> Check out our [landing page](https://mainmatter.com/rust-consulting/) if you're looking for Rust consulting or training!

## Creating a new project

A new project can be create with the `pace` command, e.g.:

```
pace my-app
```

By default, Pacesetter will generate an empty project with the complete project structure as described above but without any actual entities, controllers, etc. If you're just getting started looking at Pacesetter, opting into creation of a full project, complete with example implementations of all concepts via `--full` might be a better starting point.

For projects that do not need database access, there is also the `--minimal` option that will generate a project without any of the concepts and structure related to database access.

## Project Structure

Pacesetter uses Cargo workspaces to break down projects into separate crates.

```
.
├── cli    // CLI tools for e.g. running DB migrations or generating files
├── config // Defines the `Config` struct and handles building the configuration from environment-specific TOML files and environment variables
├── db     // Encapsulates database access, migrations, as well as entity definitions and related code (if the project uses a database)
├── macros // Contains macros that are used for application tests
└── web    // The web interface as well as tests for it
```

Let's have a look at those crates in detail:

### The `web` crate

The `web` crate contains the main axum application. It will determine the environment the application runs in, load the configuration, initialize the app state, set up tracing and error handling, and bind the server to the configured interface. The crate uses a simple folder structure:

```
web
├── controllers // Controllers implement request handlers
├── middlewares // Tower middlewares for pre-processing requests before they are passed on the request handlers
├── lib.rs      // Code for starting up the server
├── main.rs     // Main entrypoint of the application
├── routes.rs   // Mapping of request handlers to routes
├── state.rs    // Definition and construction of the application state
└── tests       // Application tests
```

#### Testing

Testing backends is straight forward: request a particular endpoint with a particular method and potentially query string and/or request body and assert the response is what you expect. However, things become more complicated when the server you're testing uses a database and in the test, you need to seed the database with test data for the test and clean up afterwards so different tests don't interfere with each other. There are several mechanisms for that like transactions, cleanup scripts, etc. Pacesetter uses an approach that allows parallel execution of tests via complete isolation without adding a ton of complexity: every test runs in its own database that's automatically created as a copy of the main database for the `test` environment and destroyed after the test has completed. That is enabled via the `[db_test]` macro:

```rs
#[db_test]
async fn test_read_all(context: &DbTestContext) {           // context includes a connection to the database that's specific to this test; the application under test is automatically set up to connect to that database
    let task_changeset: TaskChangeset = Faker.fake();
    create_task(task_changeset.clone(), &context.db_pool) // create a task in the database
        .await
        .unwrap();

    let response = context
        .app
        .request("/tasks")
        .method(Method::GET)
        .send()
        .await;                                           // load all tasks

    assert_that!(response.status(), eq(StatusCode::OK));

    let tasks: TasksList = response.into_body().into_json::<TasksList>().await;
    assert_that!(tasks, len(eq(1)));
    assert_that!(                                         // assert the task created above is returned (as the application uses the same database)
        tasks.first().unwrap().description,
        eq(task_changeset.description)
    );
}
```

### The `db` crate

The `db` crate only exists for projects that use a database and contains everything that's related to database access. Pacesetter uses sqlx and PostgreSQL without any additional ORM on top. Instead, it defines entities as simple structs along with functions for retrieving and persisting those entities. Validations are implemented on changesets that get translated to entities once they were applied successfully:

```rs
#[derive(Serialize, Debug, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
}

#[derive(Deserialize, Validate, Clone)]
pub struct TaskChangeset {
    #[validate(length(min = 1))]
    pub description: String,
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    match sqlx::query_as!(Task, "SELECT id, description FROM tasks WHERE id = $1", id)
        .fetch_optional(executor)
        .await
        .map_err(|e| crate::Error::DbError(e.into()))?
    {
        Some(task) => Ok(task),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    task: TaskChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    task.validate().map_err(crate::Error::ValidationError)?;

    let record = sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        task.description
    )
    .fetch_one(executor)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?;

    Ok(Task {
        id: record.id,
        description: task.description,
    })
}
```

The crate uses a simple folder structure:

```
db
├── migrations // Database migrations as plain SQL files
├── src
    ├── entities     // Entity structs, changesets and related functions for retrieving and persisting records (see example above)
    └── test-helpers // Functions for retrieving and persisting records that are only relevant for tests (these are defined behind the `test-helpers` feature)
```

### The `config` crate

The `config` crate contains the struct that holds all configuration values at runtime as well as code for parsing the configuration based on a hierarchy of TOML files and environment variables. The `Config` struct contains fields for the server and database configuration and can be extended freely:

```rs
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig, // The database configuration only exists for projects that use a database
    // add your config settings here…
}
```

The values for the server and database configuration are read from the `APP_SERVER__IP`, `APP_SERVER__PORT`, and `APP_DATABASE__URL` environment variables. Any application-specific settings are first read from `app.toml` and then from an environment-specific file, e.g. `production.toml` so that environment-specific settings override those in `app.toml`. The main fils and folders in the crate are:

```
config
├── environments
|   ├── development.toml // Configuration settings specific for the development environment
|   ├── production.toml  // Configuration settings specific for the production environment
|   └── test.toml        // Configuration settings specific for the test environment
├── src
|   └── lib.rs           // Contains the `Config` struct and code for constructing it based on the configuration files and environment variables
└── app.toml             // Basis configuration settings that will be overridden by the same settings in the respective environment-specific configuration file
```

### The `cli` crate

The `cli` crate contains the `db` (which only exists for projects that use a database) and `generate` binaries for running database operations such as creating or dropping the database or running migrations, and generating project files such as entities, controllers, or middlewares. The workspace is configured so that those binaries can be run with just `cargo db` and `cargo generate`:

```
» cargo db
A CLI tool to manage the project's database.

Usage: db [OPTIONS] <COMMAND>

Commands:
  drop     Drop the database
  create   Create the database
  migrate  Migrate the database
  reset    Reset (drop, create, migrate) the database
  seed     Seed the database
  help     Print this message or the help of the given subcommand(s)

Options:
  -e, --env <ENV>  Choose the environment (development, test, production). [default: development]
      --no-color   Disable colored output.
      --debug      Enable debug output.
  -h, --help       Print help
  -V, --version    Print version
```

```
» cargo generate
A CLI tool to generate project files.

Usage: generate [OPTIONS] <COMMAND>

Commands:
  middleware            Generate a middleware
  controller            Generate a controller
  controller-test       Generate a test for a controller
  migration             Generate a migration
  entity                Generate an entity
  entity-test-helper    Generate an entity test helper
  crud-controller       Generate an example CRUD controller
  crud-controller-test  Generate a test for a CRUD controller
  help                  Print this message or the help of the given subcommand(s)

Options:
      --no-color  Disable colored output.
      --debug     Enable debug output.
  -h, --help      Print help
  -V, --version   Print version
```

### Testing & CI

Projects generated by Pacesetter come with a complete CI setup for GitHub Actions that includes:

* checking the format of all Rust source files
* running Clippy on the entire project
* running all tests in all crates

## License

Pacesetter is developed by and © Mainmatter GmbH and contributors. It is released under the [MIT License](./LICENSE.md).
