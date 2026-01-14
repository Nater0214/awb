use anyhow::Context as _;
use poise::{CreateReply, command};
use tokio_stream::StreamExt as _;

use crate::{
    localize_message, settings::get_context_settings,
    utils::chunked_messages::ChunkedMessageGenerator,
};

use super::{Context, Data, Error, Result};

pub(super) fn get_all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![purge()]
}

#[command(
    slash_command,
    name_localized("en-US", "purge"),
    name_localized("es-419", "purgar"),
    subcommands("from")
)]
pub(super) async fn purge(_ctx: Context<'_>) -> Result {
    unreachable!();
}

#[command(
    slash_command,
    name_localized("en-US", "from"),
    name_localized("es-419", "de"),
    description_localized("en-US", "Purge messages from a user"),
    description_localized("es-419", "Limpiar mensajes de un usuario")
)]
pub(super) async fn from(
    ctx: Context<'_>,
    #[name_localized("en-US", "user")]
    #[name_localized("es-419", "usuario")]
    #[description_localized("en-US", "The user to purge messages from")]
    #[description_localized("es-419", "El usuario del que se van a limpiar los mensajes")]
    user: serenity::all::User,
    #[name_localized("en-US", "limit")]
    #[name_localized("es-419", "limite")]
    #[description_localized("en-US", "The number of messages to purge")]
    #[description_localized("es-419", "El número de mensajes a limpiar")]
    limit: Option<u32>,
) -> Result {
    // Get the context settings
    let context_settings = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?;

    // Send the response
    let status_message = ctx
        .say(if let Some(limit) = limit {
            localize_message!(
                "command.purge.from.response.pre.limit",
                &context_settings.language,
                limit,
                user.name
            )
            .await
            .context("Failed to localize message")?
        } else {
            localize_message!(
                "command.purge.from.response.pre.all",
                &context_settings.language,
                user.name
            )
            .await
            .context("Failed to localize message")?
        })
        .await
        .context("Failed to send message")?;

    // Get the channel this was sent in
    let channel = ctx.channel_id();

    // Get a message generator
    let message_gen = ChunkedMessageGenerator::new(100, channel, &ctx);
    let mut mess = message_gen.stream();

    // Loop through messages
    let mut counter = 0u32;
    while let Some(message) = mess.next().await {
        let message = message?;

        if message.author.id == user.id {
            message
                .delete(&ctx)
                .await
                .context("Failed to delete message")?;
            counter += 1;
            if let Some(limit) = limit
                && counter >= limit
            {
                break;
            }
        }
    }

    // Update the response
    status_message
        .edit(
            ctx,
            CreateReply::default().content(
                localize_message!(
                    "command.purge.from.response.post",
                    &context_settings.language,
                    counter,
                    user.name
                )
                .await
                .context("Failed to localize message")?,
            ),
        )
        .await
        .context("Failed to edit reply")?;

    // Return ok
    Ok(())
}
