/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use base85;

use crate::error_mapper::MapErrorToString;

pub fn encode(text: &[u8]) -> String {
    base85::encode(text)
}

pub fn decode(text: &[u8]) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(text).map_err_to_string()?;
    base85::decode(s).map_err_to_string()
}
