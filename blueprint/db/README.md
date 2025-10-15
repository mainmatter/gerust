# {{project-name}}-db

This crate contains all code related to database access: entities, migrations, functions for validating and reading and writing data.

Gerust uses [`sqlx`] without an additional ORM and is set up for use with PostgreSQL.

## Entities

Entities are defined as plain structs, e.g.:

```rust
#[derive(Serialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}
```

## Reading and writing data

Instead of using an ORM, that would introduce additional complexity, Gerust uses individual functions for reading and writing data from and to the database, e.g.:

```rust
pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<User, crate::Error> {
    match sqlx::query_as!(Task, "SELECT id, name FROM users WHERE id = $1", id)
        .fetch_optional(executor)
        .await
        .map_err(|e| crate::Error::DbError(e.into()))?
    {
        Some(user) => Ok(user),
        None => Err(crate::Error::NoRecordFound),
    }
}
```

Database queries are checked for correctness at compile time using [sqlx's compile-time checked queries][sqlx is not an ORM].

All database operations can be performed standalone as atomic operations or grouped into a transaction – the `executor` argument can either be a connection pool or a transaction, e.g.:

```rust
let user_changeset: UserChangeset = Faker.fake();
match transaction(&app_state.db_pool).await {
    Ok(mut tx) => {
        create_user(user_changeset.clone(), &mut *tx)
            .await
            .unwrap();
        tx.commit()?;
    }
    Err(e) => Err(anyhow!("Could not start transaction!")),
}
```

### Validations

Data validation on write operations is implemented via a changeset architecture. Instead of validating the entities themselves, the changesets are validated before they can be applied to an entity (in the case of an update operation) or converted into an entity (in the case of an insert operation), e.g.:

```rust
#[derive(Deserialize, Validate, Clone)]
pub struct UserChangeset {
    #[validate(length(min = 1))]
    pub name: String,
}

pub async fn create(
    user: UserChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<User, crate::Error> {
    user.validate()?;

    let record = sqlx::query!(
        "INSERT INTO users (name) VALUES ($1) RETURNING id",
        user.name
    )
    .fetch_one(executor)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?;

    Ok(User {
        id: record.id,
        description: user.name,
    })
}
```

Validations are implemented with [`validator`] and declared using the `validate` attribute on the respective fields of the struct.

### Generating test data

Application tests will typically require test data to populate the database with, e.g. a set of entities to assert that the endpoint that returning all entities of that type works correctly. Gerust uses [`fake`] for so that rules for creating fake data can be declared directly in the changesets:

```rust
#[cfg_attr(feature = "test-helpers", derive(Dummy))]
pub struct UserChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "Name"))]
    pub name: String,
}
```

That allows for straight-forward creation of entities in tests, e.g.:

```rust
let user_changeset: UserChangeset = Faker.fake();
create_user(user_changeset, &context.db_pool)
    .await
    .unwrap();
```

## Test helpers

As seen in the code example above, the fake data definition is only added to the changesets when the `test-helpers` feature flag is set. Fake data is of course only required for testing and thus should not be part of the production application. The [`web` crate](../web) only enables that feature flag for the dev dependency to this crate.

The `db` crate also comes with a dedicate module for additional helpers that only gets built when the `test-helpers` feature flag is set. The `test_helpers` module in [`src/test-helpers`](./src/test_helpers/) can be used to e.g. define functionality that allows creating entities that can not be created as part of the normal application flow but might be necessary to create in tests.

## Migrations

Migrations are stored as plain SQL files under `migrations`. In order to maintain a stable order, migrations are sorted by creation date – the [`migration` generator](../cli/README.md) will automatically generate files with the correct prefix.

[`fake`]: https://crates.io/crates/fake "fake on crates.io"
[`sqlx`]: https://crates.io/crates/sqlx "SQLx on crates.io"
[sqlx is not an ORM]: https://github.com/launchbadge/sqlx/blob/main/README.md#sqlx-is-not-an-orm "SQLx is not an ORM!"
[`validator`]: https://crates.io/crates/validator "validator on crates.io"