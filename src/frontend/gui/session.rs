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
    ChatLine, Localizer, MainWindow, Manager, Messages, fail, fail_key, history_lines,
    refresh_messages, session_names, status,
};
use crate::backend::objects::session::SessionInstance;

pub(super) fn wire_session(
    ui: &MainWindow,
    manager: &Manager,
    messages: &Messages,
    loc: &Localizer,
) {
    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let messages = messages.clone();
        let loc = loc.clone();
        ui.on_select_session(move |name| {
            let ui = ui_weak.unwrap();
            let result = {
                let mut mgr = manager.borrow_mut();
                let selected = mgr
                    .get_current_user_mut()
                    .ok_or_else(|| "No user selected".to_string())
                    .and_then(|user| user.session_manager.select_session(name.as_str()));
                selected.map(|()| history_lines(&mgr))
            };
            match result {
                Ok(lines) => {
                    ui.set_current_session(name);
                    refresh_messages(&ui, &messages, lines);
                    status(&ui, &loc, "status-session-opened");
                }
                Err(e) => fail(&ui, &e),
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let manager = manager.clone();
        let loc = loc.clone();
        ui.on_session_gen_keys(move || {
            let ui = ui_weak.unwrap();
            let result = {
                let mut mgr = manager.borrow_mut();
                mgr.get_current_user_mut()
                    .ok_or_else(|| "No user selected".to_string())
                    .and_then(|user| SessionInstance::generate_keys(&mut user.account))
            };
            match result {
                Ok(keys) => {
                    ui.set_my_keys(keys.into());
                    status(&ui, &loc, "status-keys-generated");
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
        ui.on_session_create(move |name, is_inbound, peer_keys, first_msg| {
            let ui = ui_weak.unwrap();
            if name.trim().is_empty() {
                fail_key(&ui, &loc, "status-session-name-required");
                return;
            }
            if peer_keys.trim().is_empty() {
                fail_key(&ui, &loc, "status-peer-keys-required");
                return;
            }
            if is_inbound && first_msg.trim().is_empty() {
                fail_key(&ui, &loc, "status-first-msg-required");
                return;
            }

            let result: Result<(Option<String>, Vec<SharedString>, Vec<ChatLine>), String> = {
                let mut mgr = manager.borrow_mut();
                let established = (|| {
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
                        // Empty pre-key message the peer needs to open the inbound side.
                        Ok(Some(user.session_manager.encrypt("")?))
                    }
                })();
                established.map(|init_opt| {
                    if let Err(e) = mgr.autosave() {
                        log::error!("autosave after session create failed: {}", e);
                    }
                    (init_opt, session_names(&mgr), history_lines(&mgr))
                })
            };

            match result {
                Ok((init_opt, sessions, lines)) => {
                    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                    ui.set_current_session(name);
                    refresh_messages(&ui, &messages, lines);
                    match init_opt {
                        Some(init) => {
                            ui.set_init_message(init.into());
                            status(&ui, &loc, "status-outbound-created");
                        }
                        None => {
                            ui.set_init_message(SharedString::new());
                            ui.set_creating_session(false);
                            status(&ui, &loc, "status-inbound-created");
                        }
                    }
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
        ui.on_delete_session(move |name| {
            let ui = ui_weak.unwrap();
            if name.trim().is_empty() {
                return;
            }
            let result = {
                let mut mgr = manager.borrow_mut();
                let deleted = mgr.delete_session(name.as_str());
                deleted.map(|()| session_names(&mgr))
            };
            match result {
                Ok(sessions) => {
                    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                    ui.set_current_session(SharedString::new());
                    refresh_messages(&ui, &messages, Vec::new());
                    status(&ui, &loc, "status-session-deleted");
                }
                Err(e) => fail(&ui, &e),
            }
        });
    }
}
