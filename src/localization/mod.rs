use std::collections::HashMap;

use anyhow::Context as _;
use serde_yaml::Value;
use tokio::{fs, sync::OnceCell};

pub(crate) use language::Language;
pub(crate) use translation_key::TranslationKey;

use crate::localization::translation_key::TranslationKeyParseError;

mod language;
mod translation_key;

type TranslationsMap = HashMap<Language, Value>;

static TRANSLATIONS: OnceCell<TranslationsMap> = OnceCell::const_new();

#[inline]
async fn get_translations() -> &'static TranslationsMap {
    TRANSLATIONS
        .get_or_init(|| async {
            // Combine language files
            let mut translations = TranslationsMap::new();

            // Load English
            fs::read("./lang/en_US.yaml")
                .await
                .ok()
                .and_then(|content| {
                    serde_yaml::from_slice(content.as_slice())
                        .ok()
                        .map(|value| (Language::English, value))
                })
                .and_then(|(language, value)| translations.insert(language, value));

            // Load Spanish
            fs::read("./lang/es_419.yaml")
                .await
                .ok()
                .and_then(|content| {
                    serde_yaml::from_slice(content.as_slice())
                        .ok()
                        .map(|value| (Language::Spanish, value))
                })
                .and_then(|(language, value)| translations.insert(language, value));

            translations
        })
        .await
}

#[derive(Debug, Clone)]
pub(crate) enum LocalizationError {
    BadKey(TranslationKeyParseError),
    UnknownKey(TranslationKey),
    UnsupportedLanguage(Language),
}

impl From<TranslationKeyParseError> for LocalizationError {
    fn from(err: TranslationKeyParseError) -> Self {
        Self::BadKey(err)
    }
}

impl std::fmt::Display for LocalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadKey(key) => write!(f, "Bad key: {}", key),
            Self::UnknownKey(key) => write!(f, "Unknown key: {}", key.as_string()),
            Self::UnsupportedLanguage(language) => {
                write!(f, "Unsupported language: {}", language.as_str())
            }
        }
    }
}

impl std::error::Error for LocalizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BadKey(key) => Some(key),
            Self::UnknownKey(_) => None,
            Self::UnsupportedLanguage(_) => None,
        }
    }
}

pub(crate) async fn localize_message(
    translation_key: impl AsRef<str>,
    language: impl AsRef<Language>,
    args: &[impl AsRef<str>],
) -> Result<String, anyhow::Error> {
    // Convert arguments
    let translation_key = translation_key.as_ref().parse::<TranslationKey>()?;
    let language = language.as_ref();

    // Get the correct translations set
    let translations = get_translations()
        .await
        .get(&language)
        .ok_or(LocalizationError::UnsupportedLanguage(language.to_owned()))
        .context("Could not get translations")?;

    // Get the correct translation
    let mut current = translations;
    for component in translation_key.as_slice() {
        if let Value::Mapping(mapping) = current {
            current = mapping
                .get(component)
                .ok_or(LocalizationError::UnknownKey(translation_key.clone()))
                .context("Unknown key encountered while descending the tree")?;
        } else {
            return Err(LocalizationError::UnknownKey(translation_key.clone()))
                .context("Unexpected value type while descending the tree");
        }
    }

    // Return it as a string
    if let Value::String(string) = current {
        Ok(args
            .iter()
            .map(|arg| arg.as_ref().replace("\\n", "\n").replace("\\t", "\t"))
            .fold(string.to_owned(), |string, arg| {
                string.replacen("{}", &arg, 1)
            }))
    } else {
        Err(LocalizationError::UnknownKey(translation_key.clone()))
            .context("Unexpected value type when getting translation")
    }
}

#[macro_export]
macro_rules! localize_message {
    ($key:expr, $lang:expr $(,)?) => {
        crate::localization::localize_message($key, $lang, &[] as &[&str])
    };
    ($key:expr, $lang:expr, $($args:expr),* $(,)?) => {
        crate::localization::localize_message($key, $lang, &[$($args),*])
    };
}

#[allow(unused_imports)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_localize_message() {
        assert_eq!(
            localize_message!("command.hello.response", &Language::English)
                .await
                .unwrap(),
            "Hello Nater"
        );
    }
}
