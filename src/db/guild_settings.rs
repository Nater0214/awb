use anyhow::Context;
use sea_orm::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(2))")]
pub enum Language {
    #[sea_orm(string_value = "en")]
    English,
    #[sea_orm(string_value = "es")]
    Spanish,
}

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "guild_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub guild_id: String,
    pub language: Option<Language>,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Create an entry in the guild settings table
pub(crate) async fn create_entry(
    db: &DbConn,
    guild_id: impl AsRef<str>,
) -> Result<(), anyhow::Error> {
    // Use the active value types
    use sea_orm::ActiveValue::*;

    // Get guild_id as a string
    let guild_id = guild_id.as_ref().to_string();

    // Create a new table entry
    let new_entry = ActiveModel {
        guild_id: Set(guild_id),
        language: Set(None),
        ..Default::default()
    };

    // Insert the new entry into the table
    new_entry
        .insert(db)
        .await
        .context("Could not insert new entry into guild settings table")?;

    // Return ok
    Ok(())
}
