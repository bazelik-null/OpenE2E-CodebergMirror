/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use crate::frontend::fluent_service::Localization;

use super::Strings;

/// Builds the Slint `Strings` bundle of static UI labels for the current locale.
pub(super) fn build_strings(loc: &Localization) -> Strings {
    Strings {
        login_as: loc.get("gui-login-as").into(),
        password: loc.get("gui-password").into(),
        log_in: loc.get("gui-log-in").into(),
        back: loc.get("gui-back").into(),
        select_user: loc.get("gui-select-user").into(),
        create_new_user: loc.get("gui-create-new-user").into(),
        username: loc.get("gui-username").into(),
        create_user: loc.get("gui-create-user").into(),
        log_out: loc.get("gui-log-out").into(),
        session: loc.get("gui-session").into(),
        new_session: loc.get("gui-new-session").into(),
        wizard_title: loc.get("gui-wizard-title").into(),
        session_name: loc.get("gui-session-name").into(),
        inbound: loc.get("gui-inbound").into(),
        gen_keys: loc.get("gui-gen-keys").into(),
        my_keys_label: loc.get("gui-my-keys-label").into(),
        peer_keys_label: loc.get("gui-peer-keys-label").into(),
        first_msg_label: loc.get("gui-first-msg-label").into(),
        create_session: loc.get("gui-create-session").into(),
        close: loc.get("gui-close").into(),
        init_msg_label: loc.get("gui-init-msg-label").into(),
        history: loc.get("gui-history").into(),
        message_placeholder: loc.get("gui-message-placeholder").into(),
        encrypt: loc.get("gui-encrypt").into(),
        decrypt: loc.get("gui-decrypt").into(),
        paste: loc.get("gui-paste").into(),
        outgoing_label: loc.get("gui-outgoing-label").into(),
        copy: loc.get("gui-copy").into(),
        previous: loc.get("gui-previous").into(),
        next: loc.get("gui-next").into(),
        ready_to_create: loc.get("gui-ready-to-create").into(),
    }
}
