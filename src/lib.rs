use std::{
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

/// LogAppend can add extra information after level、time and before log message
/// For example you can add `trace id`, `thread id` and so on
pub trait LogAppender: Send + Sync + 'static {
    fn append<W: Write>(stream: &mut W) -> bool;
}

pub struct NopAppender;

impl LogAppender for NopAppender {
    fn append<W: Write>(_stream: &mut W) -> bool {
        false
    }
}

/// Base Logger
pub struct BaseLogger<A: LogAppender> {
    level: LevelFilter,
    _appender: PhantomData<A>,
}

impl<A: LogAppender> BaseLogger<A> {
    /// Init and register logger
    pub fn init(level: LevelFilter) {
        static INIT_ONCE: Once = Once::new();
        INIT_ONCE.call_once(|| {
            let logger = Self { level, _appender: PhantomData };
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

impl<A> Log for BaseLogger<A>
where
    A: LogAppender,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        let module = record.module_path_static().unwrap_or("unknown");
        let stream = io::stderr();
        let mut stream = BufWriter::new(stream.lock());
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

/// Default Logger
pub type Logger = BaseLogger<NopAppender>;

/// log_print! can be used before the logging framework is initialized
/// 
/// ```rust
/// let level = LevelFilter::Info;
/// // this log will be printed
/// log_print!(Level::Debug, "going to init logger with level: {}", level);
/// Logger::init(level);
/// // this log will not be printedd
/// debug!("current log level: {}", level);
/// ```
#[macro_export]
macro_rules! log_print {
    ($level:path, $($arg:tt)*) => {
        $crate::Logger::print($level, module_path!(), &format!($($arg)*));
    };
}

#[test]
fn test_log_appender() {
    struct PIDLogAppender;

    impl LogAppender for PIDLogAppender {
        fn append<W: Write>(stream: &mut W) -> bool {
            let pid = std::process::id();
            let _ = write!(stream, "[PID: {}]", pid);
            true
        }
    }

    type MyLogger = BaseLogger<PIDLogAppender>;
    MyLogger::init(LevelFilter::Debug);

    log::error!("test log message");
}

#[test]
fn test_log_print_macro() {
    log_print!(Level::Debug, "test log message with log_print!");
}
