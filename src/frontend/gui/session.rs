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
    ChatLine, Localizer, MainWindow, Manager, Messages, fail, fail_key, get_chat_history,
    get_session_names, refresh_messages, status,
};
use crate::backend::objects::session::SessionInstance;

/// Result of session initialization: (optional pre-key message, session names, chat history)
type SessionInitResult = Result<(Option<String>, Vec<SharedString>, Vec<ChatLine>), String>;

pub(super) fn wire_session(
    ui: &MainWindow,
    manager: &Manager,
    messages: &Messages,
    loc: &Localizer,
) {
    wire_select_session(ui, manager, messages, loc);
    wire_generate_keys(ui, manager, loc);
    wire_create_session(ui, manager, messages, loc);
    wire_delete_session(ui, manager, messages, loc);
}

// Select Session

fn wire_select_session(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_select_session(move |name| {
        let ui = ui_weak.unwrap();
        let result = {
            let mut mgr = manager.borrow_mut();
            mgr.get_current_user_mut()
                .ok_or_else(|| "No user selected".to_string())
                .and_then(|user| user.session_manager.select_session(name.as_str()))
                .map(|()| get_chat_history(&mgr))
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

// Generate Keys

fn wire_generate_keys(ui: &MainWindow, manager: &Manager, loc: &Localizer) {
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

// Create Session

fn wire_create_session(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_session_create(move |name, is_inbound, peer_keys, first_msg| {
        let ui = ui_weak.unwrap();

        // Validate inputs
        if !validate_session_inputs(&ui, &loc, &name, &peer_keys, is_inbound, &first_msg) {
            return;
        }

        let result = create_session_internal(
            &manager,
            name.as_str(),
            is_inbound,
            peer_keys.as_str(),
            first_msg.as_str(),
        );

        match result {
            Ok((init_msg, sessions, chat_lines)) => {
                handle_session_created(&ui, &messages, &loc, &name, init_msg, sessions, chat_lines);
            }
            Err(e) => fail(&ui, &e),
        }
    });
}

fn validate_session_inputs(
    ui: &MainWindow,
    loc: &Localizer,
    name: &str,
    peer_keys: &str,
    is_inbound: bool,
    first_msg: &str,
) -> bool {
    if name.trim().is_empty() {
        fail_key(ui, loc, "status-session-name-required");
        return false;
    }

    if peer_keys.trim().is_empty() {
        fail_key(ui, loc, "status-peer-keys-required");
        return false;
    }

    if is_inbound && first_msg.trim().is_empty() {
        fail_key(ui, loc, "status-first-msg-required");
        return false;
    }

    true
}

fn create_session_internal(
    manager: &Manager,
    name: &str,
    is_inbound: bool,
    peer_keys: &str,
    first_msg: &str,
) -> SessionInitResult {
    let mut mgr = manager.borrow_mut();

    let init_msg = (|| -> Result<Option<String>, String> {
        let user = mgr
            .get_current_user_mut()
            .ok_or_else(|| "No user selected".to_string())?;

        if is_inbound {
            user.session_manager.establish_in_session(
                &mut user.account,
                name,
                peer_keys,
                first_msg,
            )?;
            user.session_manager.select_session(name)?;
            Ok(None)
        } else {
            user.session_manager
                .establish_out_session(&mut user.account, name, peer_keys)?;
            user.session_manager.select_session(name)?;
            // Generate pre-key message for peer to open inbound session
            Ok(Some(user.session_manager.encrypt("")?))
        }
    })()?;

    if let Err(e) = mgr.autosave() {
        log::error!("Autosave after session creation failed: {}", e);
    }

    Ok((init_msg, get_session_names(&mgr), get_chat_history(&mgr)))
}

fn handle_session_created(
    ui: &MainWindow,
    messages: &Messages,
    loc: &Localizer,
    name: &SharedString,
    init_msg: Option<String>,
    sessions: Vec<SharedString>,
    chat_lines: Vec<ChatLine>,
) {
    ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
    ui.set_current_session(name.clone());
    refresh_messages(ui, messages, chat_lines);

    match init_msg {
        Some(init) => {
            ui.set_init_message(init.into());
            status(ui, loc, "status-outbound-created");
        }
        None => {
            ui.set_init_message(SharedString::new());
            ui.set_creating_session(false);
            status(ui, loc, "status-inbound-created");
        }
    }
}

// Delete Session

fn wire_delete_session(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
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
            mgr.delete_session(name.as_str())
                .map(|()| get_session_names(&mgr))
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
