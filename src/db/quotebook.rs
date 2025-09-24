use anyhow::{Context, anyhow};
use chrono::NaiveDateTime;
use sea_orm::{QueryOrder as _, QuerySelect as _, prelude::*};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "quotebook")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub message_id: String,
    pub guild_id: String,
    pub author_id: String,
    pub datetime: NaiveDateTime,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Create an entry in the quotebook table
pub(crate) async fn create_entry(
    db: &DbConn,
    message_id: impl AsRef<str>,
    guild_id: impl AsRef<str>,
    author_id: impl AsRef<str>,
    datetime: NaiveDateTime,
) -> Result<(), anyhow::Error> {
    // Use the active value types
    use sea_orm::ActiveValue::*;

    // Get ids as strings
    let message_id = message_id.as_ref().to_string();
    let guild_id = guild_id.as_ref().to_string();
    let author_id = author_id.as_ref().to_string();

    // Create a new table entry
    let new_entry = ActiveModel {
        message_id: Set(message_id),
        guild_id: Set(guild_id),
        author_id: Set(author_id),
        datetime: Set(datetime),
        ..Default::default()
    };

    // Insert the new entry into the table
    new_entry
        .insert(db)
        .await
        .context("Could not insert new entry into quotebook table")?;

    // Return ok
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct EntryFilters {
    _message_id: Option<String>,
    _guild_id: Option<String>,
    _author_id: Option<String>,
    _datetime_start: Option<NaiveDateTime>,
    _datetime_end: Option<NaiveDateTime>,
    _limit: Option<u8>,
}

#[allow(dead_code)]
impl EntryFilters {
    pub(crate) fn new() -> Self {
        Self {
            _message_id: None,
            _guild_id: None,
            _author_id: None,
            _datetime_start: None,
            _datetime_end: None,
            _limit: None,
        }
    }

    pub(crate) fn message_id(mut self, message_id: impl AsRef<str>) -> Self {
        self._message_id = Some(message_id.as_ref().to_string());
        self
    }

    pub(crate) fn guild_id(mut self, guild_id: impl AsRef<str>) -> Self {
        self._guild_id = Some(guild_id.as_ref().to_string());
        self
    }

    pub(crate) fn author_id(mut self, author_id: impl AsRef<str>) -> Self {
        self._author_id = Some(author_id.as_ref().to_string());
        self
    }

    pub(crate) fn datetime_start(mut self, datetime: NaiveDateTime) -> Self {
        self._datetime_start = Some(datetime);
        self
    }

    pub(crate) fn datetime_end(mut self, datetime: NaiveDateTime) -> Self {
        self._datetime_end = Some(datetime);
        self
    }

    pub(crate) fn limit(mut self, limit: u8) -> Self {
        self._limit = Some(limit);
        self
    }
}

impl Default for EntryFilters {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<EntryFilters> for EntryFilters {
    fn as_ref(&self) -> &EntryFilters {
        self
    }
}

/// Get entries from the quotebook table
pub(crate) async fn get_entries(
    db: &DbConn,
    filters: impl AsRef<EntryFilters>,
) -> Result<Vec<Model>, anyhow::Error> {
    // Use the columns for ease of writing
    use Column::*;

    // Get the filters
    let filters = filters.as_ref();

    // Create the foundation of the query
    let mut query = Entity::find();

    // Add each filter if it exists
    if let Some(guild_id) = &filters._guild_id {
        query = query.filter(GuildId.eq(guild_id));
    } else {
        return Err(anyhow!("No guild ID provided")).context("No guild ID provided");
    }
    if let Some(message_id) = &filters._message_id {
        query = query.filter(MessageId.eq(message_id));
    } else {
        if let Some(author_id) = &filters._author_id {
            query = query.filter(AuthorId.eq(author_id));
        }
        if let Some(datetime_start) = &filters._datetime_start {
            query = query.filter(Datetime.gte(datetime_start.to_owned()));
        }
        if let Some(datetime_end) = &filters._datetime_end {
            query = query.filter(Datetime.lte(datetime_end.to_owned()));
        }
    }

    // Add the limit
    if let Some(limit) = &filters._limit {
        query = query.limit(*limit as u64);
    } else {
        query = query.limit(5);
    }

    // Order by the date
    query = query.order_by_desc(Datetime);

    // Execute the query
    let entries = query
        .all(db)
        .await
        .context("Could not get database entries in quotebook table")?;

    // Return the entries
    Ok(entries)
}
