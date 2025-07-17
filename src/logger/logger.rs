use std::{
    fs::File,
    io,
    io::{BufWriter, Write},
    marker::PhantomData,
    sync::Once,
};

use log::{Level, LevelFilter, Log, Metadata, Record};
use utc_dt::{
    UTCDatetime,
    time::{UTCTimestamp, UTCTransformations},
};

use super::{appender::*, writer::*};

/// Base Logger
pub struct BaseLogger<A: LogAppender, W: LogWriter = Stderr> {
    level: LevelFilter,
    writer: W,
    _appender: PhantomData<A>,
}

impl<A> BaseLogger<A, Stderr>
where
    A: LogAppender,
{
    pub fn init(level: LevelFilter) {
        Self::init_with_writer(level, Stderr {});
    }
}

impl<A> BaseLogger<A, Stdout>
where
    A: LogAppender,
{
    pub fn init(level: LevelFilter) {
        Self::init_with_writer(level, Stdout {});
    }
}

impl<A> BaseLogger<A, LogFileWriter>
where
    A: LogAppender,
{
    pub fn init(level: LevelFilter, file: File) {
        Self::init_with_writer(level, LogFileWriter::new(file));
    }
}

impl<A, W> BaseLogger<A, W>
where
    A: LogAppender,
    W: LogWriter,
{
    pub fn init_with_writer(level: LevelFilter, writer: W) {
        static INIT_ONCE: Once = Once::new();
        INIT_ONCE.call_once(|| {
            let logger = Self { level, writer, _appender: PhantomData };
            log::set_boxed_logger(Box::new(logger)).unwrap();
            log::set_max_level(level);
        })
    }

    fn now() -> String {
        UTCDatetime::from_timestamp(UTCTimestamp::try_from_system_time().unwrap()).as_iso_datetime(3)
    }

    /// Print log directly, can be used before the logging framework is initialized
    pub fn print(level: Level, module: &str, message: &str) {
        let stream = io::stderr();
        let mut stream = stream.lock();
        let _ = writeln!(stream, "[{} {} {}] - {}", Self::now(), Self::styled_level(level), module, message);
        let _ = stream.flush();
    }

    fn styled_level(level: Level) -> &'static str {
        if cfg!(feature = "log_level_color") {
            static LOG_LEVEL_NAMES: [&str; 6] = [
                "\x1b[37mOFF\x1b[0m",     // White
                "\x1b[91;1mERROR\x1b[0m", // Red
                "\x1b[33mWARN\x1b[0m",    // Yellow
                "\x1b[32mINFO\x1b[0m",    // Green
                "\x1b[34mDEBUG\x1b[0m",   // Blue
                "\x1b[36mTRACE\x1b[0m",   // Cyan
            ];
            LOG_LEVEL_NAMES[level as usize]
        } else {
            level.as_str()
        }
    }
}

impl<A, O> Log for BaseLogger<A, O>
where
    A: LogAppender,
    O: LogWriter,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        let module = record.module_path_static().unwrap_or("unknown");
        let mut stream = BufWriter::new(self.writer.get());

        let _ = write!(stream, "[{} {} {}] ", Self::now(), Self::styled_level(record.level()), module);
        // [time level module] - message
        // [time level module] [extra] - message
        if A::append(&mut stream) {
            let _ = write!(stream, " ");
        }
        let _ = writeln!(stream, "- {}", record.args());
        let _ = stream.flush();
    }

    fn flush(&self) {}
}
