/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use slint::{ComponentHandle, SharedString};

use super::{
    ChatLine, Localizer, MainWindow, Service, Messages, RepositoryCell, fail, fail_key,
    get_chat_history, refresh_messages, status,
};

use crate::backend::services::message_service;

pub(super) fn wire_chat(
    ui: &MainWindow,
    repository: &RepositoryCell,
    service: &Service,
    messages: &Messages,
    loc: &Localizer,
) {
    wire_encrypt(ui, repository, service, messages, loc);
    wire_decrypt(ui, repository, service, messages, loc);
    wire_submit_input(ui, repository, service, messages, loc);
}

// Encrypt

fn wire_encrypt(
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

    ui.on_do_encrypt(move |plaintext| {
        encrypt_op(
            &ui_weak.unwrap(),
            &repository,
            &service,
            &messages,
            &loc,
            plaintext.as_str(),
        );
    });
}

fn encrypt_op(
    ui: &MainWindow,
    repository: &RepositoryCell,
    service: &Service,
    messages: &Messages,
    loc: &Localizer,
    plaintext: &str,
) {
    if plaintext.is_empty() {
        return;
    }

    match perform_encryption(repository, service, plaintext) {
        Ok((ciphertext, lines)) => {
            ui.set_output_text(ciphertext.into());
            ui.set_message_input(SharedString::new());
            refresh_messages(ui, messages, lines);
            status(ui, loc, "status-encrypted");
        }
        Err(e) => fail(ui, &e),
    }
}

fn perform_encryption(
    repository: &RepositoryCell,
    service: &Service,
    plaintext: &str,
) -> Result<(String, Vec<ChatLine>), String> {
    let mut srv = service.borrow_mut();
    let repo = repository.borrow();

    let user = srv.get_current_user_mut().ok_or("User not found")?;

    let ciphertext = message_service::encrypt(&repo.db_handle, user, plaintext)?;

    if let Err(e) = srv.autosave(&repo.db_handle) {
        log::error!("Autosave after encrypt failed: {}", e);
    }

    Ok((ciphertext, get_chat_history(&srv, &repo)))
}

// Decrypt

fn wire_decrypt(
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

    ui.on_do_decrypt(move |ciphertext| {
        decrypt_op(
            &ui_weak.unwrap(),
            &repository,
            &service,
            &messages,
            &loc,
            ciphertext.as_str(),
        );
    });
}

fn decrypt_op(
    ui: &MainWindow,
    repository: &RepositoryCell,
    service: &Service,
    messages: &Messages,
    loc: &Localizer,
    ciphertext: &str,
) {
    if ciphertext.is_empty() {
        return;
    }

    match perform_decryption(repository, service, ciphertext) {
        Ok(lines) => {
            ui.set_message_input(SharedString::new());
            refresh_messages(ui, messages, lines);
            status(ui, loc, "status-decrypted");
        }
        Err(e) => fail(ui, &e),
    }
}

fn perform_decryption(
    repository: &RepositoryCell,
    service: &Service,
    ciphertext: &str,
) -> Result<Vec<ChatLine>, String> {
    let mut srv = service.borrow_mut();
    let repo = repository.borrow();

    let user = srv.get_current_user_mut().ok_or("User not found")?;

    let _plaintext =
        message_service::decrypt(&repo.db_handle, user, ciphertext).map_err(|e| e.to_string())?;

    if let Err(e) = srv.autosave(&repo.db_handle) {
        log::error!("Autosave after decrypt failed: {}", e);
    }

    Ok(get_chat_history(&srv, &repo))
}

// Submit Input

fn wire_submit_input(
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

    ui.on_submit_input(move |text| {
        let ui = ui_weak.unwrap();

        if text.trim().is_empty() {
            fail_key(&ui, &loc, "status-nothing");
            return;
        }

        if looks_like_ciphertext(text.as_str()) {
            decrypt_op(&ui, &repository, &service, &messages, &loc, text.as_str());
        } else {
            encrypt_op(&ui, &repository, &service, &messages, &loc, text.as_str());
        }
    });
}

// Utilities

fn looks_like_ciphertext(text: &str) -> bool {
    use vodozemac::olm::{Message, PreKeyMessage};

    let text = text.trim();

    // Try parsing as OLM ciphertext
    PreKeyMessage::from_base64(text).is_ok() || Message::from_base64(text).is_ok()
}
