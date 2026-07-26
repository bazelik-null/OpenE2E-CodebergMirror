/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use slint::ComponentHandle;

use super::{Localizer, MainWindow, fail, status};

pub(super) fn wire_clipboard(ui: &MainWindow, loc: &Localizer) {
    {
        let ui_weak = ui.as_weak();
        let loc = loc.clone();
        ui.on_copy_output(move || {
            let ui = ui_weak.unwrap();
            let text = ui.get_output_text();
            if text.is_empty() {
                return;
            }
            match set_clipboard(text.as_str()) {
                Ok(()) => status(&ui, &loc, "status-copied"),
                Err(e) => clipboard_error(&ui, &loc, &e),
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let loc = loc.clone();
        ui.on_paste_input(move || {
            let ui = ui_weak.unwrap();
            match get_clipboard() {
                Ok(text) => {
                    ui.set_message_input(text.into());
                    status(&ui, &loc, "status-pasted");
                }
                Err(e) => clipboard_error(&ui, &loc, &e),
            }
        });
    }
}

fn clipboard_error(ui: &MainWindow, loc: &Localizer, err: &str) {
    let prefix = loc.borrow().get("status-clipboard-error");
    fail(ui, &format!("{}: {}", prefix, err));
}

fn set_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| e.to_string())
}
fn get_clipboard() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}
