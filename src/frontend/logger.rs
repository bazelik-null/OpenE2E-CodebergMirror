use colorize::AnsiColor;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

pub struct OpenE2ELogger;

impl Log for OpenE2ELogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
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
            let colored_target = record.target().to_string().black();

            eprintln!("[{}] {}: {}", colored_level, colored_target, record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: OpenE2ELogger = OpenE2ELogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug))
}
