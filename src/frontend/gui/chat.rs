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
    Localizer, MainWindow, Manager, Messages, fail, fail_key, get_chat_history, refresh_messages,
    status,
};

pub(super) fn wire_chat(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    wire_encrypt(ui, manager, messages, loc);
    wire_decrypt(ui, manager, messages, loc);
    wire_submit_input(ui, manager, messages, loc);
}

// Encrypt

fn wire_encrypt(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_do_encrypt(move |plaintext| {
        encrypt_op(
            &ui_weak.unwrap(),
            &manager,
            &messages,
            &loc,
            plaintext.as_str(),
        );
    });
}

fn encrypt_op(
    ui: &MainWindow,
    manager: &Manager,
    messages: &Messages,
    loc: &Localizer,
    plaintext: &str,
) {
    if plaintext.is_empty() {
        return;
    }

    let result = {
        let mut mgr = manager.borrow_mut();
        mgr.encrypt(plaintext).map(|ciphertext| {
            if let Err(e) = mgr.autosave() {
                log::error!("Autosave after encrypt failed: {}", e);
            }
            (ciphertext, get_chat_history(&mgr))
        })
    };

    match result {
        Ok((ciphertext, lines)) => {
            ui.set_output_text(ciphertext.into());
            ui.set_message_input(SharedString::new());
            refresh_messages(ui, messages, lines);
            status(ui, loc, "status-encrypted");
        }
        Err(e) => fail(ui, &e),
    }
}

// Decrypt

fn wire_decrypt(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_do_decrypt(move |ciphertext| {
        decrypt_op(
            &ui_weak.unwrap(),
            &manager,
            &messages,
            &loc,
            ciphertext.as_str(),
        );
    });
}

fn decrypt_op(
    ui: &MainWindow,
    manager: &Manager,
    messages: &Messages,
    loc: &Localizer,
    ciphertext: &str,
) {
    if ciphertext.is_empty() {
        return;
    }

    let result = {
        let mut mgr = manager.borrow_mut();
        mgr.decrypt(ciphertext).map(|_plaintext| {
            if let Err(e) = mgr.autosave() {
                log::error!("Autosave after encrypt failed: {}", e);
            }
            get_chat_history(&mgr)
        })
    };

    match result {
        Ok(lines) => {
            ui.set_message_input(SharedString::new());
            refresh_messages(ui, messages, lines);
            status(ui, loc, "status-decrypted");
        }
        Err(e) => fail(ui, &e),
    }
}

// Submit Input

fn wire_submit_input(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    let ui_weak = ui.as_weak();
    let manager = manager.clone();
    let messages = messages.clone();
    let loc = loc.clone();

    ui.on_submit_input(move |text| {
        let ui = ui_weak.unwrap();

        if text.trim().is_empty() {
            fail_key(&ui, &loc, "status-nothing");
            return;
        }

        if looks_like_ciphertext(text.as_str()) {
            decrypt_op(&ui, &manager, &messages, &loc, text.as_str());
        } else {
            encrypt_op(&ui, &manager, &messages, &loc, text.as_str());
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
