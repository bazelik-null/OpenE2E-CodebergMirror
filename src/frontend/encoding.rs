/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;

use crate::error_mapper::MapErrorToString;

pub fn encode(text: &[u8]) -> String {
    BASE64_STANDARD_NO_PAD.encode(text)
}

pub fn decode(text: &[u8]) -> Result<Vec<u8>, String> {
    BASE64_STANDARD_NO_PAD.decode(text).map_err_to_string()
}
