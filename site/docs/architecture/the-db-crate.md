---
sidebar_position: 2
---

# The `db` crate

The `db` crate only exists for projects that use a database and contains all functionality related to database access from entity definitions, functions for reading and writing data, as well as migrations. Gerust uses [sqlx](https://crates.io/crates/sqlx) and PostgreSQL without any additional ORM on top. Instead, it defines entities as simple structs along with functions for retrieving and persisting those entities. While that leads to a bit more code, it avoids a good amount of accidental complexity that ORMs typically come with – more about that decision in the [architecture docs](../architecture/#main-choices).

```rust
#[derive(Serialize, Debug, Deserialize)]
pub struct Task {                                            // a Task entity with UUID id and text description
    pub id: Uuid,
    pub description: String,
}

pub async fn load(                                           // Function for loading a Task for an id
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
```

Data manipulation in Gerust is done via changsets. Those are separate companion structs to each entity which only contain the fields that are editable in the respective entity (e.g. not the `id` field in case that's auto-assigned by the database). The concept of changesets is inspired from [Elixir's Ecto library](https://hexdocs.pm/ecto/Ecto.Changeset.html). Validations are implemented on the changesets via the [`validator` crate](https://crates.io/crates/validator).

```rust
// db/src/entities/tasks.rs

#[derive(Deserialize, Validate, Clone)]
pub struct TaskChangeset {                                   // the changeset definition for the Task entity; it requires description to have a minimum length of 1
    #[validate(length(min = 1))]
    pub description: String,
}

pub async fn create(                                         // Function for creating a Task in the database
    task: TaskChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    task.validate().map_err(crate::Error::ValidationError)?; // Validate the changeset and return Err(…) if it isn't valid

    let record = sqlx::query!(                               // Store the data in the database
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        task.description
    )
    .fetch_one(executor)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?;

    Ok(Task {                                                // Return a Task entity
        id: record.id,
        description: task.description,
    })
}
```

Database queries are checked for correctness at compile time using sqlx's [compile-time checked queries](https://github.com/launchbadge/sqlx/blob/main/README.md#sqlx-is-not-an-orm).

## The API

Instead of using an ORM, Gerust relies on sqlx and plain functions that use plain entity structs and changesets for interfacing with the database. Each entity is defined in its own module, along with the corresponding changeset and related functions for e.g. loading an entity based on its ID, updating an entity, or creating a new one:

```rust
pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<Task>, anyhow::Error> {
    let tasks = sqlx::query_as!(Task, "SELECT id, description FROM tasks")
        .fetch_all(executor)
        .await?;
    Ok(tasks)
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

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!("DELETE FROM tasks WHERE id = $1 RETURNING id", id)
        .fetch_optional(executor)
        .await
        .map_err(|e| crate::Error::DbError(e.into()))?
    {
        Some(_) => Ok(()),
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

pub async fn update(
    id: Uuid,
    task: TaskChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    task.validate().map_err(crate::Error::ValidationError)?;

    match sqlx::query!(
        "UPDATE tasks SET description = $1 WHERE id = $2 RETURNING id, description",
        task.description,
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?
    {
        Some(record) => Ok(Task {
            id: record.id,
            description: record.description,
        }),
        None => Err(crate::Error::NoRecordFound),
    }
}
```

Any errors are mapped to variants of the `Errors` enum defined in the `db`'s `lib.rs`. That file does also define a helper function for starting a new transaction which can be passed for the `executor` argument of all the data access functions above. If no transaction is needed, a database connection can be passed as well, which is available e.g. via the application state in the [`web` crate](./the-web-crate).

## Migrations and Seeds

The `db` crate is also where the application's migrations and seed data are stored. Migrations are in the `db/migrations` folder as plain SQL files. For the moment, Gerust does not support down migrations so that each migration file simply contains the SQL to execute when the migration is applied. See data (stable data that does not typically change and could be re-imported any time. e.g. lists of currencies or countries) is defined in `db/seeds.sql`.

Generating and executing migrations as well as loading the seed data into the database can be done via the [`cli` crate](./the-cli-crate).

## File Structure

The crate's file structure consists of 3 main folders:

```
db
├── migrations       // Database migrations as plain SQL files
├── src
    ├── entities     // Entity structs, changesets and related functions for retrieving and persisting records (see example above)
    └── test-helpers // Functions for retrieving and persisting records that are only relevant for tests (these are defined behind the `test-helpers` feature)
```

## Test Helpers

The `db` crate has a feature `test-helpers` that is off by default. Hidden behind the feature flag is the `test_helpers` module, which can be used to make specific entities and database access functions available only for application tests but not for actual application code. If e.g. the system does not allow for creating new user accounts as part of the normal operation, but tests need to be able to create users to prepare the state for tests, a `create_user` function could be defined in `db/src/test_helpers/users.rs` in the `db` crate.

Entities and their changesets in the `db` crate also use the [`fake` crate](https://crates.io/crates/fake)'s `Dummy` trait for easier creation of test data when the `test-helpers` feature is enabled:

```rust
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct TaskChangeset {
    /// The description must be at least 1 character long.
    #[cfg_attr(feature = "test-helpers", dummy(faker = "Sentence(3..8)"))]
    #[validate(length(min = 1))]
    pub description: String,
}
```

With that, it's straight forward to create new tasks with realistic data in tests without having to specify all values manually:

```rust
let task_changeset: TaskChangeset = Faker.fake();
create_task(task_changeset.clone(), &context.db_pool)
    .await
    .unwrap();
```

The [`web` crate](./the-web-crate) comes preconfigured so it enables the `test-helpers` feature flag for its dependency on the `db` crate when running tests.
