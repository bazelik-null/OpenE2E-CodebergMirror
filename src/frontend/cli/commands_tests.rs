/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Unit tests for [`scan_commands`] — the CLI input parser.
//!
//! These pin down the mapping from raw input lines to [`Command`] variants:
//! sub-commands (`u`/`s`), required arguments, the "rest of line" behaviour of
//! `e`/`d`, quoted arguments, and rejection of malformed input.
//!
//! `Command` deliberately has no `PartialEq`, so tests match on variants with
//! `matches!` / `match` instead of `assert_eq!`.

use super::*;

#[test]
fn simple_commands() {
    assert!(matches!(scan_commands("exit"), Some(Command::Exit)));
    assert!(matches!(scan_commands("help"), Some(Command::Help)));
    assert!(matches!(scan_commands("history"), Some(Command::History)));
}

#[test]
fn unknown_and_empty_are_none() {
    assert!(scan_commands("").is_none());
    assert!(scan_commands("nonsense").is_none());
    assert!(scan_commands("lang").is_none()); // missing argument
}

#[test]
fn lang_command() {
    match scan_commands("lang ru") {
        Some(Command::Lang { language }) => assert_eq!(language, "ru"),
        _ => panic!("expected Lang"),
    }
}

#[test]
fn encrypt_joins_rest_of_line() {
    // `e`/`d` take the entire remainder as the payload, not just one token.
    match scan_commands("e hello world foo") {
        Some(Command::Encrypt { text }) => assert_eq!(text, "hello world foo"),
        _ => panic!("expected Encrypt"),
    }
    assert!(scan_commands("e").is_none()); // no text
}

#[test]
fn user_new_requires_name_and_password() {
    match scan_commands("u new bazya secret") {
        Some(Command::NewUser { name, password }) => {
            assert_eq!(name, "bazya");
            assert_eq!(password, "secret");
        }
        _ => panic!("expected NewUser"),
    }
    assert!(scan_commands("u new bazya").is_none()); // missing password
}

#[test]
fn user_login_and_management() {
    assert!(matches!(
        scan_commands("u login bazya secret"),
        Some(Command::LoginUser { .. })
    ));
    assert!(matches!(
        scan_commands("u logout"),
        Some(Command::LogoutUser)
    ));
    assert!(matches!(scan_commands("u list"), Some(Command::ListUsers)));
    match scan_commands("u delete bazya") {
        Some(Command::DeleteUser { name }) => assert_eq!(name, "bazya"),
        _ => panic!("expected DeleteUser"),
    }
}

#[test]
fn session_commands() {
    match scan_commands("s new demo") {
        Some(Command::NewSession { name }) => assert_eq!(name, "demo"),
        _ => panic!("expected NewSession"),
    }
    assert!(matches!(
        scan_commands("s open demo"),
        Some(Command::OpenSession { .. })
    ));
    assert!(matches!(
        scan_commands("s delete demo"),
        Some(Command::DeleteSession { .. })
    ));
    assert!(matches!(
        scan_commands("s close"),
        Some(Command::CloseSession)
    ));
    assert!(matches!(
        scan_commands("s list"),
        Some(Command::ListSessions)
    ));
}

#[test]
fn quoted_arguments_keep_spaces() {
    // Double quotes let a single argument contain spaces.
    match scan_commands("u new \"bazya smith\" pass word") {
        Some(Command::NewUser { name, password }) => {
            assert_eq!(name, "bazya smith");
            assert_eq!(password, "pass");
        }
        _ => panic!("expected NewUser with quoted name"),
    }
}
