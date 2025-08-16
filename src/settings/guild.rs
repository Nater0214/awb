use anyhow::Context as _;
use async_recursion::async_recursion;
use sea_orm::prelude::*;

use crate::{commands::Context, db, localization::Language};

/// Represents the settings of a guild
#[derive(Debug, Clone)]
pub(crate) struct GuildSettings {
    pub language: Option<Language>,
}

impl From<db::guild_settings::Model> for GuildSettings {
    fn from(model: db::guild_settings::Model) -> Self {
        Self {
            language: model.language.map(|language| language.into()),
        }
    }
}

#[async_recursion]
pub(super) async fn get_guild_settings(
    ctx: &Context<'_>,
    db: &sea_orm::DbConn,
) -> Result<Option<GuildSettings>, anyhow::Error> {
    use db::guild_settings::*;
    let guild_id = ctx.guild().map(|guild| guild.id.to_string());
    if let Some(guild_id) = guild_id {
        let model = Entity::find()
            .filter(Column::GuildId.eq(&guild_id))
            .one(db)
            .await
            .context("Could not get guild settings entry in the database")?;
        if let Some(model) = model {
            Ok(Some(model.into()))
        } else {
            db::guild_settings::create_entry(db, &guild_id)
                .await
                .context("Could not create guild settings entry in the database")?;
            get_guild_settings(ctx, db).await
        }
    } else {
        Ok(None)
    }
}
