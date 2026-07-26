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
    Localizer, MainWindow, Manager, Messages, fail, fail_key, refresh_messages, refresh_users,
    session_names, status,
};

pub(super) fn wire_auth(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let loc = loc.clone();
        ui.on_login(move |name, password| {
            let ui = ui_weak.unwrap();
            if name.trim().is_empty() || password.is_empty() {
                fail_key(&ui, &loc, "status-credentials-required");
                return;
            }
            let result = {
                let mut mgr = manager.borrow_mut();
                let logged_in = mgr.login(name.as_str(), password.as_str());
                logged_in.map(|()| session_names(&mgr))
            };
            match result {
                Ok(sessions) => {
                    ui.set_logged_in(true);
                    ui.set_current_user(name);
                    ui.set_login_user(SharedString::new());
                    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                    status(&ui, &loc, "status-logged-in");
                }
                Err(e) => fail(&ui, &e),
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let loc = loc.clone();
        ui.on_create_user(move |name, password| {
            let ui = ui_weak.unwrap();
            if name.trim().is_empty() || password.is_empty() {
                fail_key(&ui, &loc, "status-credentials-required");
                return;
            }
            let result = {
                let mut mgr = manager.borrow_mut();
                let created = mgr
                    .new_user(name.as_str(), password.as_str())
                    .and_then(|()| mgr.login(name.as_str(), password.as_str()))
                    .and_then(|()| mgr.autosave());
                created.map(|()| session_names(&mgr))
            };
            match result {
                Ok(sessions) => {
                    refresh_users(&ui, &manager);
                    ui.set_logged_in(true);
                    ui.set_current_user(name);
                    ui.set_login_user(SharedString::new());
                    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                    status(&ui, &loc, "status-user-created");
                }
                Err(e) => fail(&ui, &e),
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let messages = messages.clone();
        let loc = loc.clone();
        ui.on_logout(move || {
            let ui = ui_weak.unwrap();
            {
                let mut mgr = manager.borrow_mut();
                mgr.logout();
                if let Err(e) = mgr.autosave() {
                    log::error!("autosave on logout failed: {}", e);
                }
            }
            refresh_messages(&ui, &messages, Vec::new());
            refresh_users(&ui, &manager);
            ui.set_logged_in(false);
            ui.set_creating_session(false);
            ui.set_current_user(SharedString::new());
            ui.set_login_user(SharedString::new());
            ui.set_current_session(SharedString::new());
            ui.set_output_text(SharedString::new());
            ui.set_my_keys(SharedString::new());
            ui.set_init_message(SharedString::new());
            ui.set_sessions(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
            status(&ui, &loc, "status-logged-out");
        });
    }

    {
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
}
