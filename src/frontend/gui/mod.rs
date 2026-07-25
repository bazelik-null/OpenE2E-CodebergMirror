/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ModelRc, SharedString, VecModel};

use crate::backend::managers::user_manager::UserManager;
use crate::backend::objects::session::SessionInstance;

slint::include_modules!();

/// Launches the Slint GUI frontend.
pub fn run() -> Result<(), String> {
    if std::env::var_os("SLINT_BACKEND").is_none() {
        unsafe { std::env::set_var("SLINT_BACKEND", "winit-software") };
    }

    let ui = MainWindow::new().map_err(|e| e.to_string())?;
    let manager = Rc::new(RefCell::new(UserManager::new()?));

    // Persistent, Rust-owned model backing the chat transcript.
    let messages: Rc<VecModel<ChatLine>> = Rc::new(VecModel::default());
    ui.set_messages(ModelRc::from(messages.clone()));

    // ---- Log in ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        ui.on_login(move |name, password| {
            let ui = ui_weak.unwrap();
            if name.trim().is_empty() || password.is_empty() {
                ui.set_status_text("Username and password are required".into());
                return;
            }
            let mut mgr = manager.borrow_mut();
            match mgr.login(name.as_str(), password.as_str()) {
                Ok(()) => {
                    let sessions = session_names(&mgr);
                    drop(mgr);
                    ui.set_logged_in(true);
                    ui.set_current_user(name);
                    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                    ui.set_status_text("Logged in".into());
                }
                Err(e) => {
                    drop(mgr);
                    ui.set_status_text(e.into());
                }
            }
        });
    }

    // ---- Create user (and immediately log in) ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        ui.on_create_user(move |name, password| {
            let ui = ui_weak.unwrap();
            if name.trim().is_empty() || password.is_empty() {
                ui.set_status_text("Username and password are required".into());
                return;
            }
            let mut mgr = manager.borrow_mut();
            let result = mgr
                .new_user(name.as_str(), password.as_str())
                .and_then(|()| mgr.login(name.as_str(), password.as_str()))
                .and_then(|()| mgr.autosave());
            match result {
                Ok(()) => {
                    let sessions = session_names(&mgr);
                    drop(mgr);
                    ui.set_logged_in(true);
                    ui.set_current_user(name);
                    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                    ui.set_status_text("User created".into());
                }
                Err(e) => {
                    drop(mgr);
                    ui.set_status_text(e.into());
                }
            }
        });
    }

    // ---- Log out ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let messages = messages.clone();
        ui.on_logout(move || {
            let ui = ui_weak.unwrap();
            let mut mgr = manager.borrow_mut();
            mgr.logout();
            let _ = mgr.autosave();
            drop(mgr);

            messages.set_vec(Vec::new());
            ui.set_logged_in(false);
            ui.set_creating_session(false);
            ui.set_current_user(SharedString::new());
            ui.set_current_session(SharedString::new());
            ui.set_output_text(SharedString::new());
            ui.set_my_keys(SharedString::new());
            ui.set_init_message(SharedString::new());
            ui.set_sessions(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
            ui.set_status_text("Logged out".into());
        });
    }

    // ---- Select session (load its history) ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let messages = messages.clone();
        ui.on_select_session(move |name| {
            let ui = ui_weak.unwrap();
            let mut mgr = manager.borrow_mut();
            let outcome = mgr
                .get_current_user_mut()
                .ok_or_else(|| "No user selected".to_string())
                .and_then(|user| user.session_manager.select_session(name.as_str()));
            match outcome {
                Ok(()) => {
                    let lines = history_lines(&mgr);
                    drop(mgr);
                    ui.set_current_session(name);
                    messages.set_vec(lines);
                    ui.set_status_text("Session opened".into());
                }
                Err(e) => {
                    drop(mgr);
                    ui.set_status_text(e.into());
                }
            }
        });
    }

    // ---- Generate our key bundle for a new session ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        ui.on_session_gen_keys(move || {
            let ui = ui_weak.unwrap();
            let mut mgr = manager.borrow_mut();
            let result = mgr
                .get_current_user_mut()
                .ok_or_else(|| "No user selected".to_string())
                .and_then(|user| SessionInstance::generate_keys(&mut user.account));
            drop(mgr);
            match result {
                Ok(keys) => {
                    ui.set_my_keys(keys.into());
                    ui.set_status_text(
                        "Keys generated — send them to your peer, then paste theirs".into(),
                    );
                }
                Err(e) => ui.set_status_text(e.into()),
            }
        });
    }

    // ---- Establish a session from the exchanged keys ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let messages = messages.clone();
        ui.on_session_create(move |name, is_inbound, peer_keys, first_msg| {
            let ui = ui_weak.unwrap();
            if name.trim().is_empty() {
                ui.set_status_text("Session name is required".into());
                return;
            }
            if peer_keys.trim().is_empty() {
                ui.set_status_text("Peer's keys are required".into());
                return;
            }
            if is_inbound && first_msg.trim().is_empty() {
                ui.set_status_text(
                    "Peer's first message is required for an inbound session".into(),
                );
                return;
            }

            let mut mgr = manager.borrow_mut();
            let result: Result<Option<String>, String> = (|| {
                let user = mgr
                    .get_current_user_mut()
                    .ok_or_else(|| "No user selected".to_string())?;
                if is_inbound {
                    user.session_manager.establish_in_session(
                        &mut user.account,
                        name.as_str(),
                        peer_keys.as_str(),
                        first_msg.as_str(),
                    )?;
                    user.session_manager.select_session(name.as_str())?;
                    Ok(None)
                } else {
                    user.session_manager.establish_out_session(
                        &mut user.account,
                        name.as_str(),
                        peer_keys.as_str(),
                    )?;
                    user.session_manager.select_session(name.as_str())?;
                    let init = user.session_manager.encrypt("")?;
                    Ok(Some(init))
                }
            })();

            match result {
                Ok(init_opt) => {
                    let _ = mgr.autosave();
                    let sessions = session_names(&mgr);
                    let lines = history_lines(&mgr);
                    drop(mgr);
                    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                    ui.set_current_session(name);
                    messages.set_vec(lines);
                    match init_opt {
                        Some(init) => {
                            ui.set_init_message(init.into());
                            ui.set_status_text(
                                "Outbound session created — send the init message to your peer"
                                    .into(),
                            );
                        }
                        None => {
                            ui.set_init_message(SharedString::new());
                            ui.set_creating_session(false);
                            ui.set_status_text("Inbound session created".into());
                        }
                    }
                }
                Err(e) => {
                    drop(mgr);
                    ui.set_status_text(e.into());
                }
            }
        });
    }

    // ---- Encrypt: plaintext -> ciphertext (stored, then reload history) ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let messages = messages.clone();
        ui.on_do_encrypt(move |plaintext| {
            let ui = ui_weak.unwrap();
            if plaintext.is_empty() {
                return;
            }
            let mut mgr = manager.borrow_mut();
            match mgr.encrypt(plaintext.as_str()) {
                Ok(ciphertext) => {
                    let _ = mgr.autosave();
                    let lines = history_lines(&mgr);
                    drop(mgr);
                    ui.set_output_text(ciphertext.into());
                    ui.set_message_input(SharedString::new());
                    messages.set_vec(lines);
                    ui.set_status_text("Encrypted — copy the ciphertext below".into());
                }
                Err(e) => {
                    drop(mgr);
                    ui.set_status_text(e.into());
                }
            }
        });
    }

    // ---- Decrypt: ciphertext -> plaintext (stored, then reload history) ----
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let messages = messages.clone();
        ui.on_do_decrypt(move |ciphertext| {
            let ui = ui_weak.unwrap();
            if ciphertext.is_empty() {
                return;
            }
            let mut mgr = manager.borrow_mut();
            match mgr.decrypt(ciphertext.as_str()) {
                Ok(_plaintext) => {
                    let _ = mgr.autosave();
                    let lines = history_lines(&mgr);
                    drop(mgr);
                    ui.set_message_input(SharedString::new());
                    messages.set_vec(lines);
                    ui.set_status_text("Decrypted".into());
                }
                Err(e) => {
                    drop(mgr);
                    ui.set_status_text(e.into());
                }
            }
        });
    }

    // ---- Copy outgoing ciphertext to the system clipboard ----
    {
        let ui_weak = ui.as_weak();
        ui.on_copy_output(move || {
            let ui = ui_weak.unwrap();
            let text = ui.get_output_text();
            if text.is_empty() {
                return;
            }
            match set_clipboard(text.as_str()) {
                Ok(()) => ui.set_status_text("Copied to clipboard".into()),
                Err(e) => ui.set_status_text(format!("Clipboard error: {}", e).into()),
            }
        });
    }

    // ---- Paste clipboard contents into the input field ----
    {
        let ui_weak = ui.as_weak();
        ui.on_paste_input(move || {
            let ui = ui_weak.unwrap();
            match get_clipboard() {
                Ok(text) => {
                    ui.set_message_input(text.into());
                    ui.set_status_text("Pasted from clipboard".into());
                }
                Err(e) => ui.set_status_text(format!("Clipboard error: {}", e).into()),
            }
        });
    }

    ui.run().map_err(|e| e.to_string())?;

    drop(ui);
    match Rc::try_unwrap(manager) {
        Ok(cell) => cell.into_inner().shutdown()?,
        Err(rc) => {
            let _ = rc.borrow_mut().autosave();
        }
    }

    Ok(())
}

/// Session names of the currently logged-in user (empty if none).
fn session_names(mgr: &UserManager) -> Vec<SharedString> {
    mgr.get_current_user()
        .map(|user| {
            user.session_manager
                .get_session_names()
                .into_iter()
                .map(SharedString::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Decrypted transcript of the current session as renderable chat lines.
fn history_lines(mgr: &UserManager) -> Vec<ChatLine> {
    mgr.get_session_history()
        .map(|items| {
            items
                .into_iter()
                .map(|(ts, sender, text)| ChatLine {
                    sender: sender.into(),
                    text: text.into(),
                    time: fmt_time(ts),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Writes text to the OS clipboard (layout-independent, unlike Ctrl+C).
fn set_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| e.to_string())
}

/// Reads text from the OS clipboard.
fn get_clipboard() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

/// Formats a Unix timestamp (seconds) as local `YYYY-MM-DD HH:MM`.
fn fmt_time(ts: u64) -> SharedString {
    let dt = chrono::DateTime::<chrono::Local>::from(
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts),
    );
    dt.format("%Y-%m-%d %H:%M").to_string().into()
}
