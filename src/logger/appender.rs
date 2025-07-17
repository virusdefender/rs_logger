use std::io::Write;

/// LogAppend can add extra information after level、time and before log message.
/// For example, you can add `trace id`, `thread id` and so on
pub trait LogAppender: Send + Sync + 'static {
    fn append<W: Write>(stream: &mut W) -> bool;
}

/// NopAppender is a no-operation appender that does nothing
pub struct NopAppender;

impl LogAppender for NopAppender {
    fn append<W: Write>(_stream: &mut W) -> bool {
        false
    }
}
