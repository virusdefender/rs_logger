use std::{
    fs::File,
    io,
    io::Write,
    sync::{Arc, Mutex},
};

/// LogWriter is used to write log to a specific output, such as stdout, stderr or a file
pub trait LogWriter: Sync + Send + 'static {
    type Stream: Write;

    fn get(&self) -> Self::Stream;
}

/// Stdout is used to write log to stdout
pub struct Stdout;

impl LogWriter for Stdout {
    type Stream = io::StdoutLock<'static>;

    fn get(&self) -> Self::Stream {
        io::stdout().lock()
    }
}

/// Stderr is used to write log to stderr
pub struct Stderr;

impl LogWriter for Stderr {
    type Stream = io::StderrLock<'static>;

    fn get(&self) -> Self::Stream {
        io::stderr().lock()
    }
}

/// SharedFile is a thread-safe wrapper around a file that allows multiple threads to write to it concurrently
#[derive(Clone)]
pub struct SharedFile(Arc<Mutex<File>>);

impl Write for SharedFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut file = self.0.lock().unwrap();
        file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut file = self.0.lock().unwrap();
        file.flush()
    }
}

/// LogFileWriter is a simple implementation of LogWriter that writes logs to a single file.
pub struct LogFileWriter {
    file: SharedFile,
}

impl LogFileWriter {
    pub fn new(file: File) -> Self {
        Self { file: SharedFile(Arc::new(Mutex::new(file))) }
    }
}

impl LogWriter for LogFileWriter {
    type Stream = SharedFile;

    fn get(&self) -> Self::Stream {
        self.file.clone()
    }
}
