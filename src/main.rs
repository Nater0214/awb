use std::{env, path, sync::OnceLock};

use anyhow::Context;
use db::setup_schema;
use poise::{Framework, FrameworkError};
use sea_orm::{ConnectOptions, Database};
use serenity::all::{ClientBuilder, GatewayIntents};
use tokio::fs;
use tracing::{Level, event};
use tracing_subscriber::prelude::*;
use utils::prelude::*;

use crate::settings::get_context_settings;

mod commands;
mod db;
mod localization;
mod settings;
mod utils;

/// Whether the bot is in development mode
pub(crate) static DEVELOPMENT_MODE: OnceLock<bool> = OnceLock::new();

async fn error_handler(
    error: FrameworkError<'_, commands::Data, commands::Error>,
) -> anyhow::Result<()> {
    // Match the error type
    match error {
        FrameworkError::Command { error, ctx, .. } => {
            event!(
                Level::ERROR,
                "Error result returned from command `{}`: {:#}",
                ctx.command().name,
                error
            );
            let context_settings = get_context_settings(&ctx, &ctx.data().db)
                .await
                .context("Failed to get context settings")?;
            ctx.say(
                localize_message!(
                    "error.command.result.response",
                    &context_settings.language,
                    error.to_string()
                )
                .await
                .context("Failed to localize message")?,
            )
            .await
            .context("An additional error occurred responding to the error")?;
        }
        FrameworkError::CommandPanic { payload, ctx, .. } => {
            let payload = payload.unwrap_or("No payload".to_owned());
            event!(
                Level::ERROR,
                "Panic in command `{}`: {}",
                ctx.command().name,
                &payload,
            );
            let context_settings = get_context_settings(&ctx, &ctx.data().db)
                .await
                .context("Failed to get context settings")?;
            ctx.say(
                localize_message!(
                    "error.command.panic.response",
                    &context_settings.language,
                    &payload
                )
                .await
                .context("Failed to localize message")?,
            )
            .await
            .context("An additional error occurred responding to the error")?;
        }
        other_error => {
            event!(Level::ERROR, "Error in the framework: {}", other_error);
        }
    };

    // Return ok
    Ok(())
}

/// The main function
#[tokio::main]
async fn main() {
    // Start tracing
    let tracing_filter = tracing_subscriber::filter::LevelFilter::from_level(Level::INFO);
    let (tracing_filter, reload_handle) = tracing_subscriber::reload::Layer::new(tracing_filter);
    tracing_subscriber::registry()
        .with(tracing_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    event!(Level::INFO, "Starting bot");

    // Check if bot is in development
    if fs::try_exists(path::Path::new("dev.txt"))
        .await
        .expect_log("Could not determine if bot is in development")
    {
        event!(Level::INFO, "Changing bot to development mode");
        DEVELOPMENT_MODE
            .set(true)
            .expect_log("Failed to set development mode");
        reload_handle
            .modify(|filter| {
                *filter = tracing_subscriber::filter::LevelFilter::from_level(Level::DEBUG)
            })
            .expect_log("Failed to change log level");
    }

    // Get the path to the token file
    let token_path = env::var_os("TOKEN_PATH").unwrap_or("token.txt".into());

    // Get the token
    let token = fs::read_to_string(token_path)
        .await
        .expect_log("Could not read token file");
    event!(Level::INFO, "Token loaded");

    // Get the database url
    let database_url = env::var_os("DATABASE_URL").unwrap_or("sqlite::memory:".into());

    // Connect to the database
    let db = Database::connect(ConnectOptions::new(database_url.to_string_lossy()))
        .await
        .expect_log("Failed to connect to database");

    // Setup the database schema
    setup_schema(&db)
        .await
        .expect_log("Failed to setup database schema");

    // Create the intents
    let intents = GatewayIntents::non_privileged().union(GatewayIntents::MESSAGE_CONTENT);

    // Create the framework
    let framework = Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_all_commands(),
            on_error: |error| {
                Box::pin(async move {
                    error_handler(error)
                        .await
                        .context("An additional error occurred in handling the error")
                        .expect_log("An additional error occurred in handling the error");
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if *DEVELOPMENT_MODE.get().unwrap_or(&false) {
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        "1235772100384526377".parse().unwrap(),
                    )
                    .await?;
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                Ok(commands::Data::new(db))
            })
        })
        .build();

    // Create the client
    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect_log("Failed to create the client");

    // Start the client
    client
        .start()
        .await
        .expect_log("Failed to start the client");
}
