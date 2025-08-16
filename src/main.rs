use std::{path, sync::OnceLock};

use db::setup_schema;
use poise::{Framework, FrameworkError};
use sea_orm::Database;
use serenity::all::{ClientBuilder, GatewayIntents};
use tokio::fs;
use tracing::{Level, event};
use tracing_subscriber::prelude::*;
use utils::prelude::*;

mod commands;
mod db;
mod localization;
mod settings;
mod utils;

/// Whether the bot is in development mode
pub(crate) static DEVELOPMENT_MODE: OnceLock<bool> = OnceLock::new();

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

    // Get the token
    let token = match fs::read_to_string("token.txt").await {
        Ok(token) => token,
        Err(_) => {
            println!("Please enter your token:");
            let mut token = String::new();
            std::io::stdin().read_line(&mut token).unwrap();
            fs::write("token.txt", token.trim()).await.unwrap();
            token.trim().to_string()
        }
    };
    event!(Level::INFO, "Token loaded");

    // Connect to the database
    if !fs::try_exists(path::Path::new("db.sqlite"))
        .await
        .expect_log("Could not determine if database file exists")
    {
        match fs::File::create("db.sqlite").await {
            Ok(_) => (),
            Err(_) => panic!("Failed to create database file"),
        }
    }
    let db = Database::connect("sqlite:db.sqlite")
        .await
        .expect_log("Failed to connect to sqlite database");

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
                    match error {
                        FrameworkError::Command { error, ctx, .. } => {
                            event!(
                                Level::ERROR,
                                "Error result returned from command `{}`: {:#}",
                                ctx.command().name,
                                error
                            )
                        }
                        FrameworkError::CommandPanic { payload, ctx, .. } => {
                            event!(
                                Level::ERROR,
                                "Panic in command `{}`: {}",
                                ctx.command().name,
                                payload.unwrap_or("No payload".to_owned()),
                            )
                        }
                        other_error => {
                            event!(Level::ERROR, "Error in the framework: {}", other_error)
                        }
                    }
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
