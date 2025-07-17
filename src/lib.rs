mod logger;

pub use logger::{appender::*, logger::*, writer::*};

/// Default Logger, will output to stderr
pub type Logger = BaseLogger<NopAppender>;
/// Logger that outputs to stdout
pub type StdoutLogger = BaseLogger<NopAppender, Stdout>;
/// Logger that outputs to a file
pub type FileLogger = BaseLogger<NopAppender, LogFileWriter>;

/// log_print! can be used before the logging framework is initialized
///
/// ```rust
/// use log::{debug, LevelFilter, Level};
/// use rs_logger::{log_print, Logger};
///
/// let level = LevelFilter::Info;
/// // this log will be printed
/// log_print!(Level::Debug, "going to init logger with level: {}", level);
/// Logger::init(level);
/// // this log will not be printed
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
    use std::io::Write;

    use log::LevelFilter;

    struct PIDLogAppender;

    impl LogAppender for PIDLogAppender {
        fn append<W: Write>(stream: &mut W) -> bool {
            let pid = std::process::id();
            let _ = write!(stream, "[PID: {pid}]");
            true
        }
    }

    type MyLogger = BaseLogger<PIDLogAppender>;
    MyLogger::init(LevelFilter::Debug);

    log::error!("test log message");
}

#[test]
fn test_log_print_macro() {
    use log::Level;

    log_print!(Level::Debug, "test log message with log_print!");
}

#[ignore]
#[test]
fn test_log_file_writer() {
    use std::fs;

    use log::LevelFilter;

    let file = fs::OpenOptions::new().create(true).write(true).truncate(true).open("test_log.txt").unwrap();
    FileLogger::init(LevelFilter::Info, file);
    log::error!("test log message to file");
    assert!(String::from_utf8(fs::read("test_log.txt").unwrap()).unwrap().contains("test log message"));
    fs::remove_file("test_log.txt").unwrap();
}
