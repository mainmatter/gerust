---
sidebar_position: 1
---

# Creating the Entity

The `Note` entity constitutes the core of the application and is used to store notes in the database.

## Generating the Entity

We begin with generating the entity:

```
» cargo generate entity note
```

The creates the `Note` entity in `db/src/entities/notes.rs` along with functions for reading, creating, updating, and deleting notes.

Gerust entities are plain Rust structs. New entities come with an `id` out of the box (Gerust uses UUIDs via the [`uuid` crate](https://crates.io/crates/uuid)) and an example `name` property:

```rust
#[derive(Serialize, Debug, Deserialize)]
pub struct Note {
    // these are examples only
    pub id: Uuid,
    pub name: String,
}
```

For the `Note` entity, we'll want a `text` property instead so we can change that accordingly (as well as all other occurences of `name` in the file – we'll go into more detail about those below):

```rust
#[derive(Serialize, Debug, Deserialize)]
pub struct Note {
// diff-remove
-    // these are examples only
    pub id: Uuid,
// diff-remove
-    pub name: String,
// diff-add
+    pub text: String,
}
```

Data manipulation in Gerust is done via changsets. Those are separate companion structs to each entity which only contain the fields that are editable in the respective entity (e.g. not the `id` field since that's auto-assigned by the database). The concept of changesets is inspired from [Elixir's Ecto library](https://hexdocs.pm/ecto/Ecto.Changeset.html). Validations are implemented on the changesets via the [`validator` crate](https://crates.io/crates/validator). The `NoteChangeset` was generated along with the `Note` entity when that was created:

```rust
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct NoteChangeset {
    // these are examples only
    #[cfg_attr(feature = "test-helpers", dummy(faker = "Name()"))]
    #[validate(length(min = 1))]
    pub name: String,
}
```

Changesets are also configured for fake data generation with the [`fake` crate](https://crates.io/crates/fake) for easier fake data generation in tests (more on tests later). In this case, we can change the fake data configuration to generate a sentence with 3 to 8 words:

```rust
#[cfg(feature = "test-helpers")]
// diff-remove
-use fake::{faker::name::en::*, Dummy};
// diff-add
+use fake::{faker::lorem::en::*, Dummy};

…

#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct NoteChangeset {
// diff-remove
-    // these are examples only
// diff-remove
-    #[cfg_attr(feature = "test-helpers", dummy(faker = "Name()"))]
// diff-add
+    #[cfg_attr(feature = "test-helpers", dummy(faker = "Sentence(3..8)"))]
    #[validate(length(min = 1))]
// diff-remove
-    pub name: String,
// diff-add
+    pub text: String,
}
```

You'll notice the fake data configuration being applied only if the `test-helpers` feature is enabled. That is only the case when tests are run in the `web` crate (see e.g. the [tests for reading notes via the CRUD interface](./reading-endpoints#testing)) so that none of the fake data functionality or code becomes part of a release build of the application.

### Generating a Migration

Along with the entity, we need a migration to create the database table that stores the entity. We can generate that next:

```
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

```
» docker compose up
```

:::

and migrate the database:

```
» cargo db migrate
```

The database url can be configured in `.env` (and `.env.test` for the test environment – more on that later). By default, Gerust assumes the username to use is the same as the application's name – in this case `my_app` – with the same password.

### The DB Interface

Gerust keeps the interface for loading, creating, updating, and deleting entities completely separate from the entity structs themselves. When the `Note` entity was generated in the previous step, all related functions that interface with the database were generated automatically along with it: `load_all`, `load`, `create`, `update`, `delete`. All of those functions work with plain `Note` and `NoteChangeset` structs and execute SQL via the [`sqlx` crate](https://crates.io/crates/sqlx)'s [`query!`](https://docs.rs/sqlx/0.8.3/sqlx/macro.query.html) and [`query_as!`](https://docs.rs/sqlx/0.8.3/sqlx/macro.query_as.html).

In order to make the generated code work, we now need to remove the `todo!`s and adapt the column and property names:

```rust
pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<Note>, crate::Error> {
// diff-remove
-    let notes = sqlx::query_as!(Note, "SELECT id, name FROM notes")
// diff-add
-    let notes = sqlx::query_as!(Note, "SELECT id, text FROM notes")
        .fetch_all(executor)
        .await?;
    Ok(notes)
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Note, crate::Error> {
// diff-remove
-    todo!("Adapt the SQL query as necessary!");
    match sqlx::query_as!(
        Note,
// diff-remove
-        "SELECT id, name FROM notes WHERE id = $1",
// diff-add
+        "SELECT id, text FROM notes WHERE id = $1",
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

// diff-remove
-    todo!("Adapt the SQL query and bound parameters as necessary!");
    let record = sqlx::query!(
// diff-remove
-       "INSERT INTO notes (name) VALUES ($1) RETURNING id",
// diff-remove
-       note.name
// diff-add
+       "INSERT INTO notes (text) VALUES ($1) RETURNING id",
// diff-add
+       note.text
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(Note {
        id: record.id,
// diff-remove
-       name: note.name,
// diff-add
+       text: note.text,
    })
}

pub async fn update(
    id: Uuid,
    note: NoteChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Note, crate::Error> {
    note.validate()?;

// diff-remove
-    todo!("Adapt the SQL query and bound parameters as necessary!");
    match sqlx::query!(
// diff-remove
-       "UPDATE notes SET name = $1 WHERE id = $2 RETURNING id, name",
// diff-remove
-       note.name,
// diff-add
+       "UPDATE notes SET text = $1 WHERE id = $2 RETURNING id, text",
// diff-add
+       note.text,
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(record) => Ok(Note {
            id: record.id,
// diff-remove
-           name: record.name,
// diff-add
+           text: record.text,
        }),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
// diff-remove
-    todo!("Adapt the SQL query as necessary!");
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

Once that's done, we can run the application again:

```
» cargo run
```

:::info

SQLx does compile-time query checking which means it will connect to your database during compilation and ensure all queries as they appear in the code are actually in sync with the schema, e.g. there are no typos in column or table names, etc. If you missed a change in any of the queries in the previous step, you'll find out when you try running the application.

:::

---

Next, we'll add endpoints to the application for reading notes via the REST+JSON interface.
