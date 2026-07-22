use fluent::{FluentArgs, FluentBundle, FluentResource};
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

pub struct Localization {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    current_locale: String,
    available_locales: Vec<String>,
}

impl Localization {
    pub fn new(default_locale: &str) -> Result<Self, String> {
        let mut bundles = HashMap::new();
        let mut available_locales = Vec::new();

        // Load English
        let en_ftl = include_str!("../../locales/en-US/messages.ftl");
        let en_resource = FluentResource::try_new(en_ftl.to_string())
            .map_err(|e| format!("Failed to parse English messages: {:?}", e))?;
        let mut en_bundle = FluentBundle::new(vec![
            "en-US"
                .parse::<LanguageIdentifier>()
                .map_err(|e| format!("Failed to parse en-US language ID: {}", e))?,
        ]);
        en_bundle
            .add_resource(en_resource)
            .map_err(|e| format!("Failed to add English resource: {:?}", e))?;
        bundles.insert("en".to_string(), en_bundle);
        available_locales.push("en".to_string());

        // Load Russian
        let ru_ftl = include_str!("../../locales/ru-RU/messages.ftl");
        let ru_resource = FluentResource::try_new(ru_ftl.to_string())
            .map_err(|e| format!("Failed to parse Russian messages: {:?}", e))?;
        let mut ru_bundle = FluentBundle::new(vec![
            "ru-RU"
                .parse::<LanguageIdentifier>()
                .map_err(|e| format!("Failed to parse ru-RU language ID: {}", e))?,
        ]);
        ru_bundle
            .add_resource(ru_resource)
            .map_err(|e| format!("Failed to add Russian resource: {:?}", e))?;
        bundles.insert("ru".to_string(), ru_bundle);
        available_locales.push("ru".to_string());

        let normalized_locale = if default_locale == "ru" || default_locale == "ru-RU" {
            "ru".to_string()
        } else {
            "en".to_string()
        };

        Ok(Self {
            bundles,
            current_locale: normalized_locale,
            available_locales,
        })
    }

    /// Get a translated string with no arguments
    pub fn get(&self, key: &str) -> String {
        self.get_with_args(key, None)
    }

    /// Get a translated string with arguments
    pub fn get_with_args(&self, key: &str, args: Option<&FluentArgs>) -> String {
        if let Some(bundle) = self.bundles.get(&self.current_locale)
            && let Some(msg) = bundle.get_message(key)
            && let Some(pattern) = msg.value()
        {
            let mut errors = vec![];
            let value = bundle.format_pattern(pattern, args, &mut errors);
            return value.to_string();
        }
        key.to_string() // Fallback to key if translation not found
    }

    /// Set the current locale
    pub fn set_locale(&mut self, locale: &str) -> Result<(), String> {
        let normalized = if locale == "ru" || locale == "ru-RU" {
            "ru".to_string()
        } else if locale == "en" || locale == "en-US" {
            "en".to_string()
        } else {
            return Err(format!(
                "Unsupported language: {}. Available: {}",
                locale,
                self.available_locales.join(", ")
            ));
        };

        if self.bundles.contains_key(&normalized) {
            self.current_locale = normalized;
            Ok(())
        } else {
            Err(format!("Language {} not available", locale))
        }
    }
}

/// Create FluentArgs from string pairs
pub fn fluent_args(pairs: &[(&str, &str)]) -> FluentArgs<'static> {
    let mut args = FluentArgs::new();
    for (key, value) in pairs {
        args.set(key.to_string(), value.to_string());
    }
    args
}
