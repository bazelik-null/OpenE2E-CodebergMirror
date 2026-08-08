/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

pub enum Command {
    // Misc
    Exit,
    Help,
    Lang { language: String },

    // User control
    NewUser { name: String, password: String },
    DeleteUser { name: String },
    LogoutUser,
    LoginUser { name: String, password: String },
    ListUsers,

    // Session control
    NewSession { name: String },
    DeleteSession { name: String },
    CloseSession,
    OpenSession { name: String },
    ListSessions,

    // Message control
    Encrypt { text: String },
    Decrypt { text: String },
    History,
}

pub fn scan_commands(input: &str) -> Option<Command> {
    let tokens = parse_quoted_input(input);
    let mut tokens_iter = tokens.iter().map(|s| s.as_str());

    match tokens_iter.next()? {
        "exit" => Some(Command::Exit),
        "help" => Some(Command::Help),
        "lang" => {
            let language = tokens.get(1)?.to_string();
            Some(Command::Lang { language })
        }
        "e" => {
            let text = tokens_iter.collect::<Vec<_>>().join(" ");
            (!text.is_empty()).then_some(Command::Encrypt { text })
        }
        "d" => {
            let text = tokens_iter.collect::<Vec<_>>().join(" ");
            (!text.is_empty()).then_some(Command::Decrypt { text })
        }
        "history" => Some(Command::History),
        "s" => scan_subcommand(&mut tokens_iter, scan_session_commands),
        "u" => scan_subcommand(&mut tokens_iter, scan_user_commands),
        _ => None,
    }
}

fn scan_subcommand<F>(tokens: &mut dyn Iterator<Item = &str>, handler: F) -> Option<Command>
where
    F: Fn(&str, Vec<&str>) -> Option<Command>,
{
    let subcommand = tokens.next().unwrap_or("");
    let arguments: Vec<&str> = tokens.collect();
    handler(subcommand, arguments)
}

fn parse_quoted_input(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            _ => current_token.push(ch),
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}

fn scan_session_commands(subcommand: &str, arguments: Vec<&str>) -> Option<Command> {
    match subcommand {
        "new" => arguments.first().map(|&name| Command::NewSession {
            name: name.to_string(),
        }),
        "delete" => arguments.first().map(|&name| Command::DeleteSession {
            name: name.to_string(),
        }),
        "close" => Some(Command::CloseSession),
        "open" => arguments.first().map(|&name| Command::OpenSession {
            name: name.to_string(),
        }),
        "list" => Some(Command::ListSessions),
        _ => None,
    }
}

fn scan_user_commands(subcommand: &str, arguments: Vec<&str>) -> Option<Command> {
    match subcommand {
        "new" => {
            let [name, password, ..] = arguments.as_slice() else {
                return None;
            };
            Some(Command::NewUser {
                name: name.to_string(),
                password: password.to_string(),
            })
        }
        "delete" => arguments.first().map(|&name| Command::DeleteUser {
            name: name.to_string(),
        }),
        "logout" => Some(Command::LogoutUser),
        "login" => {
            let [name, password, ..] = arguments.as_slice() else {
                return None;
            };
            Some(Command::LoginUser {
                name: name.to_string(),
                password: password.to_string(),
            })
        }
        "list" => Some(Command::ListUsers),
        _ => None,
    }
}
