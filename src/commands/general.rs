use anyhow::{Context as _, anyhow};
use poise::{Command, command};

use crate::localize_message;
use crate::settings::get_context_settings;

use super::{Context, Data, Error, Result};

pub(super) fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![hello(), whereareyou(), whoareyou()]
}

#[command(
    slash_command,
    name_localized("en-US", "hello"),
    name_localized("es-419", "hola"),
    description_localized("en-US", "Give a simple greeting"),
    description_localized("es-419", "Dar un saludo sencillo")
)]
pub(super) async fn hello(ctx: Context<'_>) -> Result {
    let context_settings = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?;
    ctx.say(
        localize_message!(
            "command.hello.response",
            &context_settings.language,
            ctx.author().display_name()
        )
        .await
        .context("Failed to localize message")?,
    )
    .await
    .context("Failed to send message")?;

    // Return ok
    Ok(())
}

#[command(
    slash_command,
    name_localized("en-US", "whereareyou"),
    name_localized("es-419", "dondeestas"),
    description_localized("en-US", "Say where I am"),
    description_localized("es-419", "Decir dónde estoy")
)]
pub(super) async fn whereareyou(ctx: Context<'_>) -> Result {
    // Get the context settings
    let context_settings = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?;

    if let Some(guild_channel) = ctx.guild_channel().await {
        let guild_name = ctx
            .guild()
            .ok_or(anyhow!("Guild not found"))
            .context("Guild not found")?
            .name
            .clone();
        ctx.say(
            localize_message!(
                "command.whereareyou.response.server",
                &context_settings.language,
                guild_name,
                guild_channel.name
            )
            .await
            .context("Could not localize message")?,
        )
        .await
        .context("Failed to send message")?;
    } else {
        ctx.say(
            localize_message!(
                "command.whereareyou.response.dm",
                &context_settings.language,
            )
            .await
            .context("Could not localize message")?,
        )
        .await
        .context("Failed to send message")?;
    }

    // Return ok
    Ok(())
}

#[command(
    slash_command,
    name_localized("en-US", "whoareyou"),
    name_localized("es-419", "quienestas"),
    description_localized("en-US", "Say who I am and some info about me"),
    description_localized("es-419", "Decir quién soy y algunos datos sobre mí")
)]
pub(super) async fn whoareyou(ctx: Context<'_>) -> Result {
    // Get the context settings
    let context_settings = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?;

    // Say some info
    ctx.say(
        localize_message!(
            "command.whoareyou.response",
            &context_settings.language,
            env!("CARGO_PKG_VERSION")
        )
        .await
        .context("Failed to localize message")?,
    )
    .await
    .context("Failed to send message")?;

    // Return ok
    Ok(())
}
