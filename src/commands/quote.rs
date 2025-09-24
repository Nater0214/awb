use std::str::FromStr;

use anyhow::{Context as _, anyhow};
use poise::{Command, command};
use serenity::all::{GetMessages, Member, MessageId};

use crate::{db, localize_message, settings::get_context_settings};

use super::{Context, Data, Error, Result};

pub(super) fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![quote()]
}

#[command(
    slash_command,
    guild_only,
    name_localized("en-US", "quote"),
    name_localized("es-419", "cita"),
    subcommands("save", "view")
)]
pub(super) async fn quote(_ctx: Context<'_>) -> Result {
    unreachable!();
}

#[command(
    slash_command,
    name_localized("en-US", "save"),
    name_localized("es-419", "guardar"),
    description_localized("en-US", "Save a quote"),
    description_localized("es-419", "Guardar una cita")
)]
pub(super) async fn save(
    ctx: Context<'_>,
    #[name_localized("en-US", "message")]
    #[name_localized("es-419", "mensaje")]
    #[description_localized("en-US", "The message ID of the quote to save")]
    #[description_localized("es-419", "El ID del mensaje de la cita a guardar")]
    message: Option<MessageId>,
) -> Result {
    // Get the context settings
    let context_settings = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?;

    // Defer the response
    ctx.defer().await.context("Failed to defer response")?;

    // Get the last message sent if no message was provided
    let message = if let Some(message) = message {
        ctx.channel_id()
            .message(ctx, message)
            .await
            .context("Could not get message")?
    } else {
        ctx.channel_id()
            .messages(ctx, GetMessages::default().limit(2))
            .await?
            .iter()
            .nth(1)
            .ok_or(anyhow!("No messages found"))
            .context("No messages found")?
            .to_owned()
    };

    // Create the database entry
    db::quotebook::create_entry(
        &ctx.data().db,
        message.id.to_string(),
        ctx.guild_id()
            .ok_or(anyhow!("No guild ID found"))
            .context("No guild ID found")?
            .to_string(),
        message.author.id.to_string(),
        message.id.created_at().naive_utc(),
    )
    .await
    .context("Failed to create entry in quotebook table")?;

    // Say that the quote was saved
    ctx.say(
        localize_message!(
            "command.quote.save.response",
            &context_settings.language,
            message.link()
        )
        .await
        .context("Failed to localize message")?,
    )
    .await
    .context("Failed to send message")?;

    // Return ok
    Ok(())
}

/// Create a preview for a quote
async fn create_quote_preview(
    ctx: Context<'_>,
    entry: db::quotebook::Model,
) -> anyhow::Result<String> {
    // Get the language
    let language = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?
        .language;

    // Get the quote id
    let quote_id = entry.id;

    // Get the quote message
    let message = ctx
        .channel_id()
        .message(
            ctx,
            MessageId::from_str(&entry.message_id).context("Invalid message ID")?,
        )
        .await
        .context("Could not get message")?;

    // Get the quote message link
    let message_link = message.link();

    // Get the quote message content
    let message_content = message.content.clone();

    // Only get first line of message if multi-line
    let message_content = if message_content.lines().count() > 1 {
        format!(
            "{}...",
            &message_content
                .lines()
                .next()
                .ok_or(anyhow!("No lines found"))?
                .to_owned()
        )
    } else {
        message_content
    };

    // Shorten message content if needed
    let message_content = if message_content.chars().count() > 100 {
        format!("{}...", &message_content[..100])
    } else {
        message_content
    };

    // Get the author's name
    let author_name = message.author.display_name();

    // Create the preview
    Ok(localize_message!(
        "command.quote.view.preview",
        &language,
        quote_id,
        author_name,
        message_content,
        message_link
    )
    .await
    .context("Failed to localize message")?)
}

#[command(
    slash_command,
    name_localized("en-US", "view"),
    name_localized("es-419", "ver"),
    description_localized("en-US", "View some quotes"),
    description_localized("es-419", "Ver algunas citas")
)]
pub(super) async fn view(
    ctx: Context<'_>,

    #[name_localized("en-US", "limit")]
    #[name_localized("es-419", "limite")]
    #[description_localized("en-US", "The number of quotes to view")]
    #[description_localized("es-419", "El número de citas a ver")]
    limit: Option<u8>,

    #[name_localized("en-US", "author")]
    #[name_localized("es-419", "autor")]
    #[description_localized("en-US", "The author of the quotes to view")]
    #[description_localized("es-419", "El autor de las citas a ver")]
    author: Option<Member>,
    // #[name_localized("en-US", "start_date")]
    // #[name_localized("es-419", "fecha_inicio")]
    // #[description_localized("en-US", "The start date of the quotes to view")]
    // #[description_localized("es-419", "La fecha de inicio de las citas a ver")]
    // start_date: Option<NaiveDateTime>,

    // #[name_localized("en-US", "end_date")]
    // #[name_localized("es-419", "fecha_fin")]
    // #[description_localized("en-US", "The end date of the quotes to view")]
    // #[description_localized("es-419", "La fecha de fin de las citas a ver")]
    // end_date: Option<NaiveDateTime>,
) -> Result {
    // Get the context settings
    let context_settings = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?;

    // Defer the response
    ctx.defer().await.context("Failed to defer response")?;

    // Create the filters struct
    let mut filters = db::quotebook::EntryFilters::new();

    // Add filters as needed
    filters = filters.guild_id(
        ctx.guild_id()
            .ok_or(anyhow!("No guild ID found"))?
            .to_string(),
    );
    if let Some(limit) = limit {
        filters = filters.limit(limit);
    }
    if let Some(author) = author {
        filters = filters.author_id(author.user.id.to_string());
    }
    // if let Some(start_date) = start_date {
    //     filters = filters.datetime_start(start_date);
    // }
    // if let Some(end_date) = end_date {
    //     filters = filters.datetime_end(end_date);
    // }

    // Get the entries from the database
    let entries = db::quotebook::get_entries(&ctx.data().db, filters)
        .await
        .context("Could not get entries from database")?;

    // Branch bases on entries being empty or not
    if entries.is_empty() {
        // Say that there are no quotes
        ctx.say(
            localize_message!(
                "command.quote.view.response.empty",
                &context_settings.language
            )
            .await
            .context("Failed to localize message")?,
        )
        .await
        .context("Failed to send message")?;
    } else {
        // Get the message previews
        let mut previews = Vec::new();
        for entry in entries {
            previews.push(create_quote_preview(ctx, entry).await?);
        }

        // Respond with the quotes
        ctx.say(
            localize_message!(
                "command.quote.view.response.previews",
                &context_settings.language,
                previews.join("\n")
            )
            .await
            .context("Failed to localize message")?,
        )
        .await
        .context("Failed to send message")?;
    }

    // Return ok
    Ok(())
}
