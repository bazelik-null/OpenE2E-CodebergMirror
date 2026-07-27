/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use super::{
    Localizer, MainWindow, Manager, Messages, fail, fail_key, get_session_names, refresh_messages,
    refresh_users, status,
};

pub(super) fn wire_auth(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    wire_login(ui, manager, loc);
    wire_create_user(ui, manager, loc);
    wire_logout(ui, manager, messages, loc);
    wire_delete_user(ui, manager, loc);
}

// Login

fn wire_login(ui: &MainWindow, manager: &Manager, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let loc = loc.clone();

    ui.on_login(move |name, password| {
        let ui = ui_weak.unwrap();

        if !validate_credentials(&ui, &loc, &name, &password) {
            return;
        }

        let result = {
            let mut mgr = manager.borrow_mut();
            mgr.login(name.as_str(), password.as_str())
                .map(|()| get_session_names(&mgr))
        };

        match result {
            Ok(sessions) => {
                handle_login_success(&ui, &name, sessions);
                status(&ui, &loc, "status-logged-in");
            }
            Err(e) => fail(&ui, &e),
        }
    });
}

fn validate_credentials(ui: &MainWindow, loc: &Localizer, name: &str, password: &str) -> bool {
    if name.trim().is_empty() || password.is_empty() {
        fail_key(ui, loc, "status-credentials-required");
        return false;
    }
    true
}

fn handle_login_success(ui: &MainWindow, name: &SharedString, sessions: Vec<SharedString>) {
    ui.set_logged_in(true);
    ui.set_current_user(name.clone());
    ui.set_login_user(SharedString::new());
    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
}

// Create User

fn wire_create_user(ui: &MainWindow, manager: &Manager, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let loc = loc.clone();

    ui.on_create_user(move |name, password| {
        let ui = ui_weak.unwrap();

        if !validate_credentials(&ui, &loc, &name, &password) {
            return;
        }

        let result = create_user_internal(&manager, name.as_str(), password.as_str());

        match result {
            Ok(sessions) => {
                refresh_users(&ui, &manager);
                handle_login_success(&ui, &name, sessions);
                status(&ui, &loc, "status-user-created");
            }
            Err(e) => fail(&ui, &e),
        }
    });
}

fn create_user_internal(
    manager: &Manager,
    name: &str,
    password: &str,
) -> Result<Vec<SharedString>, String> {
    let mut mgr = manager.borrow_mut();

    mgr.new_user(name, password)?;
    mgr.login(name, password)?;
    mgr.autosave()?;

    Ok(get_session_names(&mgr))
}

// Logout

fn wire_logout(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_logout(move || {
        let ui = ui_weak.unwrap();

        perform_logout(&manager);
        clear_ui_state(&ui, &messages);
        refresh_users(&ui, &manager);

        status(&ui, &loc, "status-logged-out");
    });
}

fn perform_logout(manager: &Manager) {
    let mut mgr = manager.borrow_mut();
    mgr.logout();
    if let Err(e) = mgr.autosave() {
        log::error!("Autosave on logout failed: {}", e);
    }
}

fn clear_ui_state(ui: &MainWindow, messages: &Messages) {
    refresh_messages(ui, messages, Vec::new());
    ui.set_logged_in(false);
    ui.set_creating_session(false);
    ui.set_current_user(SharedString::new());
    ui.set_login_user(SharedString::new());
    ui.set_current_session(SharedString::new());
    ui.set_output_text(SharedString::new());
    ui.set_my_keys(SharedString::new());
    ui.set_init_message(SharedString::new());
    ui.set_sessions(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
}

// Delete User

fn wire_delete_user(ui: &MainWindow, manager: &Manager, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let loc = loc.clone();

    ui.on_delete_user(move |name| {
        let ui = ui_weak.unwrap();

        if name.trim().is_empty() {
            return;
        }

        let result = {
            let mut mgr = manager.borrow_mut();
            mgr.delete_user(name.as_str())
        };

        match result {
            Ok(()) => {
                if ui.get_login_user() == name {
                    ui.set_login_user(SharedString::new());
                }
                refresh_users(&ui, &manager);
                status(&ui, &loc, "status-user-deleted");
            }
            Err(e) => fail(&ui, &e),
        }
    });
}
