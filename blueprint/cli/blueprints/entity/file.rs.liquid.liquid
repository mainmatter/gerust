#[cfg(feature = "test-helpers")]
use fake::{faker::lorem::en::*, Dummy};
use serde::Deserialize;
use serde::Serialize;
use sqlx::Postgres;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Debug, Deserialize)]
pub struct {{entity_struct_name}} {
    pub id: Uuid,
    {%- for field in fields %}
    pub {{ field.name }}: {{ field.type }},
    {%- endfor %}
}

#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct {{entity_struct_name}}Changeset {
    {%- for field in fields %}
    //#[cfg_attr(feature = "test-helpers", dummy(faker = "…()"))]
    //#[validate(…))]
    pub {{ field.name }}: {{ field.type }},
    {%- endfor %}
}

pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<{{entity_struct_name}}>, crate::Error> {
    let {{entity_plural_name}} = sqlx::query_as!({{entity_struct_name}}, "SELECT id, {{ fields | map: "name" | join: ", " }} FROM {{entity_plural_name}}")
        .fetch_all(executor)
        .await?;
    Ok({{entity_plural_name}})
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<{{entity_struct_name}}, crate::Error> {
    match sqlx::query_as!(
        {{entity_struct_name}},
        "SELECT id, {{ fields | map: "name" | join: ", " }} FROM {{entity_plural_name}} WHERE id = $1",
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some({{entity_singular_name}}) => Ok({{entity_singular_name}}),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    {{entity_singular_name}}: {{entity_struct_name}}Changeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<{{entity_struct_name}}, crate::Error> {
    {{entity_singular_name}}.validate()?;

    let record = sqlx::query!(
        "INSERT INTO {{entity_plural_name}} ({{ fields | map: "name" | join: ", " }}) VALUES ({%- for field in fields -%}${{ forloop.index }}{%- unless forloop.last -%}, {% endunless -%}{%- endfor -%}) RETURNING id",
        {%- for field in fields %}
        {{entity_singular_name}}.{{ field.name }},
        {%- endfor %}
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok({{entity_struct_name}} {
        id: record.id,
        {%- for field in fields %}
        {{ field.name }}: {{entity_singular_name}}.{{ field.name }},
        {%- endfor %}
    })
}

pub async fn update(
    id: Uuid,
    {{entity_singular_name}}: {{entity_struct_name}}Changeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<{{entity_struct_name}}, crate::Error> {
    {{entity_singular_name}}.validate()?;

    match sqlx::query!(
        "UPDATE {{entity_plural_name}} SET {% for field in fields -%}{{ field.name }} = ${{ forloop.index }}{%- unless forloop.last -%}, {% endunless -%}{%- endfor %} WHERE id = ${{ fields | size | plus: 1 }} RETURNING id",
        {%- for field in fields %}
        {{entity_singular_name}}.{{ field.name }},
        {%- endfor %}
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(record) => Ok({{entity_struct_name}} {
            id: record.id,
            {%- for field in fields %}
            {{ field.name }}: {{entity_singular_name}}.{{ field.name }},
            {%- endfor %}
        }),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!("DELETE FROM {{entity_plural_name}} WHERE id = $1 RETURNING id", id)
        .fetch_optional(executor)
        .await
        .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}
