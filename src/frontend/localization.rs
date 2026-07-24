/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use fluent::{FluentArgs, FluentBundle, FluentResource};
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

pub struct Localization {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    current_locale: String,
    available_locales: Vec<String>,
}

impl Localization {
    /// Create a new Localization instance with embedded locales
    pub fn new(default_locale: &str) -> Result<Self, String> {
        let mut bundles = HashMap::new();
        let mut available_locales = Vec::new();

        // Load all embedded locales
        let locales = Self::embedded_locales();

        for (locale_code, ftl_content) in locales {
            match Self::create_bundle(&locale_code, ftl_content) {
                Ok(bundle) => {
                    bundles.insert(locale_code.clone(), bundle);
                    available_locales.push(locale_code);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load locale: {}", e);
                }
            }
        }

        if bundles.is_empty() {
            return Err("No locales could be loaded".to_string());
        }

        available_locales.sort();

        let normalized_locale = Self::normalize_locale(default_locale);
        let current_locale = if bundles.contains_key(&normalized_locale) {
            normalized_locale
        } else {
            bundles.keys().next().unwrap().clone()
        };

        Ok(Self {
            bundles,
            current_locale,
            available_locales,
        })
    }

    /// All embedded locale resources
    /// TODO: Hardcoding locale dir is not good
    fn embedded_locales() -> Vec<(String, &'static str)> {
        vec![
            (
                "en".to_string(),
                include_str!("../../locales/en-US/cli.ftl"),
            ),
            (
                "ru".to_string(),
                include_str!("../../locales/ru-RU/cli.ftl"),
            ),
        ]
    }

    /// Create a FluentBundle for a given locale
    fn create_bundle(
        locale_code: &str,
        ftl_content: &str,
    ) -> Result<FluentBundle<FluentResource>, String> {
        if ftl_content.is_empty() {
            return Err(format!("Empty locale content for '{}'", locale_code));
        }

        let lang_id = Self::locale_to_language_id(locale_code)
            .map_err(|e| format!("Failed to parse language ID for '{}': {}", locale_code, e))?;

        let resource = FluentResource::try_new(ftl_content.to_string())
            .map_err(|e| format!("Failed to parse messages for '{}': {:?}", locale_code, e))?;

        let mut bundle = FluentBundle::new(vec![lang_id]);
        bundle
            .add_resource(resource)
            .map_err(|e| format!("Failed to add resource for '{}': {:?}", locale_code, e))?;

        Ok(bundle)
    }

    /// Map locale code to LanguageIdentifier
    fn locale_to_language_id(locale_code: &str) -> Result<LanguageIdentifier, String> {
        // Try parsing directly first (e.g., "en-US")
        if let Ok(lang_id) = locale_code.parse::<LanguageIdentifier>() {
            return Ok(lang_id);
        }

        // If that fails, append a region code
        // TODO: Hardcode again?
        let lang_id_str = match locale_code {
            "en" => "en-US",
            "ru" => "ru-RU",
            other => other,
        };

        lang_id_str
            .parse::<LanguageIdentifier>()
            .map_err(|e| format!("Invalid language ID '{}': {}", lang_id_str, e))
    }

    /// Normalize locale string to short code
    fn normalize_locale(locale: &str) -> String {
        locale.split('-').next().unwrap_or("en").to_lowercase()
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
        let normalized = Self::normalize_locale(locale);

        if self.bundles.contains_key(&normalized) {
            self.current_locale = normalized;
            Ok(())
        } else {
            Err(format!(
                "Unsupported locale: '{}'. Available: {}",
                locale,
                self.available_locales.join(", ")
            ))
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
