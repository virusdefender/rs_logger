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

/// LogAppender 可以在level、时间等信息后面和真正的日志信息前面添加额外的信息
/// 比如添加 trace id、请求 id 等信息
pub trait LogAppender: Send + Sync + 'static {
    fn append<W: Write>(stream: &mut W) -> bool;
}

pub struct NopAppender;

impl LogAppender for NopAppender {
    fn append<W: Write>(_stream: &mut W) -> bool {
        false
    }
}

pub struct BaseLogger<A: LogAppender> {
    level: LevelFilter,
    _appender: PhantomData<A>,
}

impl<A: LogAppender> BaseLogger<A> {
    /// 初始化并注册日志记录器
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

    /// 无视 Level，不依赖日志框架，直接打印日志
    /// 用于日志框架初始化之前的简单打印日志需求
    pub fn print(level: Level, module: &str, message: &str) {
        let stream = io::stderr();
        let mut stream = stream.lock();
        let _ = writeln!(stream, "[{} {} {}] - {}", Self::now(), Self::styled_level(level), module, message);
        let _ = stream.flush();
    }

    fn styled_level(level: Level) -> &'static str {
        if cfg!(feature = "log_level_color") {
            static LOG_LEVEL_NAMES: [&str; 6] = [
                "\x1b[37mOFF\x1b[0m",     // 默认颜色 (白色)
                "\x1b[91;1mERROR\x1b[0m", // 亮红色并加粗
                "\x1b[33mWARN\x1b[0m",    // 黄色
                "\x1b[32mINFO\x1b[0m",    // 绿色
                "\x1b[34mDEBUG\x1b[0m",   // 蓝色
                "\x1b[36mTRACE\x1b[0m",   // 青色
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
        // append 信息之后，需要添加一个额外的空格
        if A::append(&mut stream) {
            let _ = writeln!(stream, " - {}", record.args());
        } else {
            let _ = writeln!(stream, "- {}", record.args());
        }
        let _ = stream.flush();
    }

    fn flush(&self) {}
}

/// 默认 Logger
pub type Logger = BaseLogger<NopAppender>;

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
            let _ = write!(stream, "[PID: {}] ", pid);
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
