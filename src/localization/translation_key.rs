use std::str::FromStr;

/// An error parsing a translation key
#[derive(Debug, Clone)]
pub(crate) struct TranslationKeyParseError {
    input_string: String,
}

impl std::fmt::Display for TranslationKeyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid translation key: {}", self.input_string)
    }
}

impl std::error::Error for TranslationKeyParseError {}

/// A key used for a translation
#[derive(Debug, Clone)]
pub(crate) struct TranslationKey {
    components: Vec<String>,
}

impl TranslationKey {
    pub(crate) fn new() -> Self {
        Self { components: vec![] }
    }

    pub(crate) fn add_component(&mut self, component: impl AsRef<str>) {
        self.components.push(component.as_ref().to_owned());
    }

    pub(crate) fn as_string(&self) -> String {
        self.components.join(".")
    }

    pub(crate) fn as_slice(&self) -> &[String] {
        self.components.as_slice()
    }
}

impl FromStr for TranslationKey {
    type Err = TranslationKeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut key = Self::new();
        for component in s.split('.') {
            if component.is_empty() {
                return Err(TranslationKeyParseError {
                    input_string: s.to_owned(),
                });
            }
            key.add_component(component);
        }
        Ok(key)
    }
}

impl AsRef<TranslationKey> for TranslationKey {
    fn as_ref(&self) -> &TranslationKey {
        self
    }
}
