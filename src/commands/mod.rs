use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use serenity::all::{MessageId, UserId};

pub(self) mod general;
pub(self) mod settings;
pub(self) mod shadowbox;

/// The error type for commands
pub(crate) type Error = anyhow::Error;

/// The result type for commands
pub(crate) type Result = anyhow::Result<(), Error>;

/// The context for commands
pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;

/// The data for commands
#[derive(Debug, Clone)]
pub(crate) struct Data {
    pub(crate) db: DatabaseConnection,
    pub(self) menu_selections: DashMap<(MessageId, UserId), String>,
}

impl Data {
    pub(crate) fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            menu_selections: DashMap::new(),
        }
    }
}

pub(crate) fn get_all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![]
        .into_iter()
        .chain(general::get_all_commands())
        .chain(settings::get_all_commands())
        .chain(shadowbox::get_all_commands())
        .collect()
}
