/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Unit tests for [`Localization`] — the Fluent-backed translation layer.
///
/// Covers translation of a known key, the fallback-to-key behaviour for unknown keys, locale switching, rejection of unsupported locales, and argument interpolation.
use super::*;

#[test]
fn known_key_is_translated() {
    let loc = Localization::new("en").unwrap();
    assert_eq!(loc.get("gui-log-in"), "Log in");
}

#[test]
fn unknown_key_falls_back_to_key() {
    let loc = Localization::new("en").unwrap();
    assert_eq!(loc.get("no-such-key"), "no-such-key");
}

#[test]
fn switching_locale_changes_translation() {
    let mut loc = Localization::new("en").unwrap();
    assert_eq!(loc.get("gui-log-in"), "Log in");

    loc.set_locale("ru").unwrap();
    assert_eq!(loc.get("gui-log-in"), "Войти");
}

#[test]
fn set_unsupported_locale_fails() {
    let mut loc = Localization::new("en").unwrap();
    assert!(loc.set_locale("xx").is_err());
}

#[test]
fn unknown_default_locale_still_loads() {
    // An unknown default falls back to an available locale rather than failing.
    assert!(Localization::new("zz").is_ok());
}

#[test]
fn arguments_are_interpolated() {
    let loc = Localization::new("en").unwrap();
    let args = fluent_args(&[("username", "alice")]);

    let out = loc.get_with_args("user-created", Some(&args));
    assert!(out.contains("alice"), "interpolated output was: {}", out);
}
