use anyhow::Context as _;
use async_recursion::async_recursion;
use sea_orm::{ActiveValue::Set, IntoActiveModel as _, prelude::*};
use serenity::all::UserId;

use crate::{commands::Context, db, localization::Language};

/// Represents the settings of a user
#[derive(Debug, Clone)]
pub(crate) struct UserSettings {
    pub language: Option<Language>,
}

impl From<db::user_settings::Model> for UserSettings {
    fn from(model: db::user_settings::Model) -> Self {
        Self {
            language: model.language.map(|language| language.into()),
        }
    }
}

/// Get the user settings
#[async_recursion]
pub(super) async fn get_user_settings(
    ctx: &Context<'_>,
    db: &DbConn,
) -> Result<UserSettings, anyhow::Error> {
    use db::user_settings::*;

    let user_id = ctx.author().id.to_string();
    let model = Entity::find()
        .filter(Column::UserId.eq(user_id))
        .one(db)
        .await
        .context("Could not get user settings entry in the database")?;
    if let Some(model) = model {
        Ok(model.into())
    } else {
        db::user_settings::create_entry(db, ctx.author().id.to_string())
            .await
            .context("Could not create user settings entry in the database")?;
        get_user_settings(ctx, db).await
    }
}
