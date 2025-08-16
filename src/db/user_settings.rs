use anyhow::Context as _;
use async_recursion::async_recursion;
use sea_orm::{IntoActiveModel as _, prelude::*};
use serenity::all::UserId;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(2))")]
pub enum Language {
    #[sea_orm(string_value = "en")]
    English,
    #[sea_orm(string_value = "es")]
    Spanish,
}

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub user_id: String,
    pub language: Option<Language>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Create an entry in the user settings table
pub(crate) async fn create_entry(
    db: &DbConn,
    user_id: impl AsRef<str>,
) -> Result<(), anyhow::Error> {
    // Use the active value types
    use sea_orm::ActiveValue::*;

    // Get user_id as a string
    let user_id = user_id.as_ref().to_string();

    // Create a new entry
    let new_entry = ActiveModel {
        user_id: Set(user_id),
        language: Set(None),
        ..Default::default()
    };

    // Insert the new entry into the database
    new_entry
        .insert(db)
        .await
        .context("Could not insert new entry into user settings table")?;

    // Return ok
    Ok(())
}

/// Change a user's settings
#[async_recursion]
pub(crate) async fn update_entry<V>(
    db: &DbConn,
    user_id: UserId,
    column: Column,
    value: V,
) -> Result<(), anyhow::Error>
where
    V: Into<sea_orm::Value> + Send,
{
    let value = value.into();
    let user_id_ = user_id.to_string();
    let model = Entity::find()
        .filter(Column::UserId.eq(&user_id_))
        .one(db)
        .await
        .context("Could not get user settings entry in the database")?;
    if let Some(model) = model {
        let mut model = model.into_active_model();
        model.set(column, value);
        model
            .update(db)
            .await
            .context("Could not update user settings entry in the database")?;
    } else {
        create_entry(db, &user_id_)
            .await
            .context("Could not create user settings entry in the database")?;
        update_entry(db, user_id, column, value).await?;
    }
    Ok(())
}
