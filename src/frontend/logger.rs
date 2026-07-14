use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

pub struct OpenE2ELogger;

impl Log for OpenE2ELogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // [LEVEL]: target: message
            eprintln!(
                "[{}]: {}: {}",
                record.level().as_str().to_uppercase(),
                record.target(),
                record.args()
            )
        }
    }

    fn flush(&self) {}
}

static LOGGER: OpenE2ELogger = OpenE2ELogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
