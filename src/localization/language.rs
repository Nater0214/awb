use std::str::FromStr;

use strum::EnumIter;

use crate::db;

#[derive(Debug, Clone)]
pub(crate) struct LanguageParseError {
    input: String,
}

impl LanguageParseError {
    pub(crate) fn new(input: String) -> Self {
        Self { input }
    }
}

impl std::fmt::Display for LanguageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unexpected language: {}", self.input)
    }
}

impl std::error::Error for LanguageParseError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, EnumIter)]
pub(crate) enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::English => "en_US",
            Self::Spanish => "es_419",
        }
    }
}

impl FromStr for Language {
    type Err = LanguageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en-US" => Ok(Self::English),
            "es-419" => Ok(Self::Spanish),
            other => Err(LanguageParseError::new(other.to_owned())),
        }
    }
}

impl From<db::guild_settings::Language> for Language {
    fn from(value: db::guild_settings::Language) -> Self {
        match value {
            db::guild_settings::Language::English => Self::English,
            db::guild_settings::Language::Spanish => Self::Spanish,
        }
    }
}

impl From<db::user_settings::Language> for Language {
    fn from(value: db::user_settings::Language) -> Self {
        match value {
            db::user_settings::Language::English => Self::English,
            db::user_settings::Language::Spanish => Self::Spanish,
        }
    }
}

impl From<Language> for db::guild_settings::Language {
    fn from(value: Language) -> Self {
        match value {
            Language::English => Self::English,
            Language::Spanish => Self::Spanish,
        }
    }
}

impl From<Language> for db::user_settings::Language {
    fn from(value: Language) -> Self {
        match value {
            Language::English => Self::English,
            Language::Spanish => Self::Spanish,
        }
    }
}

impl AsRef<Language> for Language {
    fn as_ref(&self) -> &Language {
        self
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::English => write!(f, "English"),
            Self::Spanish => write!(f, "Spanish"),
        }
    }
}
