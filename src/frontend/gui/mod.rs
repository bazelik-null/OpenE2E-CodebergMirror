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
use crate::frontend::fluent_manager::Localization;

slint::include_modules!();

mod auth;
mod chat;
mod clipboard;
mod localization;
mod session;

// Type aliases for wrappers
type Manager = Rc<RefCell<UserManager>>;
type Messages = Rc<VecModel<ChatLine>>;
type Localizer = Rc<RefCell<Localization>>;

// Public API

/// Launches the Slint GUI frontend
pub fn run() -> Result<(), String> {
    force_software_renderer();

    let ui = MainWindow::new().map_err(|e| e.to_string())?;
    let manager = Rc::new(RefCell::new(UserManager::new()?));
    let messages = Rc::new(VecModel::default());

    initialize_ui(&ui, &manager, &messages)?;
    wire_callbacks(&ui, &manager, &messages);

    ui.run().map_err(|e| e.to_string())?;

    cleanup(ui, manager);
    Ok(())
}

// Initialization

fn initialize_ui(ui: &MainWindow, manager: &Manager, messages: &Messages) -> Result<(), String> {
    ui.set_messages(ModelRc::from(messages.clone()));
    refresh_users(ui, manager);

    let loc = Rc::new(RefCell::new(Localization::new("en")?));
    ui.set_t(localization::build_strings(&loc.borrow()));
    ui.set_language("en".into());

    Ok(())
}

fn wire_callbacks(ui: &MainWindow, manager: &Manager, messages: &Messages) {
    let loc = Rc::new(RefCell::new(Localization::new("en").unwrap()));

    auth::wire_auth(ui, manager, messages, &loc);
    session::wire_session(ui, manager, messages, &loc);
    chat::wire_chat(ui, manager, messages, &loc);
    clipboard::wire_clipboard(ui, &loc);
    wire_language(ui, &loc);
}

// Shutdown & Cleanup

/// Flush state and stop the autosave worker cleanly
fn cleanup(ui: MainWindow, manager: Manager) {
    drop(ui);

    match Rc::try_unwrap(manager) {
        Ok(cell) => {
            if let Err(e) = cell.into_inner().shutdown() {
                log::error!("Shutdown failed: {}", e);
            }
        }
        Err(rc) => {
            if let Err(e) = rc.borrow_mut().autosave() {
                log::error!("Final autosave failed: {}", e);
            }
        }
    }
}

// Status Messages

/// Sets a localized success/info status message
fn status(ui: &MainWindow, loc: &Localizer, key: &str) {
    let message = loc.borrow().get(key);
    ui.set_status_text(message.into());
    ui.set_status_is_error(false);
}

/// Sets a localized error status message
fn fail_key(ui: &MainWindow, loc: &Localizer, key: &str) {
    let message = loc.borrow().get(key);
    fail(ui, &message);
}

/// Sets a raw error status message
fn fail(ui: &MainWindow, msg: &str) {
    ui.set_status_text(msg.into());
    ui.set_status_is_error(true);
}

// UI Wiring

/// Wires the language toggle and rebuilds static strings on switch
fn wire_language(ui: &MainWindow, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let loc = loc.clone();

    ui.on_toggle_language(move || {
        let ui = match ui_weak.upgrade() {
            Some(ui) => ui,
            None => return,
        };

        let current_lang = ui.get_language();
        let next_lang = if current_lang == "en" { "ru" } else { "en" };

        let mut localizer = loc.borrow_mut();
        if let Err(e) = localizer.set_locale(next_lang) {
            log::error!("Failed to set locale: {}", e);
            return;
        }
        drop(localizer);

        ui.set_t(localization::build_strings(&loc.borrow()));
        ui.set_language(next_lang.into());
    });
}

// Data Retrieval & Transformation

/// Gets the session names of the currently logged-in user
fn get_session_names(manager: &UserManager) -> Vec<SharedString> {
    manager
        .get_current_user()
        .map(|user| user.session_manager.get_session_names())
        .map(|names| names.into_iter().map(SharedString::from).collect())
        .unwrap_or_default()
}

/// Gets the decrypted transcript of the current session as chat lines
fn get_chat_history(manager: &UserManager) -> Vec<ChatLine> {
    manager
        .get_session_history()
        .map(|items| {
            items
                .into_iter()
                .map(|(timestamp, sender, text)| ChatLine {
                    sender: sender.into(),
                    text: text.into(),
                    time: format_timestamp(timestamp),
                })
                .collect()
        })
        .unwrap_or_default()
}

// UI Updates

/// Refreshes the transcript model and updates message count for auto-scroll
fn refresh_messages(ui: &MainWindow, messages: &Messages, lines: Vec<ChatLine>) {
    let count = lines.len() as i32;
    messages.set_vec(lines);
    ui.set_message_count(count);
}

/// Refreshes the list of existing usernames shown on the login screen
fn refresh_users(ui: &MainWindow, manager: &Manager) {
    let users = {
        let mgr = manager.borrow();
        mgr.get_usernames()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>()
    };
    ui.set_users(ModelRc::new(VecModel::from(users)));
}

// Utilities

/// Ensures the software renderer is used if no backend is explicitly set
/// The default FemtoVG (OpenGL) renderer leaves the window invisible on machines with limited GL support (VMs, RDP, some GPU drivers)
/// This sets the software renderer as the default, which requires no GPU
fn force_software_renderer() {
    if std::env::var_os("SLINT_BACKEND").is_none() {
        // SAFETY: Called before any other thread is spawned.
        unsafe { std::env::set_var("SLINT_BACKEND", "winit-software") };
    }
}

/// Formats a Unix timestamp as local time
fn format_timestamp(ts: u64) -> SharedString {
    let datetime = chrono::DateTime::<chrono::Local>::from(
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts),
    );
    datetime.format("%Y-%m-%d %H:%M").to_string().into()
}
