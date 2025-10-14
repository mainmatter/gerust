---
sidebar_position: 1
---

# Creating the Entity

The `Note` entity constitutes the core of the application and is used to store notes in the database.

## Generating the Entity

We begin with generating the entity:

```sh
» cargo generate entity note text:string
```

The creates the `Note` entity with a property `text` of type `String` in `db/src/entities/notes.rs` along with functions for reading, creating, updating, and deleting notes.

Gerust entities are plain Rust structs. New entities come with an `id` out of the box (Gerust uses UUIDs via the [`uuid` crate](https://crates.io/crates/uuid)):

```rust
#[derive(Serialize, Debug, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub text: String,
}
```

Data manipulation in Gerust is done via changesets. Those are separate companion structs to each entity which only contain the fields that are editable in the respective entity (e.g. not the `id` field since that's auto-assigned by the database). The concept of changesets is inspired from [Elixir's Ecto library](https://hexdocs.pm/ecto/Ecto.Changeset.html). Validations are implemented on the changesets via the [`validator` crate](https://crates.io/crates/validator). The `NoteChangeset` was generated along with the `Note` entity when that was created:

```rust
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct NoteChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "…"))]
    #[validate(…)]
    pub text: String,
}
```

Changesets are also configured for fake data generation with the [`fake` crate](https://crates.io/crates/fake) for easier fake data generation in tests (more on tests later). In this case, we can change the fake data configuration to generate a sentence with 3 to 8 words.

We'll also want to validate the minimum length of `text` to be at least 1 character:

```rust
…

#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct NoteChangeset {
// diff-remove
-    #[cfg_attr(feature = "test-helpers", dummy(faker = "…"))]
// diff-add
+    #[cfg_attr(feature = "test-helpers", dummy(faker = "Sentence(3..8)"))]
// diff-remove
-    #[validate(…)]
// diff-add
+    #[validate(length(min = 1))]
}
```

You'll notice the fake data configuration being applied only if the `test-helpers` feature is enabled. That is only the case when tests are run in the `web` crate (see e.g. the [tests for reading notes via the CRUD interface](./reading-endpoints#testing)) so that none of the fake data functionality or code becomes part of a release build of the application.

### Generating a Migration

Along with the entity, we need a migration to create the database table that stores the entity. We can generate that next:

```sh
» cargo generate migration create_notes
```

which generates the migration file in `/db/migrations/1737540625__create_notes.sql` (timestamp prefix will vary). Migrations in Gerust are written in plain SQL so for the notes table, we can use this:

```sql
CREATE TABLE notes (
    id uuid PRIMARY KEY default gen_random_uuid(),
    text varchar(255) NOT NULL
);
```

:::info

Gerust comes with a Docker setup out-of-the-box, pre-configured with the right username and password (as configured in the `.env` file). If you're not running a PostgreSQL server in your development environment, start up the containers with

```sh
» docker compose up
```

:::

and migrate the database:

```sh
» cargo db migrate
```

The database url can be configured in `.env` (and `.env.test` for the test environment – more on that later). By default, Gerust assumes the username to use is the same as the application's name – in this case `my_app` – with the same password.

### The DB Interface

Gerust keeps the interface for loading, creating, updating, and deleting entities completely separate from the entity structs themselves. When the `Note` entity was generated in the previous step, all related functions that interface with the database were generated automatically along with it: `load_all`, `load`, `create`, `update`, `delete`. All of those functions work with plain `Note` and `NoteChangeset` structs and execute SQL via the [`sqlx` crate](https://crates.io/crates/sqlx)'s [`query!`](https://docs.rs/sqlx/0.8.3/sqlx/macro.query.html) and [`query_as!`](https://docs.rs/sqlx/0.8.3/sqlx/macro.query_as.html):

```rust
pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<Note>, crate::Error> {
    let notes = sqlx::query_as!(Note, "SELECT id, text FROM notes")
        .fetch_all(executor)
        .await?;
    Ok(notes)
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Note, crate::Error> {
    match sqlx::query_as!(
        Note,
        "SELECT id, text FROM notes WHERE id = $1",
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(note) => Ok(note),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    note: NoteChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Note, crate::Error> {
    note.validate()?;

    let record = sqlx::query!(
        "INSERT INTO notes (text) VALUES ($1) RETURNING id",
        note.text,
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(Note {
        id: record.id,
        text: note.text,
    })
}

pub async fn update(
    id: Uuid,
    note: NoteChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Note, crate::Error> {
    note.validate()?;

    match sqlx::query!(
        "UPDATE notes SET text = $1 WHERE id = $2 RETURNING id",
        note.text,
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(record) => Ok(Note {
            id: record.id,
            text: note.text,
        }),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!("DELETE FROM notes WHERE id = $1 RETURNING id", id)
        .fetch_optional(executor)
        .await
        .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}
```

The entity and the functions for reading and writing it are ready to use and we can run the application again to confirm everything works as expected:

```sh
» cargo run
```

:::info

SQLx does compile-time query checking which means it will connect to your database during compilation and ensure all queries as they appear in the code are actually in sync with the schema, e.g. there are no typos in column or table names, etc. If you missed a change in any of the queries in the previous step, you'll find out when you try running the application.

:::

---

Next, we'll add endpoints to the application for reading notes via the REST+JSON interface.
