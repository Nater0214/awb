use anyhow::Context as _;
use sea_orm::prelude::*;

use crate::{commands::Context, localization::Language};

use super::{guild, user};

/// Represents the settings of a context
#[derive(Debug, Clone)]
pub(crate) struct ContextSettings {
    pub language: Language,
}

/// Get the settings for a context
pub(crate) async fn get_context_settings(
    ctx: &Context<'_>,
    db: &DbConn,
) -> Result<ContextSettings, anyhow::Error> {
    // Get the user's settings
    let user_settings = user::get_user_settings(ctx, db)
        .await
        .context("Could not get user settings from the database")?;

    // Get the guild's settings
    let guild_settings = guild::get_guild_settings(ctx, db)
        .await
        .context("Could not get guild settings from the database")?;

    // Combine the settings to get the context settings
    if let Some(guild_settings) = guild_settings {
        Ok(ContextSettings {
            language: user_settings
                .language
                .unwrap_or(guild_settings.language.unwrap_or_default()),
        })
    } else {
        Ok(ContextSettings {
            language: user_settings.language.unwrap_or_default(),
        })
    }
}
