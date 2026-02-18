use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use gpui::SharedString;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/calmui_i18n_generated.rs"));
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum Locale {
    #[default]
    System,
    Tag(String),
}

impl From<String> for Locale {
    fn from(value: String) -> Self {
        if value.trim().eq_ignore_ascii_case("system") {
            return Self::System;
        }
        Self::Tag(value.trim().to_string())
    }
}

impl From<&str> for Locale {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

#[derive(Clone)]
pub struct I18nManager {
    catalog: Arc<I18nCatalog>,
    locale: Arc<RwLock<Locale>>,
}

impl Default for I18nManager {
    fn default() -> Self {
        Self::new()
    }
}

impl I18nManager {
    pub fn new() -> Self {
        Self {
            catalog: Arc::new(I18nCatalog::load()),
            locale: Arc::new(RwLock::new(Locale::System)),
        }
    }

    pub fn locale(&self) -> Locale {
        self.locale
            .read()
            .expect("i18n locale state poisoned")
            .clone()
    }

    pub fn set_locale(&self, locale: impl Into<Locale>) {
        *self.locale.write().expect("i18n locale state poisoned") = locale.into();
    }

    pub fn default_locale(&self) -> &'static str {
        self.catalog.default_locale
    }

    pub fn resolved_locale(&self) -> &'static str {
        self.catalog
            .resolve_locale(self.requested_locale().as_deref())
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.lookup(key).is_some()
    }

    pub fn t(&self, key: &str) -> SharedString {
        if let Some(value) = self.lookup(key) {
            value.into()
        } else {
            key.to_string().into()
        }
    }

    pub fn t_with(&self, key: &str, params: &[(&str, &str)]) -> SharedString {
        let template = self.lookup(key);
        if params.is_empty() {
            return if let Some(value) = template {
                value.into()
            } else {
                key.to_string().into()
            };
        }

        let raw = template.unwrap_or(key);
        format_template(raw, params).into()
    }

    fn requested_locale(&self) -> Option<String> {
        match self.locale() {
            Locale::System => sys_locale::get_locale(),
            Locale::Tag(tag) => Some(tag),
        }
    }

    fn lookup(&self, key: &str) -> Option<&'static str> {
        let resolved = self.resolved_locale();
        self.catalog.lookup(resolved, key)
    }
}

struct I18nCatalog {
    default_locale: &'static str,
    locales: HashMap<&'static str, HashMap<&'static str, &'static str>>,
    normalized_locale_lookup: HashMap<String, &'static str>,
    language_lookup: HashMap<String, &'static str>,
}

impl I18nCatalog {
    fn load() -> Self {
        let mut locales = HashMap::new();
        let mut normalized_locale_lookup = HashMap::new();
        let mut language_lookup = HashMap::new();
        let mut ambiguous_languages = HashSet::new();

        for (locale, entries) in generated::LOCALES.iter().copied() {
            let normalized = normalize_locale_tag(locale);
            normalized_locale_lookup.insert(normalized.clone(), locale);

            let language = normalized.split('-').next().unwrap_or_default().to_string();
            if let Some(existing) = language_lookup.get(&language) {
                if *existing != locale {
                    ambiguous_languages.insert(language.clone());
                }
            } else {
                language_lookup.insert(language, locale);
            }

            locales.insert(locale, entries.iter().copied().collect::<HashMap<_, _>>());
        }

        for language in ambiguous_languages {
            language_lookup.remove(&language);
        }

        if !locales.contains_key(generated::DEFAULT_LOCALE) {
            locales.insert(generated::DEFAULT_LOCALE, HashMap::new());
            normalized_locale_lookup.insert(
                normalize_locale_tag(generated::DEFAULT_LOCALE),
                generated::DEFAULT_LOCALE,
            );
            let language = normalize_locale_tag(generated::DEFAULT_LOCALE)
                .split('-')
                .next()
                .unwrap_or_default()
                .to_string();
            language_lookup
                .entry(language)
                .or_insert(generated::DEFAULT_LOCALE);
        }

        Self {
            default_locale: generated::DEFAULT_LOCALE,
            locales,
            normalized_locale_lookup,
            language_lookup,
        }
    }

    fn resolve_locale(&self, requested: Option<&str>) -> &'static str {
        let Some(requested) = requested else {
            return self.default_locale;
        };

        let normalized = normalize_locale_tag(requested);
        if let Some(locale) = self.normalized_locale_lookup.get(&normalized) {
            return locale;
        }

        let language = normalized.split('-').next().unwrap_or_default();
        if let Some(locale) = self.language_lookup.get(language) {
            return locale;
        }

        self.default_locale
    }

    fn lookup(&self, locale: &'static str, key: &str) -> Option<&'static str> {
        self.locales
            .get(locale)
            .and_then(|entries| entries.get(key).copied())
    }
}

fn normalize_locale_tag(tag: &str) -> String {
    let trimmed = tag.trim();
    let without_encoding = trimmed.split('.').next().unwrap_or(trimmed);
    let without_variant = without_encoding
        .split('@')
        .next()
        .unwrap_or(without_encoding);
    without_variant
        .replace('_', "-")
        .split('-')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

fn format_template(template: &str, params: &[(&str, &str)]) -> String {
    let values = params.iter().copied().collect::<HashMap<&str, &str>>();
    let mut output = String::with_capacity(template.len());
    let mut cursor = 0;

    while cursor < template.len() {
        let tail = &template[cursor..];
        let Some(open_rel) = tail.find('{') else {
            output.push_str(tail);
            break;
        };

        let open = cursor + open_rel;
        output.push_str(&template[cursor..open]);

        let token_start = open + 1;
        let Some(close_rel) = template[token_start..].find('}') else {
            output.push_str(&template[open..]);
            break;
        };
        let close = token_start + close_rel;
        let token = &template[token_start..close];

        if let Some(value) = values.get(token) {
            output.push_str(value);
        } else {
            output.push_str(&template[open..=close]);
        }

        cursor = close + 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::I18nManager;

    #[test]
    fn missing_translation_shows_key() {
        let i18n = I18nManager::new();
        i18n.set_locale("zh-CN");
        assert_eq!(i18n.t("demo.only_en").to_string(), "demo.only_en");
    }

    #[test]
    fn supports_locale_tag_normalization() {
        let i18n = I18nManager::new();
        i18n.set_locale("zh_CN");
        assert_eq!(i18n.resolved_locale(), "zh-CN");
        assert_eq!(i18n.t("common.confirm").to_string(), "确认");
    }

    #[test]
    fn supports_placeholder_interpolation() {
        let i18n = I18nManager::new();
        i18n.set_locale("en-US");
        assert_eq!(
            i18n.t_with("demo.greeting", &[("name", "Ethan")])
                .to_string(),
            "Hello, Ethan"
        );
    }
}
