/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

fn main() {
    // Only compile the Slint UI when the `gui` feature is enabled.
    if std::env::var_os("CARGO_FEATURE_GUI").is_some() {
        slint_build::compile("src/frontend/gui/slint/main.slint")
            .expect("failed to compile Slint UI (src/frontend/gui/slint/main.slint)");
    }
}
