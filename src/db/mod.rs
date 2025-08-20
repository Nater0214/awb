use anyhow::Context as _;
use sea_orm::{DatabaseBackend, Schema, prelude::*};

pub(crate) mod guild_settings;
pub(crate) mod quotebook;
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

    // Create quotebook table
    let stmt = schema
        .create_table_from_entity(quotebook::Entity)
        .if_not_exists()
        .take();
    db.execute(db.get_database_backend().build(&stmt))
        .await
        .context("Failed to create quotebook table")?;

    // Return ok
    Ok(())
}
