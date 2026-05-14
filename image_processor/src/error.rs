use std::fmt;

/// Ошибки приложения процессора изображений.
#[derive(Debug)]
pub enum ProcessorError {
    Io(String),
    Image(String),
    Plugin(String),
    Args(String),
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessorError::Io(msg) => write!(f, "ошибка ввода-вывода: {msg}"),
            ProcessorError::Image(msg) => write!(f, "ошибка изображения: {msg}"),
            ProcessorError::Plugin(msg) => write!(f, "ошибка плагина: {msg}"),
            ProcessorError::Args(msg) => write!(f, "ошибка аргументов: {msg}"),
        }
    }
}

impl std::error::Error for ProcessorError {}

impl From<std::io::Error> for ProcessorError {
    fn from(e: std::io::Error) -> Self {
        ProcessorError::Io(e.to_string())
    }
}

impl From<image::ImageError> for ProcessorError {
    fn from(e: image::ImageError) -> Self {
        ProcessorError::Image(e.to_string())
    }
}
