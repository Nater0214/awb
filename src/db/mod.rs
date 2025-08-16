use anyhow::Context;
use sea_orm::{DatabaseBackend, Schema, prelude::*};

pub(crate) mod guild_settings;
pub(crate) mod user_settings;

/// Setup the schema of the database
pub(crate) async fn setup_schema(db: &DbConn) -> Result<(), anyhow::Error> {
    // Create the schema
    let schema = Schema::new(DatabaseBackend::Sqlite);

    // Create guild_settings table
    let stmt = schema
        .create_table_from_entity(guild_settings::Entity)
        .if_not_exists()
        .take();
    db.execute(db.get_database_backend().build(&stmt))
        .await
        .context("Failed to create guild_settings table")?;

    // Create user_settings table
    let stmt = schema
        .create_table_from_entity(user_settings::Entity)
        .if_not_exists()
        .take();
    db.execute(db.get_database_backend().build(&stmt))
        .await
        .context("Failed to create user_settings table")?;

    // Return ok
    Ok(())
}
