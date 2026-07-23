/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use colorize::AnsiColor;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

pub struct OpenE2ELogger;

#[cfg(debug_assertions)]
const DEBUG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
const DEBUG_LEVEL: Level = Level::Error;

impl Log for OpenE2ELogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= DEBUG_LEVEL
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = record.level().as_str().to_uppercase();
            let colored_level = match record.level() {
                Level::Error => level_str.red().bold(),
                Level::Warn => level_str.yellow().bold(),
                Level::Info => level_str.green().bold(),
                Level::Debug => level_str.blue().bold(),
                Level::Trace => level_str.magenta().bold(),
            };
            let colored_target = record.target().to_string().b_black();

            eprintln!("[{}] {}: {}", colored_level, colored_target, record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: OpenE2ELogger = OpenE2ELogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug))
}
