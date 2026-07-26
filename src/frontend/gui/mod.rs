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
use crate::frontend::localization::Localization;

slint::include_modules!();

mod auth;
mod chat;
mod clipboard;
mod i18n;
mod session;

type Manager = Rc<RefCell<UserManager>>;
type Messages = Rc<VecModel<ChatLine>>;
type Localizer = Rc<RefCell<Localization>>;

/// Launches the Slint GUI frontend.
///
/// `UserManager` lives on the UI (main) thread inside an `Rc<RefCell<…>>`; every
/// callback runs on that same thread, so there is no cross-thread sharing. Each
/// callback mutates the manager inside a short borrow scope, then pushes the new
/// state back into the window through setters.
pub fn run() -> Result<(), String> {
    force_software_renderer();

    let ui = MainWindow::new().map_err(|e| e.to_string())?;
    let manager: Manager = Rc::new(RefCell::new(UserManager::new()?));
    let messages: Messages = Rc::new(VecModel::default());
    ui.set_messages(ModelRc::from(messages.clone()));
    refresh_users(&ui, &manager);

    let loc: Localizer = Rc::new(RefCell::new(Localization::new("en")?));
    ui.set_t(i18n::build_strings(&loc.borrow()));
    ui.set_language("en".into());

    auth::wire_auth(&ui, &manager, &messages, &loc);
    session::wire_session(&ui, &manager, &messages, &loc);
    chat::wire_chat(&ui, &manager, &messages, &loc);
    clipboard::wire_clipboard(&ui, &loc);
    wire_language(&ui, &loc);

    ui.run().map_err(|e| e.to_string())?;

    shutdown(ui, manager);
    Ok(())
}

/// The default FemtoVG (OpenGL) renderer leaves the window invisible on machines
/// with limited GL (VMs, RDP, some GPU drivers). Default to the software
/// renderer, which needs no GPU, unless the user overrides it.
fn force_software_renderer() {
    if std::env::var_os("SLINT_BACKEND").is_none() {
        // Safe: called before any other thread is spawned.
        unsafe { std::env::set_var("SLINT_BACKEND", "winit-software") };
    }
}

/// Flush state and stop the autosave worker cleanly. Dropping `ui` releases the
/// callback closures holding the other `Rc` clones, so the manager can be
/// reclaimed and consumed by `shutdown(self)`.
fn shutdown(ui: MainWindow, manager: Manager) {
    drop(ui);
    match Rc::try_unwrap(manager) {
        Ok(cell) => {
            if let Err(e) = cell.into_inner().shutdown() {
                log::error!("shutdown failed: {}", e);
            }
        }
        Err(rc) => {
            if let Err(e) = rc.borrow_mut().autosave() {
                log::error!("final autosave failed: {}", e);
            }
        }
    }
}

/// Sets a localized success/info status message (rendered in gray).
fn status(ui: &MainWindow, loc: &Localizer, key: &str) {
    ui.set_status_text(loc.borrow().get(key).into());
    ui.set_status_is_error(false);
}

/// Sets a localized error status message (rendered in red).
fn fail_key(ui: &MainWindow, loc: &Localizer, key: &str) {
    fail(ui, &loc.borrow().get(key));
}

/// Sets a raw error status message (rendered in red).
fn fail(ui: &MainWindow, msg: &str) {
    ui.set_status_text(msg.into());
    ui.set_status_is_error(true);
}

/// Wires the language toggle (en ⇄ ru) and rebuilds the static strings on switch.
fn wire_language(ui: &MainWindow, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let loc = loc.clone();
    ui.on_toggle_language(move || {
        let ui = ui_weak.unwrap();
        let next = if ui.get_language() == "en" { "ru" } else { "en" };
        {
            let mut l = loc.borrow_mut();
            if let Err(e) = l.set_locale(next) {
                log::error!("set_locale failed: {}", e);
                return;
            }
        }
        ui.set_t(i18n::build_strings(&loc.borrow()));
        ui.set_language(next.into());
    });
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

/// Replaces the transcript model and updates `message-count`, which drives the
/// auto-scroll-to-bottom in the UI.
fn refresh_messages(ui: &MainWindow, messages: &Messages, lines: Vec<ChatLine>) {
    let count = lines.len() as i32;
    messages.set_vec(lines);
    ui.set_message_count(count);
}

/// Refreshes the list of existing usernames shown on the login screen.
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

/// Formats a Unix timestamp (seconds) as local `YYYY-MM-DD HH:MM`.
fn fmt_time(ts: u64) -> SharedString {
    let dt = chrono::DateTime::<chrono::Local>::from(
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts),
    );
    dt.format("%Y-%m-%d %H:%M").to_string().into()
}
