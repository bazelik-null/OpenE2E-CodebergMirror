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
    ChatLine, Localizer, MainWindow, Messages, RepositoryCell, Service, fail, fail_key,
    get_chat_history, get_session_names, refresh_messages, status,
};
use crate::backend::objects::session::SessionInstance;
use crate::frontend::encoding;

/// Result of session initialization: (optional pre-key message, session names, chat history)
type SessionInitResult = Result<(Option<String>, Vec<SharedString>, Vec<ChatLine>), String>;

pub(super) fn wire_session(
    ui: &MainWindow,
    repository: &RepositoryCell,
    service: &Service,
    messages: &Messages,
    loc: &Localizer,
) {
    wire_select_session(ui, repository, service, messages, loc);
    wire_generate_keys(ui, service, loc);
    wire_create_session(ui, repository, service, messages, loc);
    wire_delete_session(ui, repository, service, messages, loc);
}

// Select Session

fn wire_select_session(
    ui: &MainWindow,
    repository: &RepositoryCell,
    service: &Service,
    messages: &Messages,
    loc: &Localizer,
) {
    let ui_weak = ui.as_weak();
    let service = service.clone();
    let repository = repository.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_select_session(move |name| {
        let ui = ui_weak.unwrap();
        let result = {
            let mut srv = service.borrow_mut();
            let repo = repository.borrow();
            srv.get_current_user_mut()
                .ok_or_else(|| "No user selected".to_string())
                .and_then(|user| user.session_service.select_session(name.as_str()))
                .map(|()| get_chat_history(&srv, &repo))
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

fn wire_generate_keys(ui: &MainWindow, service: &Service, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let service = service.clone();
    let loc = loc.clone();

    ui.on_session_gen_keys(move || {
        let ui = ui_weak.unwrap();
        let result = {
            let mut srv = service.borrow_mut();
            srv.get_current_user_mut()
                .ok_or_else(|| "No user selected".to_string())
                .and_then(|user| SessionInstance::generate_keys(&mut user.account))
        };

        match result {
            Ok(keys) => {
                let encoded = encoding::encode(&keys);
                ui.set_my_keys(encoded.into());
                status(&ui, &loc, "status-keys-generated");
            }
            Err(e) => fail(&ui, &e),
        }
    });
}

// Create Session

fn wire_create_session(
    ui: &MainWindow,
    repository: &RepositoryCell,
    service: &Service,
    messages: &Messages,
    loc: &Localizer,
) {
    let ui_weak = ui.as_weak();
    let service = service.clone();
    let repository = repository.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_session_create(move |name, is_inbound, peer_keys, first_msg| {
        let ui = ui_weak.unwrap();

        // Validate inputs
        if !validate_session_inputs(&ui, &loc, &name, &peer_keys, is_inbound, &first_msg) {
            return;
        }

        let result = create_session_internal(
            &repository,
            &service,
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
    repository: &RepositoryCell,
    service: &Service,
    name: &str,
    is_inbound: bool,
    peer_keys: &str,
    first_msg: &str,
) -> SessionInitResult {
    let mut srv = service.borrow_mut();
    let repo = repository.borrow();

    let init_msg = (|| -> Result<Option<String>, String> {
        let user = srv
            .get_current_user_mut()
            .ok_or_else(|| "No user selected".to_string())?;

        if is_inbound {
            let decoded_msg = encoding::decode(first_msg.as_bytes())?;
            let decoded_keys = encoding::decode(peer_keys.as_bytes())?;
            user.session_service.establish_in_session(
                &mut user.account,
                name,
                &decoded_keys,
                &decoded_msg,
            )?;
            user.session_service.select_session(name)?;
            Ok(None)
        } else {
            let decoded_keys = encoding::decode(peer_keys.as_bytes())?;
            user.session_service
                .establish_out_session(&mut user.account, name, &decoded_keys)?;
            user.session_service.select_session(name)?;

            let init_message = user.session_service.encrypt("".as_bytes())?;

            let encoded = encoding::encode(&init_message);

            Ok(Some(encoded))
        }
    })()?;

    if let Err(e) = srv.autosave(&repo.db_handle) {
        log::error!("Autosave after session creation failed: {}", e);
    }

    Ok((
        init_msg,
        get_session_names(&srv),
        get_chat_history(&srv, &repo),
    ))
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

fn wire_delete_session(
    ui: &MainWindow,
    repository: &RepositoryCell,
    service: &Service,
    messages: &Messages,
    loc: &Localizer,
) {
    let ui_weak = ui.as_weak();
    let service = service.clone();
    let repository = repository.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_delete_session(move |name| {
        let ui = match ui_weak.upgrade() {
            Some(ui) => ui,
            None => return,
        };

        if name.trim().is_empty() {
            return;
        }

        match perform_delete_session(&repository, &service, name.as_str()) {
            Ok(sessions) => {
                ui.set_sessions(ModelRc::new(VecModel::from(sessions)));
                ui.set_current_session(SharedString::new());
                refresh_messages(&ui, &messages, Vec::new());
                status(&ui, &loc, "status-session-deleted");
            }
            Err(e) => {
                log::error!("Failed to delete session: {}", e);
            }
        }
    });
}

fn perform_delete_session(
    repository: &RepositoryCell,
    service: &Service,
    name: &str,
) -> Result<Vec<SharedString>, String> {
    let mut srv = service.borrow_mut();
    let repo = repository.borrow();
    let user = srv.get_current_user_mut().ok_or("User not found")?;
    let username = user.name.clone();

    user.session_service
        .delete_session_full(&repo.db_handle, &username, name)?;

    srv.autosave(&repo.db_handle)?;

    Ok(get_session_names(&srv))
}
