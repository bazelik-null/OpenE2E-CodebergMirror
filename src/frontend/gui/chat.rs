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
    Localizer, MainWindow, Manager, Messages, fail, fail_key, history_lines, refresh_messages,
    status,
};

pub(super) fn wire_chat(ui: &MainWindow, manager: &Manager, messages: &Messages, loc: &Localizer) {
    {
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

    {
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

    // Enter in the input field: auto-route by content. An OLM ciphertext decodes
    // as base64; plaintext does not. Empty input is reported, not silently ignored.
    {
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
                log::error!("autosave after encrypt failed: {}", e);
            }
            (ciphertext, history_lines(&mgr))
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
                log::error!("autosave after decrypt failed: {}", e);
            }
            history_lines(&mgr)
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

fn looks_like_ciphertext(text: &str) -> bool {
    use vodozemac::olm::{Message, PreKeyMessage};

    let text = text.trim();
    if PreKeyMessage::from_base64(text).is_ok() || Message::from_base64(text).is_ok() {
        return true;
    }

    if text.len() < 40 {
        return false;
    }
    let base64ish = text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
        .count();
    // At least 90% base64-like characters => almost certainly a (broken) ciphertext.
    base64ish * 10 >= text.len() * 9
}
